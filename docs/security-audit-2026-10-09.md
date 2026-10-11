# Security and conformance audit — 2026-10-09

> Written when the project was named Solutio. Paths such as `programs/solutio` are now `programs/marjan`, and `Program<Solutio>` is `Program<Marjan>`; the findings and line numbers refer to the code as audited.

Audit of `programs/solutio` (commit `8407f77`) and the surrounding repository against the **Solana AI Kit 2.3.0** (Superteam / `solanabr/ai-kit`): the house rules in `CLAUDE-solana.md`, the `/audit-solana` checklist, `/build-program`, `/test-rust`, `/profile-cu`, the `solana-qa-engineer` coverage bar and the `deployment.md` runbook.

The kit's external skill packs (`auditor-skill`, `solana-dev`) are git submodules and were not in the archive, so the checklist applied is the one written inline in `/audit-solana`, not auditor-skill's itemized checklists.

## Scope

17 instructions. The program holds no tokens and moves no lamports beyond rent; its "value" is the integrity of two ledgers: art. 86 adhesion caps (Carona) and the single-financing balance (Obligations). Instructions that change authority or what a financier relies on were reviewed first.

| Instruction | Signers | Writes | Sensitive |
| --- | --- | --- | --- |
| `init_registry` | payer, authority | registry (init) | **sets the root of trust** |
| `register_agency` | payer, registry authority | registry, agency (init) | **creates identities and attributes** |
| `create_ata`, `add_item`, `set_ata_status` | manager | ata, item | caps and record status |
| `request_adhesion` | payer, adherent | item, usage (`init_if_needed`), request (init) | **reserves cap** |
| `supplier_respond` | supplier | item, usage, request | releases cap |
| `authorize_adhesion`, `deny_adhesion`, `extend_execution` | manager | item, usage, request | releases cap |
| `formalize_adhesion` | adherent | request | executes the purchase |
| `expire_adhesion` | none (permissionless) | item, usage, request | releases cap |
| `register_obligation` | payer, debtor | obligation (init) | creates the financeable asset |
| `attach_fiscal_document` | payer, debtor | obligation, fiscal_document (init) | backs eligibility |
| `mark_eligible` | eligibility verifier | obligation | **gates financing** |
| `finance` | payer, financier, creditor | obligation, financing (init) | **single-financing invariant** |
| `record_reduction`, `record_payment` | debtor | obligation | impairment, settlement |

## Summary

| Severity | Count |
| --- | --- |
| Critical | 0 |
| High | 1 (already known, open in `DECISIONS.md`) |
| Medium | 3 |
| Low | 5 |
| Info | 7 |

What holds up well: every state account is a typed `Account<T>` (owner and discriminator checked); every stored PDA is re-validated with its stored bump; the cross-account chains (`request.item == item`, `item.ata == ata`, `ata.supplier == signer`, usage PDA from `request.adherent_agency`) close the substitution paths in Carona; all amount arithmetic is checked, `overflow-checks = true` in release; no `unwrap`/`expect` in program code; one `#[error_code]` enum, so no code collisions; no CPIs; legal rules are pure functions with randomized property tests.

## Findings

### H-1 — First caller of `init_registry` becomes the root of trust (known)

- `programs/solutio/src/instructions/registry.rs:5-28`, instruction `init_registry`
- Exploit: on any fresh deployment, whoever sends `init_registry` first (a bot watching the deploy) becomes `registry.authority` and picks `eligibility_verifier`. They can then register agencies with any sphere and attributes (e.g. `is_health_ministry = true`, lifting the §5 cap via `HealthEmergency`) and mark any obligation eligible.
- Status: already listed as pending in `docs/DECISIONS.md` and the README. Devnet is not exposed (initialized right after deploy). Blocking for any non-demo deployment.
- Fix: add `program` (`Program<Solutio>`) and `program_data` accounts with `constraint = program.programdata_address()? == Some(program_data.key())` and `constraint = program_data.upgrade_authority_address == Some(authority.key())`, or compare `authority` to a compiled-in admin key.

