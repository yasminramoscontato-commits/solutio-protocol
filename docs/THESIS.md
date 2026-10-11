# Thesis and validation status

**Thesis.** Public bodies, suppliers and financiers share limits that no single party controls: how much of a price record can be adhered to, and how much of a payment obligation can be financed. Because the shared state is split across systems, the limits are checked after the fact and break in predictable ways.

A public program that holds this state, and enforces the limits as rules per jurisdiction, makes those failures unreachable. Financiers pay for a receivable they can verify, so the public layer can be free for government.

Each hypothesis below has a status:
- **Validated**: public evidence or a working artifact supports it.
- **Partial**: some evidence, with a stated gap.
- **Not validated**: no evidence yet, with the next step stated.

Evidence ids refer to [`framework/EVIDENCE.md`](framework/EVIDENCE.md). Scenario ids refer to [`framework/SCENARIOS.md`](framework/SCENARIOS.md).

| # | Hypothesis | Status | Evidence | Gap / next step |
| --- | --- | --- | --- | --- |
| H1 | Statutory adhesion caps break in practice | **Validated** | `BR-TCU-547-2026` (negative balances, rounding up, authorization after expiry); `BR-TCU-1487-2007`; scenarios S02–S04 reproduce the outcomes and show Marjan refusing them | How often it happens across all records is unknown: the TCU itself could not extract reliable balances |
| H2 | No shared state exists across systems and spheres | **Validated** | 179 systems run price registration (`BR-TCU-547-2026`); the national procurement portal (PNCP) does not count adherents' contracts; field observation of 3–4 parallel records per adhesion (`FIELD-STATE-CPB`) | Interviews to measure reconciliation effort (hours per adhesion) |
| H3 | The same structural rule exists outside Brazil | **Validated (legal)** / **Partial (market)** | `EU-CJEU-C216-17`, `EU-CJEU-C23-20`, `US-GAO-05-201`; scenario S08 runs the EU rule on the same engine | No evidence yet of foreign buyers' demand; next step is a conversation with one EU central purchasing body |
| H4 | Financing the same asset twice is a real loss mode, and government receivables lack the registry that private receivables have | **Validated** | `GL-QINGDAO-2014` (conviction); `GL-FIRSTBRANDS-2025` and `GL-TRICOLOR-2025` (allegations); `BR-BCB-RECEIVABLES` vs `BR-ANTECIPAGOV`; scenario S09 | Whether electronic duplicatas could legally carry government receivables is unverified |
| H5 | Late payment harms suppliers, and verified obligations make financing possible | **Partial** | `MU-CNM-2025`, `BR-TESOURO-RAP-2026` (R$ 109bn delivered and verified, still unpaid), `EU-LATE-PAYMENT-BANKRUPTCIES`, `GL-ITALY-PCC` (correlation, not causation) | Financiers' willingness to pay has not been tested; next step is interviews with two receivables funds (FIDCs) and one factoring company |
| H6 | Rules can be enforced as code, with one profile per jurisdiction | **Validated (technical)** | 51 program tests; 22 framework tests; 12 scenario replays across the federal, Alagoas and EU profiles; program live on devnet with on-chain refusals and a parallel-submission concurrency test (`docs/DEVNET.md`); security audit with high and medium findings fixed | Legal validation of the rulebook by the GARC team and a procurement lawyer |
| H7 | Solana is economically viable at public-sector scale | **Partial** | Measured on devnet: 95,000 lamports in fees and about 8.1M lamports of rent deposits per adhesion plus financed obligation; implemented closing returns about 44% of the gross cost; national scenario under 1 transaction per second on average (`docs/framework/ECONOMICS.md`) | With the assumed prices, on-chain cost is still 16–25% of revenue after closing. Next lever: record obligations on-chain only when financing is requested, and shrink the permanent marker accounts |
| H8 | Governments adopt a free compliance layer, and financiers pay for verification | **Not validated** | — | Pilot proposal to one state central purchasing body; pricing interviews with financiers |

## Why a public chain and not a better central system

The TCU's recommendations for the federal adhesion module (items 9.1.1.6 to 9.1.1.12 and 9.1.3.2) describe what Marjan does by construction:
- round down;
- block adhesions when no quantity is available;
- block authorization after expiry;
- keep one system of record;
- make execution traceable;
- publish balances.

A well-run central system could also do this. Two facts argue for a shared public state instead:
1. **No single operator.** Price registration runs on 179 systems across federal, state and municipal spheres, and records are adhered to across state lines (`AL-ATA-USED-OUT-OF-STATE`). No single operator is accepted by all of them.
2. **Different readers.** The parties who must verify the balance are auditors, citizens and competing financiers. They need to read it without permission and without trusting any one interface.

The honest counter-argument stays in the README: if one operator were trusted by all parties, a database would do.

## What the evidence does not show

- That Marjan would have prevented any specific case cited. The cases show the failure modes; the scenarios show the rules refusing them.
- Any pilot, partnership, user or transaction with real public money. Everything on devnet uses DEMO identities.
- That the framework's economics are a forecast. They are scenarios built on stated assumptions and on costs measured on devnet.
