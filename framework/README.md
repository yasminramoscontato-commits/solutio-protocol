# Marjan framework

The framework turns what Marjan knows into data and runnable scripts: the problem, the rules, and the economics. The Solana program (`programs/marjan`) enforces the subset of rules that are on-chain today; the framework holds all of them and is where new rules are specified before they move into the program.

| File | Contents |
| --- | --- |
| `marjan_framework/evidence.json` | Evidence base, from global to municipal: every claim with its status (law, court, official data, audit finding, estimate, survey, allegation, field observation), date and source |
| `marjan_framework/rulebook.json` | Business rules (PROC-xx for shared purchasing, EU-xx for EU framework agreements, PAY-xx for payment obligations), each with its legal and institutional source, how it shows up in practice, and where it is enforced |
| `marjan_framework/engine.py` | Reference engine for price records and EU framework agreements, under two control modes: STRICT (Marjan) and LEGACY (reproduces the outcomes documented by TCU Acórdão 547/2026) |
| `marjan_framework/payments.py` | Single financing per receivable, chronological payment queue (art. 141), overdue flags (BR/EU), AntecipaGov profile |
| `marjan_framework/scenarios.py` | 12 replays of documented cases through the rules |
| `marjan_framework/economics.py` + `assumptions.json` | Unit economics from on-chain costs measured on devnet, and scenarios from pilot to global |

## Run

No dependencies beyond Python 3.10+.

```bash
cd framework
python -m unittest discover -s tests -t .   # 22 tests
python -m marjan_framework                  # regenerates docs/framework/*.md
```

Generated reports: [`RULEBOOK.md`](../docs/framework/RULEBOOK.md), [`EVIDENCE.md`](../docs/framework/EVIDENCE.md), [`SCENARIOS.md`](../docs/framework/SCENARIOS.md), [`ECONOMICS.md`](../docs/framework/ECONOMICS.md).

To refresh the measured on-chain costs after a new devnet run: `cd client && node devnet-demo.mjs && node measure-costs.mjs`.

## Adding a jurisdiction

1. Add its rules to `rulebook.json` with sources.
2. Express the differences as a profile in `engine.py` (Alagoas art. 33 is the template: one condition keyed on the adherent's profile).
3. Add a scenario that replays a documented case and a test that asserts its outcome.
4. When the rule is stable, implement it in `programs/marjan/src/rules.rs` and add an integration test.

## Limits

- The LEGACY mode reproduces the **outcomes** TCU documented. It does not claim to reproduce the federal system's internal logic.
- Scenario quantities are normalized unless the source gives figures.
- Economic scenarios are not forecasts: every input that is not tied to an evidence id is labeled as an assumption.