### M-1 — No rotation or revocation of any key

- `state.rs` (`Registry`, `Agency.active`), no instruction writes them after init
- Exploit: a leaked eligibility-verifier key can mark any obligation eligible indefinitely; a leaked agency key keeps requesting adhesions and registering obligations. `Agency.active` is set to `true` once and never cleared, so every `AgencyInactive` constraint is dead code (and that error is the only one no test can reach).
- Fix: `set_registry_authority` (two-step: propose, accept), `set_eligibility_verifier` and `set_agency_active`, all signed by the registry authority. Add the negative tests.

### M-2 — One executed adhesion can back unlimited obligations of any size

- `programs/solutio/src/instructions/obligation.rs:40-61` (`register_obligation`, `source_request` branch)
- Exploit: Prefeitura A executes an adhesion of 50 units × R$ 200,00 (R$ 10.000,00) and then registers five obligations of R$ 10.000,00 each, all citing the same `source_request`. Each one carries the provenance link that a financier reads as "derived from a lawful adhesion", although together they claim five times what was authorized.
- Fix: store `obligated_amount` on `AdhesionRequest` (writable `source_request`, plus the `item` for `unit_price`) and require `obligated_amount + verified_amount <= authorized_qty * unit_price` with checked math. Additive layout change: bump an account version or migrate before mainnet.

### M-3 — `formalize_adhesion` ignores a suspended or cancelled record

- `programs/solutio/src/instructions/carona.rs:436-461` (`FormalizeAdhesion` has no `ata` account)
- Exploit: the manager authorizes; the supplier is then sanctioned and the record suspended (`set_ata_status(Suspended)`). The adherent still formalizes the contract inside its 90-day window. `state.rs` cites Decree 11.462/2023 art. 28 §1 for exactly this case ("no new contracts may derive from the record"), so the program does not enforce a rule its own documentation claims.
- Fix: add `ata` with `constraint = request.ata == ata.key()` and `require!(ata.status == AtaStatus::Active)`. If the legal reading is that an already-authorized adhesion survives suspension, record that decision in `LEGAL_RULES.md` instead.

### L-1 — `init_if_needed` on `AgencyItemUsage`

- `carona.rs:207-212`; `Cargo.toml` enables the `init-if-needed` feature
- Kit house rule: no `init_if_needed`. The code is safe today (the PDA is unique per item and agency, and the `usage.item == Pubkey::default()` guard prevents re-initialization), but the guard is hand-written and easy to break in a refactor.
- Fix: a separate `open_usage` instruction (or create it in the same transaction from the client), then `init` only. Or record the exception and its reason in `DECISIONS.md`.

### L-2 — Pending requests never time out

- `carona.rs` `ExpireAdhesion` only lapses `Authorized` requests
- A request left in `Requested` or `SupplierAccepted` keeps its quantity reserved against the §5 pool until the supplier declines or the manager denies. A supplier that never answers stalls the item for every other agency; only an active denial frees it.
- Fix: a response deadline (e.g. N days after `requested_at`) after which `expire_adhesion` also releases pending requests, permissionlessly.

### L-3 — Record status not checked in `add_item` and `extend_execution`

- `carona.rs:100-158` and `carona.rs:421-432`
- Items can be added to a cancelled or expired record; execution can be extended while the record is suspended.
- Fix: `require!(ata.status == AtaStatus::Active)` in both; `add_item` also `now <= valid_until`.

### L-4 — Fiscal document keys can be squatted

