"""Tests for the Solutio framework. Run: python -m unittest discover -s framework/tests"""

import json
import random
import re
import unittest
from datetime import date
from pathlib import Path

from solutio_framework import economics, scenarios
from solutio_framework.engine import (
    DAY, LEGACY, STRICT, Body, Exception_, FrameworkAgreement, PriceRecordItem, Sphere,
)
from solutio_framework.payments import (
    AntecipaGovContract, Art141Exception, Category, Claim, PaymentQueue, Receivable, ReceivableRegistry, overdue_flags,
)

PKG = Path(scenarios.__file__).resolve().parent
T0 = 1_000 * DAY


def item(registered=100, manager=None, mode=STRICT, **kw):
    manager = manager or Body("m", Sphere.STATE)
    return PriceRecordItem(manager, {manager.id: registered}, T0 - DAY, T0 + 365 * DAY, mode=mode, **kw)


class KnowledgeBaseIntegrity(unittest.TestCase):
    def setUp(self):
        self.rules = {r["id"]: r for r in json.loads((PKG / "rulebook.json").read_text())["rules"]}
        self.evidence = {e["id"]: e for e in json.loads((PKG / "evidence.json").read_text())["items"]}

    def test_every_rule_has_a_source_and_valid_enforcement(self):
        for r in self.rules.values():
            self.assertTrue(r["sources"], r["id"])
            self.assertIn(r["enforcement"], {"program", "framework", "advisory", "human"})

    def test_every_evidence_item_is_sourced_and_labeled(self):
        statuses = set(json.loads((PKG / "evidence.json").read_text())["status_legend"])
        for e in self.evidence.values():
            self.assertIn(e["status"], statuses, e["id"])
            self.assertRegex(e["date"], r"^\d{4}-\d{2}-\d{2}$")
            if e["status"] != "field-observation":
                self.assertTrue(e["url"].startswith("https://"), e["id"])

    def test_scenarios_cite_existing_evidence_and_rules(self):
        for s in scenarios.run_all():
            for ev in s.evidence:
                self.assertIn(ev, self.evidence, s.id)
            for st in s.steps:
                for d in (st.legacy, st.solutio):
                    if d is not None and d.rule:
                        self.assertIn(d.rule, self.rules, f"{s.id}: {d.rule}")

    def test_field_evidence_contains_no_personal_or_process_identifiers(self):
        text = json.dumps(self.evidence["FIELD-STATE-CPB"])
        self.assertIsNone(re.search(r"\d{5}\.\d{10}/\d{4}", text))      # SEI process numbers
        self.assertIsNone(re.search(r"\d{2}\.\d{3}\.\d{3}/\d{4}-\d{2}", text))  # CNPJ


class Procurement(unittest.TestCase):
    def test_strict_rounds_down_and_legacy_rounds_up(self):
        self.assertEqual(item(21).individual_cap(), 10)
        self.assertEqual(item(21, mode=LEGACY).individual_cap(), 11)

    def test_strict_balance_never_negative_under_random_sequences(self):
        rng = random.Random(7)
        for _ in range(300):
            it = item(rng.randint(1, 300))
            bodies = [Body(f"b{i}", Sphere.MUNICIPAL) for i in range(rng.randint(1, 10))]
            live = []
            for _ in range(40):
                d, a = it.request(rng.choice(bodies), rng.randint(1, it.registered), T0)
                if d.ok:
                    it.supplier_accept(a)
                    it.authorize(a, rng.randint(1, a.qty), T0)
                    live.append(a)
                self.assertGreaterEqual(it.available_for_adhesion(), 0)
                for b in bodies:
                    self.assertLessEqual(it.body_committed(b), it.individual_cap())

    def test_legacy_can_go_negative(self):
        s = scenarios.s03_negative_balance()
        self.assertLess(s.metrics[LEGACY.name]["available_for_adhesion"], 0)
        self.assertEqual(s.metrics[STRICT.name]["available_for_adhesion"], 0)

    def test_supplier_must_accept_first_and_expiry_blocks_authorization(self):
        it = item()
        d, a = it.request(Body("x", Sphere.MUNICIPAL), 10, T0)
        self.assertEqual(it.authorize(a, 10, T0).rule, "PROC-04")
        it.supplier_accept(a)
        self.assertEqual(it.authorize(a, 10, T0 + 400 * DAY).rule, "PROC-05")

    def test_lapse_releases_quantity(self):
        it = item()
        d, a = it.request(Body("x", Sphere.MUNICIPAL), 50, T0)
        it.supplier_accept(a)
        it.authorize(a, 50, T0)
        self.assertEqual(it.available_for_adhesion(), 150)
        self.assertFalse(it.lapse(a, T0 + 89 * DAY).ok)
        self.assertTrue(it.lapse(a, T0 + 91 * DAY).ok)
        self.assertEqual(it.available_for_adhesion(), 200)

    def test_exceptions(self):
        fed_ms = item(manager=Body("ms", Sphere.FEDERAL, is_health_ministry=True))
        d, _ = fed_ms.request(Body("x", Sphere.STATE), 50, T0, Exception_.HEALTH_EMERGENCY)
        self.assertTrue(d.ok)
        state_rec = item()
        d, _ = state_rec.request(Body("y", Sphere.MUNICIPAL), 10, T0, Exception_.FEDERAL_PROGRAMME)
        self.assertEqual(d.rule, "PROC-10")

    def test_alagoas_profile(self):
        outcomes = [st.solutio.ok for st in scenarios.s07_alagoas_art33().steps]
        self.assertEqual(outcomes, [False, True, True])

    def test_participants_and_roles(self):
        s = scenarios.s06_field_participant()
        self.assertEqual([st.solutio.ok for st in s.steps], [True, False, False, True])
        it = item()
        self.assertEqual(it.request(Body("m", Sphere.STATE), 1, T0)[0].rule, "PROC-07")

    def test_split_items_respect_per_item_caps(self):
        s = scenarios.s05_field_split_items()
        self.assertEqual([st.solutio.ok for st in s.steps], [False, True, True, True])
        self.assertEqual(s.metrics["taken"], 18)
        self.assertFalse(s.metrics["alert"])

    def test_tcu_1487(self):
        m = scenarios.s01_tcu_1487_2007().metrics
        self.assertEqual((m["granted_units"], m["refused_requests"]), (200, 58))

    def test_eu_framework(self):
        fa = FrameworkAgreement(10, {"A"})
        self.assertEqual(fa.call_off("B", 1).rule, "EU-02")
        self.assertTrue(fa.call_off("A", 10).ok)
        self.assertEqual(fa.call_off("A", 1).rule, "EU-01")


