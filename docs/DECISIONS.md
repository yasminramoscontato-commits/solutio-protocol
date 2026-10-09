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

## Pending

- [ ] Devnet deployment (requires explicit authorization from the founder)
- [ ] Concurrency demo on a real validator (two simultaneous acceptances for the last units)
- [ ] Public verifier page that reads state directly from the chain
- [ ] Validation interviews with procurement officers and financiers
- [ ] Optional: financing pool with a test stablecoin
