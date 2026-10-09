# Decision log

| Date | Decision | Reason |
| --- | --- | --- |
| 2026-10-09 | Build Solutio, not a clinical chain-of-custody system | Multi-institution problem with a numeric legal rule no single party controls; data is public by law; founder works inside public procurement workflows |
| 2026-10-09 | Two modules — Carona (art. 86 caps) and Obligations (single financing) — sharing the invariant pattern but not the same data model | Different legal rules and different signers; avoids an over-generic abstraction |
| 2026-10-09 | All legal and accounting rules live in `rules.rs` as pure functions | Exhaustive and randomized testing without a VM; one-to-one traceability to legal sources |
| 2026-10-09 | ~~Acceptance by the supplier is the authoritative cap check~~ (superseded below) | First version |
| 2026-10-09 | Flow follows Decree 11.462/2023 art. 31 §1: request → supplier accepts → manager authorizes | The decree requires the supplier's acceptance before the manager's authorization; the first version had them reversed |
| 2026-10-09 | Caps are checked when the request reserves quantity; every later step can only keep or release it | Mirrors the federal tool's balance formula (pending + authorized count); the reserving transaction writes the item account, so competing requests are serialized |
| 2026-10-09 | Partial authorization and denial require a justification hash | Mirrors "aceitar parcialmente / negar" in the federal tool |
| 2026-10-09 | Exceptions to §5 are claimed per adhesion, not set per item | Decree art. 32 §§1–2 ties them to the purpose of the purchase and to who manages the record |
| 2026-10-09 | 90-day execution window, manager extension, permissionless lapse | Decree art. 31 §§2–3; lapsing needs no trusted party, so balances cannot be held hostage |
| 2026-10-09 | Record status: Active, Suspended, Cancelled (final) | Decree arts. 28–29 |
| 2026-10-09 | Product insight: in Compras.gov.br the supplier's acceptance is an uploaded document; in Solutio it is the supplier's signature | Removes a manual verification step and a forgery vector |
| 2026-10-09 | `verified` ≠ `eligible` ≠ `financed` | Liquidation recognizes a debt; it does not prove the claim is assignable, unencumbered or financeable |
| 2026-10-09 | Reductions are always recorded and can mark an obligation Impaired | A disallowance is a government act; the protocol must reflect reality, not refuse it |
| 2026-10-09 | Financing closes after the first payment (MVP) | Keeps the balance model simple and defensible until payment routing to financiers is modeled |
| 2026-10-09 | Sponsored fee payer: agency wallets hold no SOL | Public servants should never need to buy crypto to sign |
| 2026-10-09 | Demo identity issuer instead of Solana Attestation Service in the MVP | Time; SAS integration is the first production step |
| 2026-10-09 | Build target SBPF v2 (`anchor build --arch v2`) | LiteSVM 0.10 (test runtime) does not load SBPF v3; devnet compatibility to be confirmed at deploy time |
| 2026-10-09 | Monetary values in integer cents (u64); no floating point | Exact arithmetic for public money |
| 2026-10-09 | No token as business model | Revenue must come from verification and financing volume, not from a speculative asset |
| 2026-10-09 | Apache-2.0 license | Permissive, with an explicit patent grant |
| 2026-10-09 | Fix: the federal-programme exception requires a federally managed record | Law art. 86 §6 lifts §5 only for records of the federal Executive; the first version accepted it for any manager |
| 2026-10-09 | Rule profiles per agency (`Baseline`, `Alagoas`), attested by the registry | Each sphere regulates art. 86 for itself; the adherent's regulation decides whom it may adhere to. Alagoas art. 33 is the first rule that differs from the federal one |
| 2026-10-09 | Deployed to devnet (program `5cmBDMRdqAfrhmkMmLVxTBCNBHxh5sneyvWXJViyjz9E`), authorized by the founder | SBPF v2 is enabled on devnet; the deployed bytes match the tested binary. 32-step scripted run recorded in `docs/DEVNET.md` |
| 2026-10-09 | Refused steps in the demo are sent with preflight disabled | The program's rejection is then recorded on-chain and visible in the explorer, which is the evidence judges need |

| 2026-10-09 | Priority order: evidenced gaps → business rules → framework, before more program features | Founder's direction: the MVP serves the thesis, not the reverse |
| 2026-10-09 | Field knowledge enters the repository only as anonymized flows and dispatch templates | Internal documents stay private; the founder's operational knowledge is presented as such |
| 2026-10-09 | Framework in dependency-free Python, with rules and evidence as data | Anyone can rerun the replays and economics; rules can be reviewed by lawyers without reading Rust |
| 2026-10-09 | Government pays nothing; revenue from financiers (verification fee, take rate via licensed partner) | Removes the procurement barrier to adoption; the party that bears double-financing risk pays |
| 2026-10-09 | Finding: rent is ~99% of on-chain cost per obligation | Measured on devnet. A close instruction for settled accounts becomes a requirement for scale |

## Pending

- [x] Devnet deployment (authorized 2026-10-09)
- [ ] Transfer the program's upgrade authority from the session's demo key to a wallet controlled by the founder
- [ ] Restrict `init_registry` to the program's upgrade authority before any non-demo use (today the first caller becomes the registry authority; on devnet we initialized it right after deploying)
- [ ] Validate the Alagoas profile and the questions in LEGAL_RULES.md with GARC/SEI staff
- [ ] Program: participant quotas, supply authorization and rectification (specified as PROC-11/12 in the framework)
- [ ] Program: close settled accounts to recover rent (economics finding)
- [ ] Interviews: two receivables funds (FIDCs) and one factoring company on willingness to pay (H5, H8)
- [ ] Concurrency demo on a real validator (two simultaneous acceptances for the last units)
- [ ] Public verifier page that reads state directly from the chain
- [ ] Validation interviews with procurement officers and financiers
- [ ] Optional: financing pool with a test stablecoin
