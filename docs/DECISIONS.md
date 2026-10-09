# Decision log

| Date | Decision | Reason |
| --- | --- | --- |
| 2026-10-09 | Build Solutio, not a clinical chain-of-custody system | Multi-institution problem with a numeric legal rule no single party controls; data is public by law; founder works inside public procurement workflows |
| 2026-10-09 | Two modules — Carona (art. 86 caps) and Obligations (single financing) — sharing the invariant pattern but not the same data model | Different legal rules and different signers; avoids an over-generic abstraction |
| 2026-10-09 | All legal and accounting rules live in `rules.rs` as pure functions | Exhaustive and randomized testing without a VM; one-to-one traceability to legal sources |
| 2026-10-09 | Acceptance by the supplier is the authoritative cap check | It is the transaction that writes the item totals, so concurrent acceptances are serialized by Solana's account locks |
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