- `obligation.rs:91-141`, PDA `[FISCAL_DOC_SEED, doc_key_hash]`
- Any registered agency can attach the hash of someone else's NF-e key to its own obligation first. The rightful obligation can then never attach it, and there is no unbind or dispute path.
- Fix: include the issuer in the check (store the CNPJ hash and require it to match the obligation's creditor attestation), or add a registry-signed `detach_fiscal_document` for disputes.

### L-5 — Wrong error for an over-long agency name

- `registry.rs:61`: `require!(name.len() <= 64, ErrorCode::InvalidQuantity)`
- Fix: a dedicated `NameTooLong` variant (append at the end of the enum so existing codes keep their numbers).

### Info

1. `space = 8 + T::INIT_SPACE` everywhere; the Anchor 1.x convention is `T::DISCRIMINATOR.len() + T::INIT_SPACE`.
2. `carona.rs:387` subtracts without `checked_sub` (guarded by the `require!` above it; clippy `arithmetic_side_effects` flags it).
3. `debug_assert_eq!` in `handle_add_item` (`carona.rs:139`) is compiled out of the release binary; move it to a unit test.
4. Events use `emit!` (program logs), which validators may truncate. For an audit trail other institutions index, `emit_cpi!` is more robust.
5. No emergency pause. The deployment runbook says a pause only exists if every instruction checks a flag, so it has to be designed in before launch.
6. `doc_key_hash` and `obligation_id` are deterministic hashes of structured, low-entropy identifiers (an NF-e key is mostly public fields plus an 8-digit code). Anyone who guesses a key can confirm it on-chain; do not describe these hashes as confidential.
7. No `close` instructions (known, `ECONOMICS.md`). When added: close only terminal states, drain all lamports and let Anchor zero the data, and keep `Executed` requests that an obligation's `source_request` points to.

## Needs verification

- Whether `solana-verify build` reproduces an SBPF v2 binary (`anchor build --arch v2`); if not, the devnet binary cannot be verified from the repository.

## Tests run

| Check (kit command) | Result |
| --- | --- |
| `cargo fmt --all --check` (`/build-program`) | **Fails**: 9 files differ (`lib.rs`, `error.rs`, `rules.rs`, the three instruction files, the three test files). No `rustfmt.toml`, so the 120-column style is not the default; either run `cargo fmt` or commit a `rustfmt.toml` with `max_width = 120` |
| `cargo clippy -p solutio --lib -- -D warnings` | Pass |
| clippy with the audit flags (`arithmetic_side_effects`, `unwrap_used`, `expect_used`, `panic`) | 1 warning (Info 2) |
| `cargo test -p solutio --lib` | 13 passed (rules, including the two randomized property tests) |
| LiteSVM integration tests (25) | **Not reproduced in this session**: they `include_bytes!` `target/deploy/solutio.so`, and the SBF toolchain host (`release.anza.xyz`) is blocked by this environment's network policy. Count matches the 38 reported (13 + 25) |
| Negative-test coverage (kit bar: every error code hit) | Not hit by any test: `AccountMismatch`, `AgencyInactive` (unreachable, M-1), `InvalidAmount` (rules only), `InvalidValidity`, `Overflow`. `AccountMismatch` matters most: it is the substitution guard; add a test passing an item from another record |
| Fuzzing (Trident, kit bar 10+ min) | None. Randomized property tests cover the pure rules, not instruction sequences with real accounts |
| CU (`/profile-cu`, from `client/devnet-costs.json`) | All measured paths 5,239–27,462 CU (≤ 14% of 200k): efficient. Worst: a refused `request_adhesion` at the global cap (27,462). Not measured: `deny_adhesion`, `extend_execution`, `expire_adhesion`, `record_reduction`, `set_ata_status` |
| `cargo audit` (382 crates) | 0 vulnerabilities; 6 warnings, all transitive through Anchor, the Solana SDK or LiteSVM: unmaintained `ansi_term`, `bincode`, `derivative`, `libsecp256k1`, `paste`, and `rand 0.7.3` unsound (RUSTSEC-2026-0097, only with a custom logger calling `rand::rng()`). Nothing to act on in this repository; recheck after Anchor upgrades |

## Deploy readiness (`deployment.md`)

| Item | Status |
| --- | --- |
| Devnet first | Done |
| Upgrade authority | Session demo key; transfer pending (`DECISIONS.md`) |
| Deploy artifact from `solana-verify build` | No: `anchor build --arch v2`, SHA-256 recorded in `DEVNET.md` |
| Executable hash recorded with the release commit | Partially (SHA-256 of the `.so`, not `solana-verify get-executable-hash`) |
| Admin keys, pause, security assumptions documented | Partially (README limitations); no pause |
| CI (build, test, clippy, audit) | None (`.github/workflows` absent) |
| `Anchor.toml` | Only `[programs.localnet]`; add `[programs.devnet]` with the deployed ID |

## Verdict

**Not ready for an external audit yet; fine for the devnet demo.** Before any non-demo use: fix H-1, M-1, M-2 and M-3, close the formatting and negative-test gaps, and adopt a verifiable build. None of the findings lets a third party exceed an art. 86 cap or finance the same balance twice in the current devnet deployment; the Medium findings are about trusted parties (registered agencies, key holders) and about rules the documentation claims but the program does not yet enforce.

## Remediation — 2026-10-09

Fixed in the program and covered by `programs/solutio/tests/security.rs` (all LiteSVM suites were run in this session: 13 rules + 17 Carona + 8 Obligations + 13 security = 51 passing).

| Finding | Fix | Test |
| --- | --- | --- |
| H-1 | `init_registry` requires the program's upgrade authority (`Program<Solutio>` + `ProgramData` constraints) | `h1_only_the_upgrade_authority_can_initialize_the_registry` |
| M-1 | `set_eligibility_verifier`, `set_registry_authority` (old and new key both sign), `set_agency_active`; every agency-signed instruction now checks `active` | `m1_the_eligibility_verifier_can_be_replaced`, `m1_registry_authority_rotation_needs_both_keys_and_moves_power`, `m1_a_revoked_agency_cannot_sign_anything` |
| M-2 | `AdhesionRequest` stores `unit_price` and `obligated_amount`; obligations citing it stay within `authorized_qty × unit_price` | `m2_obligations_cannot_exceed_the_value_of_their_source_adhesion` |
| M-3 | `formalize_adhesion` takes the record and refuses unless it is active. Legal reading: executing an authorized adhesion is a new contract, which art. 28 §1 forbids during suspension; the 90-day clock keeps running and the manager may extend after reactivation | `m3_no_execution_while_the_record_is_suspended` |
| L-1 | Kept `init_if_needed` for the usage account; the exception and its guard are recorded in `DECISIONS.md` | existing Carona tests |
| L-2 | `expire_adhesion` also lapses requests pending for more than 90 days (`RESPONSE_WINDOW_SECS`, a design choice recorded in `LEGAL_RULES.md`) | `l2_unanswered_requests_lapse_after_the_response_window` |
| L-3 | `add_item` requires an active record in force; `extend_execution` requires an active record | `l3_no_items_on_a_cancelled_record`, `m3_…` |
| L-4 | Open: needs an issuer attestation for fiscal documents. Recorded as a known limitation | — |
| L-5 | `NameTooLong`, appended at the end of the error enum | `l5_long_agency_names_get_their_own_error` |
| Info 2, 3 | `checked_sub`; `debug_assert` removed | clippy |
| Info 7 | `close_request` (rejected or lapsed), `close_financing` and `close_obligation` (settled), rent back to the stored `rent_payer`; fiscal documents and executed adhesions are never closed | `closing_returns_rent_only_for_terminal_records`, `a_rejected_request_can_be_closed_by_anyone` |
| Negative coverage | Tests now hit `AccountMismatch`, `AgencyInactive`, `InvalidValidity`, `InvalidAmount` | `an_item_from_another_record_is_rejected_as_a_mismatch`, `invalid_validity_and_zero_amounts_are_refused` |
| fmt | `rustfmt.toml` with `max_width = 120`; `cargo fmt --check` passes | — |
| Anchor.toml | `[programs.devnet]` added | — |

Still open: verifiable build, CI, fuzzing, emergency pause, `emit_cpi!`, upgrade-authority transfer, L-4, and `Overflow` coverage (unreachable with realistic inputs).
