# The problem chain, from global to municipal

The problem is the same at every level: **independent public buyers share one limit, and nobody holds the shared state**. Two limits matter:

- a **quantity limit** on shared purchasing instruments (Brazilian price records, EU framework agreements, US IDIQ contracts);
- a **money limit** on what can be financed against a public payment obligation.

When no one holds the shared state, limits are checked after the fact. The checks then fail in the same few ways: balances below zero, rounding in the wrong direction, authorizations after expiry, and the same receivable pledged twice.

Every claim below carries an evidence id from [`framework/EVIDENCE.md`](framework/EVIDENCE.md), where its status and source are listed. Allegations are marked as such.

---

## 1. Global: public money is huge, slow, and financed blind

- **Scale.** Governments spend about **US$11 trillion a year**, about 12% of world GDP (`GL-WB-PROCUREMENT`). Factoring, the financing of receivables, moves **EUR 4.0 trillion** a year (`GL-FCI-FACTORING`).
- **Slow payment is a recognised harm.**
  - The EU caps public payment terms at 30 days, or 60 for public healthcare (`EU-DIR-2011-7`).
  - The Court of Justice condemned Italy for breaching that cap (`EU-CJEU-C122-18`).
  - The Commission attributes **1 in 4 EU bankruptcies** to late payment (`EU-LATE-PAYMENT-BANKRUPTCIES`).
  - In India, small firms have **INR 55,244 crore** in delayed-payment claims on a government portal (`GL-INDIA-SAMADHAAN`).
- **Shared registries correlate with faster payment.** Italy's average public payment time fell from about **120–130 days (2013) to 30 days (2024)** after it built a central invoice platform (`GL-ITALY-PCC`). This is a correlation, not proof of cause.
- **One asset, many financings.** This is a recurring way to lose money:
  - **Qingdao, 2014:** duplicated warehouse receipts raised over **US$1.7bn** in bank loans. The founder was convicted and sentenced to 23 years (`GL-QINGDAO-2014`).
  - **First Brands and Tricolor, 2025:** prosecutors allege double-pledged invoices and collateral worth billions. These are allegations under indictment (`GL-FIRSTBRANDS-2025`, `GL-TRICOLOR-2025`).
  - **Greensill:** the Swiss regulator found the bank could not check whether the receivables it financed were actually owed (`GL-GREENSILL-FINMA`).
- **Earlier ledger pilots did not enforce limits.** Peru recorded 154,400 purchase orders on a blockchain for tamper-evidence. None of the pilots enforced a statutory cap or a single financing per receivable (`GL-PILOTS-LEDGER`).

## 2. Same rule, different law

| Jurisdiction | Instrument | Shared limit |
| --- | --- | --- |
| European Union | Framework agreement | A maximum must be stated, may be split among named authorities, and the agreement ends when it is reached (`EU-CJEU-C216-17`, `EU-CJEU-C23-20`) |
| United States | IDIQ / interagency contracts | Orders may not exceed the stated maximum. In one GAO audit, 10 of 11 orders on a shared contract were out of scope (`US-GAO-05-201`) |
| Brazil | Ata de registro de preços (price record) | 50% per adherent and 2x in total (Law 14.133, art. 86) |

Marjan's reference engine runs the Brazilian and EU rules on the same core (scenario S08 in [`framework/SCENARIOS.md`](framework/SCENARIOS.md)).

## 3. Brazil: the rules exist, the shared state does not

- **Scale.** Public procurement averaged **12.5% of GDP** (`BR-IPEA-PROCUREMENT`). From 2023 to mid-2025, price-registration procedures moved about **R$ 970.9bn** in 301.5k procurements, run on **179 different systems**. The federal systems covered R$ 337.8bn of that (`BR-TCU-547-2026`).
- **The caps break inside the official tool.** Brazil's Federal Court of Accounts (TCU) audited the federal adhesion module in 2026 (`BR-TCU-547-2026`). It found:
  - balances shown below zero (**−2, −155, −25**);
  - the 50% cap **rounded up**;
  - authorizations accepted **after the record expired**;
  - no reliable remaining balances.

  Its recommendations amount to a specification: round down, block when nothing is available, enforce validity, keep every adhesion in one system of record, and publish balances. The national courts-of-accounts associations ask managing bodies for the same thing (`BR-ATRICON-NOTA-01-2025`).
