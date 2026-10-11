# Marjan

**From public obligation to payment — verifiably.**

Marjan is an open Solana protocol for public procurement obligations. It enforces statutory caps on shared government contracts, starting with Brazil's adhesion limits (Law 14.133/2021, art. 86, and Decree 11.462/2023, arts. 31–33), and records each government payment obligation through explicit, evidence-backed states. Within the protocol, a cap cannot be exceeded and an obligation's balance cannot be financed twice, even under concurrent requests. Anyone can verify that state on-chain without trusting our interface.

Built for the Colosseum **Crypto World's Fair** hackathon (Solana track).

---

## Status (what is real today)

| Component | Status |
| --- | --- |
| Anchor program (`programs/marjan`) | Implemented, builds for SBF |
| Legal rules as pure functions (`rules.rs`) | Implemented, 13 unit tests incl. 2 randomized suites (2,000 sequences each) |
| Security audit (Solana AI Kit checklist) | [`docs/security-audit-2026-10-09.md`](docs/security-audit-2026-10-09.md): 0 critical; high and medium findings fixed, 13 regression tests |
| Module 1 — Carona (art. 86 + Decree 11.462/2023 + Alagoas profile) | Implemented, 17 integration tests |
| Module 2 — Obligations (single financing) | Implemented, 8 integration tests |
| Integration test environment | LiteSVM (in-process Solana VM running the compiled program) |
| Devnet deployment | **Live**, audited binary — [`5cmBDMRd…jz9E`](https://explorer.solana.com/address/5cmBDMRdqAfrhmkMmLVxTBCNBHxh5sneyvWXJViyjz9E?cluster=devnet); scripted scenario and parallel-submission concurrency test in [`docs/DEVNET.md`](docs/DEVNET.md) |
| TypeScript client + IDL (`client/`) | Demo script used for the devnet run; cost measurement script |
| Framework (`framework/`) | Evidence base, rulebook (24 rules across Brazil, Alagoas and the EU), reference engine, 12 scenario replays, unit economics from measured devnet costs; 22 tests |
| Public verifier page | **Not yet** |
| Financing pool (test stablecoin) | **Not started** — experimental, optional |

Last full test run: 51 passed, 0 failed (2026-10-09). Devnet uses test SOL and DEMO identities with fictional names. No real funds, no real government data, no pilot or partnership is claimed.

---

## Start here

- [`docs/PROBLEM_CHAIN.md`](docs/PROBLEM_CHAIN.md): the problem from global to municipal, with every claim sourced.
- [`docs/THESIS.md`](docs/THESIS.md): hypotheses and what is validated, partial or not yet validated.
- [`docs/framework/RULEBOOK.md`](docs/framework/RULEBOOK.md): business rules and their legal sources.
- [`docs/framework/SCENARIOS.md`](docs/framework/SCENARIOS.md): documented cases replayed through the rules.
- [`docs/framework/ECONOMICS.md`](docs/framework/ECONOMICS.md): measured costs and revenue scenarios.
- [`docs/field/OPERATIONAL_FLOWS.md`](docs/field/OPERATIONAL_FLOWS.md): how a state central purchasing body works today (anonymized).

## The problem

Governments are among the largest buyers and slowest payers. Brazilian public procurement averaged **12.5% of GDP in 2006–2016** ([IPEA, TD 2476](https://repositorio.ipea.gov.br/bitstream/11058/9315/2/td_2476_sumex.pdf)). In a December 2025 survey by the National Confederation of Municipalities (CNM), **1,202 of the 4,172 responding municipalities (28.8%)** reported overdue payments to suppliers ([Brasil 61](https://brasil61.com/n/cerca-de-um-terco-das-prefeituras-estao-com-o-pagamento-de-fornecedores-atrasado-bras2515415)).

Two rules decide whether a public claim is sound, and both are hard to verify across institutions today:

1. **Was the contract lawful?** Brazil lets non-participating agencies "ride" (*carona*) on another agency's price-registration record, but caps each rider at 50% of the registered quantity and all riders together at 2x (art. 86, §§4–5). The sum depends on decisions made by independent agencies, often at different levels of government, in systems that do not talk to each other.
2. **Has the claim already been financed?** Financing the same receivable twice is the classic factoring fraud. A lender must confirm the claim is valid, unpaid and not already pledged.

Similar state-run answers exist — Italy's PCC (2012), India's TReDS, Brazil's AntecipaGov (2021). Each runs inside one country's systems, for regulated financiers. Brazil's federal *Gestão de Atas* tool already controls adhesion balances for records it hosts, but access requires a government login, the supplier's acceptance is an uploaded document, and records managed in state or municipal systems stay outside it. Marjan explores a complementary layer: rules and balances that any party can verify, with acceptance by signature, configurable per jurisdiction.

---

## What the program guarantees — and what it does not

**Guaranteed by the program, over its own recorded state:**

- Adhesions follow the order of Decree 11.462/2023, art. 31: the agency requests, the **supplier accepts by signature**, and only then the managing agency authorizes, fully or partially with a recorded justification.
- A request reserves quantity at once, as the federal *Gestão de Atas* tool does. No sequence of requests, including competing ones, exceeds §4 (50% per agency) or §5 (2x per item, or the lower maximum set by the tender).
- Statutory exceptions (Ministry of Health emergencies, federal-programme transfers) lift §5 only when they apply, and never lift §4.
- Federal agencies cannot adhere to state, district or municipal records (§8). Expired, suspended or cancelled records accept nothing.
- Rules that a state adds on top of the statute are enforced per agency profile. First profile: **Alagoas** (Decree 95.019/2023), whose state agencies cannot adhere to municipal records other than those of state capitals (art. 33).
- An authorized adhesion must be executed within 90 days (extendable by the manager, never past validity); after that **anyone** can lapse it and its quantity returns to the pool.
- An obligation is financeable only after a **separate** eligibility confirmation by a designated verifier, backed by attached fiscal documents. Verified (liquidated) ≠ eligible.
- The cumulative financed amount never exceeds the eligible amount. A fiscal document can back only one obligation.
- Financing requires the creditor's signature and evidence that the debtor was notified of the assignment.
- Reductions (withholdings, disallowances) are recorded even when they hurt a financier, and mark the obligation **Impaired** for everyone to see.
- Agency wallets never need SOL: a sponsor pays fees and rent (tested).

**Not guaranteed — depends on off-chain evidence and processes:**

- That a document is authentic, that goods were delivered, or that the debtor will pay.
- That a claim was not assigned *outside* Marjan. The protocol reduces duplication risk; it cannot eliminate operations it does not see.
- Legal eligibility for assignment itself. The program records who confirmed it and the evidence hash; the legal judgment is human.

---

## Architecture

```mermaid
flowchart LR
  subgraph M1[Module 1 · Carona]
    R[Agency requests] --> C{Caps §4 · §5 · §8<br/>status · validity}
    C -- no --> X[Refused · balance intact]
    C -- ok, quantity reserved --> S[Supplier accepts]
    S --> A[Manager authorizes<br/>full or partial]
    A --> E[Executed within 90 days]
    A -. deadline missed .-> L[Lapsed · quantity released]
  end
  subgraph M2[Module 2 · Obligations]
    V[Verified<br/>debtor recognizes debt] --> D[Fiscal documents attached]
    D --> L[Eligible<br/>signed by verifier]
    L --> F[Financed<br/>financier + creditor sign]
    F --> P[Paid / Settled]
    F -. reduction .-> I[Impaired]
  end
  E -. optional source .-> V
```

- `src/rules.rs` — every legal and accounting rule as a pure function with no Solana dependency. Handlers only load accounts, call these functions and persist results.
- `src/instructions/carona.rs` — price records, items, reservation, supplier response, authorization, execution, lapse and record status.
- `src/instructions/obligation.rs` — obligations, fiscal documents, eligibility, financing, reductions, payments.
- `src/events.rs` — every transition emits an event, so history can be rebuilt from the ledger alone.

The legal/documentary layer is separated from the financial act: eligibility and financing are different instructions with different signers.

See [`docs/LEGAL_RULES.md`](docs/LEGAL_RULES.md) for the rule → code → test traceability matrix and [`docs/DECISIONS.md`](docs/DECISIONS.md) for the decision log.

---

## Why Solana

Solana is the shared state between parties with no common operator: agencies at different levels of government, suppliers, financiers and auditors.

- **The program, not a UI, enforces the rules.** A transaction that would breach a cap fails on-chain.
- **Account-level write locking serializes conflicting writes.** Requests for the same item, or financings of the same obligation, are processed one after another, and each re-checks current state. Tested on devnet by submitting six requests at once for a pool that fits four: four landed, two were refused, the item ended at exactly 200 (`docs/DEVNET.md`).
- **Program-derived addresses** let anyone locate a record from public identifiers (price record, item, agency, obligation, document key).
- **Fee-payer separation** lets officials sign without holding SOL (`agency_wallets_sign_without_holding_any_sol`).

Honest caveat: a well-governed shared database run by a single trusted operator could enforce the same rules. Marjan's bet is that, across independent levels of government and competing financiers, nobody is accepted as that operator, and verification must be open to anyone without permission.

---

## Run it

Toolchain used: Rust 1.89 (pinned in `rust-toolchain.toml`), Solana CLI 4.3.0 (Agave), Anchor CLI 1.2.1, platform-tools v1.57.

```bash
# Build the program (SBPF v2: the version supported by the LiteSVM test runtime)
anchor build --no-idl --arch v2

# Unit + integration tests (LiteSVM runs the compiled target/deploy/marjan.so)
cargo test -p marjan

# Replay the scripted scenario against the devnet deployment (needs a funded devnet keypair)
cd client && npm install && node devnet-demo.mjs
```

---

## Known limitations

- Identity: agencies are registered by a labeled **demo issuer**. Production would use credentials from an accountable authority (e.g. Solana Attestation Service issued by an audit court or state government) and multisig wallets per agency.
- Exceptions: the registry attests which agency is the Ministry of Health; the legal judgment that a purchase is an emergency or executes a federal programme stays human (evidence hash recorded).
- Participating agencies (other than the manager) and quantity reallocation (*remanejamento*, Decree art. 30) are not modeled yet.
- Decree 11.462/2023 is the federal regulation; Alagoas is the first state profile. Other state and municipal regulations become profiles once validated.
- After the debtor's first payment, new financing is closed (simplifying policy for the MVP).
- Open legal interpretation questions are listed in `docs/LEGAL_RULES.md`.

## Research

Marjan is also the artifact of a Design Science Research study on *rules as code* in public procurement: which statutory requirements can be enforced by a program, which require recorded human judgment, and whether shared on-chain state makes compliance more verifiable across institutions.

## License

Apache-2.0. See [LICENSE](LICENSE).