class Payments(unittest.TestCase):
    def test_single_financing_invariant_random(self):
        rng = random.Random(11)
        for _ in range(300):
            reg = ReceivableRegistry()
            verified = rng.randint(1, 10_000)
            reg.register(Receivable("r", "d", "c", verified))
            reg.attach_document("r", "doc")
            reg.mark_eligible("r", rng.randint(1, verified))
            for _ in range(20):
                reg.finance("r", rng.randint(1, verified))
                r = reg.receivables["r"]
                self.assertLessEqual(r.financed, r.eligible)

    def test_document_backs_one_receivable(self):
        reg = ReceivableRegistry()
        reg.register(Receivable("a", "d", "c", 10))
        reg.register(Receivable("b", "d", "c", 10))
        self.assertTrue(reg.attach_document("a", "nfe").ok)
        self.assertEqual(reg.attach_document("b", "nfe").rule, "PAY-04")

    def test_art141_queue(self):
        q = PaymentQueue()
        q.add(Claim("1", "s", Category.SERVICES, date(2026, 1, 1), 1))
        q.add(Claim("2", "s", Category.SERVICES, date(2026, 2, 1), 1))
        q.add(Claim("3", "s", Category.GOODS, date(2026, 3, 1), 1))
        self.assertTrue(q.pay("3").ok, "different category is a different queue")
        self.assertEqual(q.pay("2").rule, "PAY-02")
        self.assertTrue(q.pay("2", Art141Exception.ESSENTIAL).ok)
        self.assertEqual(len(q.log), 1)

    def test_overdue(self):
        self.assertEqual(overdue_flags(date(2026, 7, 1), date(2026, 9, 1)), [])
        self.assertTrue(overdue_flags(date(2026, 7, 1), date(2026, 9, 2)))
        self.assertEqual(overdue_flags(date(2026, 7, 1), date(2026, 7, 31), "EU"), [])
        self.assertTrue(overdue_flags(date(2026, 7, 1), date(2026, 8, 1), "EU"))
        self.assertEqual(overdue_flags(date(2026, 7, 1), date(2026, 8, 1), "EU", public_health=True), [])

    def test_antecipagov(self):
        c = AntecipaGovContract(100)
        self.assertEqual(c.advance("x", 71).rule, "PAY-06")
        self.assertTrue(c.advance("x", 70).ok)
        self.assertEqual(c.advance("y", 1).rule, "PAY-06")


class Economics(unittest.TestCase):
    def test_measured_costs_load_and_rent_dominates(self):
        lc = economics.lifecycle_costs(economics.load_costs())
        fees = lc["adhesion"].fee_lamports + lc["obligation"].fee_lamports
        rent = lc["adhesion"].rent_lamports + lc["obligation"].rent_lamports
        self.assertGreater(fees, 0)
        self.assertGreater(rent / (rent + fees), 0.95)

    def test_modes_are_ordered(self):
        lc = economics.lifecycle_costs(economics.load_costs())
        modes = ["no_close", "fees_only"] + (["measured_close"] if "closing" in lc else [])
        cost = {m: economics.per_obligation_lamports(lc, m) for m in modes}
        self.assertLess(cost["fees_only"], cost["no_close"])
        if "measured_close" in cost:
            self.assertLess(cost["measured_close"], cost["no_close"])
            self.assertGreater(cost["measured_close"], cost["fees_only"], "markers stay open by design")


if __name__ == "__main__":
    unittest.main()