- **History repeats.** In 2007, 62 bodies adhered to one R$ 32M record, a theoretical exposure of about R$ 2bn (`BR-TCU-1487-2007`).
- **Payment stress.**
  - The federal government carries **R$ 391.5bn** of unpaid committed spending into 2026, of which **R$ 109bn** is for goods and services already delivered and verified (`BR-TESOURO-RAP-2026`).
  - The law fixes a chronological payment order (`BR-LAW-14133-ART141`).
  - It also lets a supplier suspend work after two months of delay (`BR-LAW-14133-ART137`).
- **The anti-double-financing fix covers private receivables only.** Card receivables and electronic duplicatas have registries built to stop the same receivable being traded twice (`BR-BCB-RECEIVABLES`).
  - Government contract receivables are not covered by either registry.
  - The federal advance programme locks a single bank and caps advances at 70% of the remaining balance. It runs only inside the federal portal and publishes no volumes (`BR-ANTECIPAGOV`).

## 4. State: Alagoas

- **Its own rules.** Alagoas regulates price registration itself, with a rule the federal decree does not have: its state bodies may adhere to municipal records only of state capitals (`AL-DECREE-95019`, art. 33). It also mandates the federal tool, whose defects TCU documented (art. 24).
- **Records cross state lines.** An Alagoas record was adhered to by a police force in another state (`AL-ATA-USED-OUT-OF-STATE`). The balance of one record is therefore shared across federal units.
- **Delays reach essential services.** Federal prosecutors demanded that the state regularize late transfers to an oncology hospital serving 47 municipalities (`AL-MPF-CHAMA`). These are health transfers, not ordinary purchases.
- **Field observation** (`FIELD-STATE-CPB`, anonymized):
  - one adhesion is recorded in the electronic process, in Compras.gov.br, and in an internal control system or spreadsheets;
  - the supplier's acceptance arrives by e-mail and is attached as a PDF;
  - one adhesion took **51 days** from request to authorization;
  - authorizations are rectified after budget review.

## 5. Municipal: where the caps break and suppliers wait

- **Adhesion is mainstream.** **52.6%** of São Paulo's 643 municipalities adhere to other entities' records. **22.5%** of them do so without a study showing the adhesion was advantageous (`MU-TCESP-IRALC`).
- **One record, many cities.**
  - Several Alagoas and Pernambuco municipalities bought robotics kits through the same few municipal records. The TCU suspended new federal transfers for them (`MU-TCU-ROBOTICS`).
  - In Rio Grande do Sul, one record was reused by at least 10 cities, at R$ 32k per unit against an audit estimate of about R$ 10k (`MU-TCERS-SCREENS`).
  - The São Paulo court of accounts suspended R$ 7.7M paid through an out-of-state consortium record (`MU-TCESP-VOTORANTIM`).
- **Suppliers wait.** **28.8%** of responding municipalities were behind on supplier payments (`MU-CNM-2025`).

---

## What Marjan changes at each level

| Level | Failure | Marjan |
| --- | --- | --- |
| Global | The same receivable is financed twice | One eligible balance per obligation; one document per obligation (PAY-04) |
| Global | Late payment is invisible until it becomes a crisis | The age of every verified obligation is public; legal thresholds are flagged (PAY-03, PAY-07) |
| EU / US / Brazil | A shared cap is checked after the fact | The cap is checked when quantity is reserved; when it is exhausted, the instrument stops (PROC-02, EU-01) |
| Brazil | Balances below zero, rounding up, authorization after expiry | Unreachable by construction (PROC-01, 02, 03, 05) |
| Brazil | 179 systems and no shared balance | One public state that any system can read and write through the same rules |
| State | Its own rules on top of federal law | Rule profiles per regulation (PROC-09) |
| Municipal | Many cities on one record, with no visible balance | Per-item and per-object balances, public to every buyer, auditor and citizen (PROC-14, PROC-15) |
