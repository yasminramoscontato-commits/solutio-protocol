"""Replays of documented cases through the rules.

Each scenario is built from a public artifact (evidence id in evidence.json) or
from the founder's anonymized field observation. Where a documented defect
exists, the same sequence runs under LEGACY and under STRICT (Marjan), so the
difference is measured, not asserted.

Quantities are normalized (e.g. 100 units = the registered quantity) unless the
source gives real figures. Nothing here claims to reproduce internal data of
any government system.
"""

from __future__ import annotations

from dataclasses import dataclass, field
from datetime import date
from typing import Optional

from .engine import (
    DAY, LEGACY, STRICT, Body, ControlMode, Decision, Exception_, FrameworkAgreement,
    PriceRecordItem, Sphere, object_aggregation,
)
from .payments import (
    AntecipaGovContract, Art141Exception, Category, Claim, PaymentQueue, Receivable,
    ReceivableRegistry, overdue_flags,
)

T0 = 1_000 * DAY


@dataclass
class Step:
    action: str
    legacy: Optional[Decision]
    marjan: Decision


@dataclass
class Scenario:
    id: str
    title: str
    level: str                      # global | eu | national | state | municipal | field
    evidence: list[str]
    question: str
    steps: list[Step] = field(default_factory=list)
    metrics: dict = field(default_factory=dict)
    finding: str = ""


def _item(mode: ControlMode, registered: int = 100, manager: Optional[Body] = None, days: int = 365, **kw) -> PriceRecordItem:
    manager = manager or Body("manager", Sphere.STATE)
    return PriceRecordItem(manager, {manager.id: registered}, T0 - DAY, T0 + days * DAY, mode=mode, **kw)


def _bodies(n: int, sphere: Sphere = Sphere.MUNICIPAL, prefix: str = "body") -> list[Body]:
    return [Body(f"{prefix}-{i + 1}", sphere) for i in range(n)]


# ---------------------------------------------------------------------------


def s01_tcu_1487_2007() -> Scenario:
    s = Scenario(
        "S01", "62 bodies adhere to one record (TCU Acórdão 1.487/2007)", "national", ["BR-TCU-1487-2007"],
        "What would a record registered for 100 units deliver if 62 non-participants each asked for half?",
    )
    item = _item(STRICT)
    granted = refused = 0
    for b in _bodies(62):
        d, a = item.request(b, 50, T0)
        if d.ok:
            item.supplier_accept(a)
            item.authorize(a, 50, T0)
            granted += 50
        else:
            refused += 1
    s.steps.append(Step("62 requests of 50 units each (half the record)", None,
                        Decision.allow(f"{granted} units granted, {refused} requests refused by PROC-02")))
    s.metrics = {"requested_units": 62 * 50, "granted_units": granted, "refused_requests": refused,
                 "uncapped_multiple_reported_by_tcu": 62}
    s.finding = (f"Without caps, TCU computed a theoretical exposure of 62x the tendered value (if every adherent bought the full "
                 f"quantity). With art. 86 enforced at request time, the 5th and later requests are refused: {granted} units (2x) is the ceiling.")
    return s


def s02_rounding() -> Scenario:
    s = Scenario(
        "S02", "50% of an odd quantity (TCU 547/2026, para. 67)", "national", ["BR-TCU-547-2026"],
        "Does the control round the 50% cap up or down?",
    )
    for mode in (LEGACY, STRICT):
        item = _item(mode, registered=21)
        d, _ = item.request(Body("adherent", Sphere.MUNICIPAL), 11, T0)
        s.metrics[mode.name] = {"cap": item.individual_cap(), "request_11": d.ok}
    legacy_cap, strict_cap = s.metrics[LEGACY.name]["cap"], s.metrics[STRICT.name]["cap"]
    s.steps.append(Step("Adherent requests 11 of 21 registered units",
                        Decision.allow(f"cap {legacy_cap}: 11 accepted (52.4% > 50%)"),
                        Decision.deny("PROC-01", f"cap {strict_cap}: 11 refused")))
    s.finding = "Rounding up lets each adherent exceed the statutory 50%; TCU recommends rounding down (item 9.1.1.6), which Marjan does."
    return s


def s03_negative_balance() -> Scenario:
    s = Scenario(
        "S03", "Adhesion balance goes negative (TCU 547/2026, para. 66)", "national", ["BR-TCU-547-2026"],
        "Can concurrent pending requests push total adhesions above 2x?",
    )
    for mode in (LEGACY, STRICT):
        item = _item(mode)
        pending = []
        for b in _bodies(5):
            d, a = item.request(b, 50, T0)
            if d.ok:
                pending.append(a)
        for a in pending:
            item.supplier_accept(a)
            item.authorize(a, 50, T0)
        s.metrics[mode.name] = {"authorized_units": sum(a.authorized_qty for a in item.adhesions),
                                "available_for_adhesion": item.available_for_adhesion()}
    leg, sol = s.metrics[LEGACY.name], s.metrics[STRICT.name]
    s.steps.append(Step("Five bodies file 50 units each before any is authorized; all are then authorized",
                        Decision.allow(f"{leg['authorized_units']} authorized; balance {leg['available_for_adhesion']}"),
                        Decision.deny("PROC-02", f"5th request refused at filing; balance {sol['available_for_adhesion']}")))
    s.finding = ("When pending requests do not reserve quantity, the balance goes negative exactly as TCU observed "
                 "(-2, -155, -25 in real records). Reserving at request time makes a negative balance unreachable.")
    return s


def s04_authorize_after_expiry() -> Scenario:
    s = Scenario(
        "S04", "Authorization after the record expired (TCU 547/2026, para. 68)", "national", ["BR-TCU-547-2026"],
        "Can a request filed while valid be authorized after expiry?",
    )
    for mode in (LEGACY, STRICT):
        item = _item(mode, days=10)
        d, a = item.request(Body("adherent", Sphere.MUNICIPAL), 10, T0)
        item.supplier_accept(a)
        s.metrics[mode.name] = item.authorize(a, 10, T0 + 20 * DAY)
    s.steps.append(Step("Request on day 0, authorization on day 20, record valid until day 10",
                        s.metrics[LEGACY.name], s.metrics[STRICT.name]))
    s.metrics = {k: v.ok for k, v in s.metrics.items()}
    s.finding = "Marjan checks validity at authorization too (TCU item 9.1.1.7)."
    return s


def s05_field_split_items() -> Scenario:
    s = Scenario(
        "S05", "A body needing 18 units reaches them across three identical items", "field", ["FIELD-STATE-CPB"],
        "Does the per-item cap hold, and what is the body's share of the object?",
    )
    obj = "CATSER-outsourcing-A3"
    manager = Body("central-purchasing-body", Sphere.STATE, profile="BR-AL")
    items = [PriceRecordItem(manager, {manager.id: q}, T0 - DAY, T0 + 365 * DAY, object_key=obj) for q in (10, 21, 8)]
    body = Body("state-secretariat", Sphere.STATE, profile="BR-AL")
    d, _ = items[1].request(body, 18, T0)
    s.steps.append(Step("Request 18 units on the item with 21 registered", None, d))
    for item, qty in zip(items, (4, 10, 4)):
        d, a = item.request(body, qty, T0)
        s.steps.append(Step(f"Request {qty} of {item.registered} registered", None, d))
    agg = object_aggregation(body, items)
    s.metrics = agg
    s.finding = (f"Each item stays within 50% ({agg['per_item_share']}%). Across the object the body holds "
                 f"{agg['share_of_object']}% of {agg['registered_across_items']} units, so the split also respects a "
                 "per-object reading. Marjan reports this aggregate (PROC-15) without judging it.")
    return s


def s06_field_participant() -> Scenario:
    s = Scenario(
        "S06", "Participant supply authorizations and rectification", "field", ["FIELD-STATE-CPB"],
        "Can a body use a record where it is not a participant, and what happens when an authorization is rectified?",
    )
    manager = Body("central-purchasing-body", Sphere.STATE, profile="BR-AL")
    uni = Body("state-university", Sphere.STATE, profile="BR-AL")
    with_uni = PriceRecordItem(manager, {manager.id: 500, uni.id: 48}, T0 - DAY, T0 + 365 * DAY)
    without_uni = PriceRecordItem(manager, {manager.id: 500}, T0 - DAY, T0 + 365 * DAY)
    s.steps.append(Step("Use 48 units of a record where it is a participant", None, with_uni.participant_use(uni, 48, T0)))
    s.steps.append(Step("Use 1 more unit", None, with_uni.participant_use(uni, 1, T0)))
    s.steps.append(Step("Use 8 units of a record where it is not a participant", None, without_uni.participant_use(uni, 8, T0)))
    s.steps.append(Step("Rectify the authorization from 48 to 36 after budget review", None, with_uni.participant_rectify(uni, 12)))
    s.metrics = {"quota_left_after_rectification": with_uni.participant_quotas[uni.id] - with_uni.participant_used[uni.id]}
    s.finding = "The record itself refuses a non-participant and restores rectified quantity, instead of a manual check in spreadsheets."
    return s


def s07_alagoas_art33() -> Scenario:
    s = Scenario(
        "S07", "Alagoas: state bodies and municipal records (Decree 95.019/2023, art. 33)", "state", ["AL-DECREE-95019"],
        "Which municipal records may an Alagoas state body adhere to?",
    )
    al_state = Body("al-state-secretariat", Sphere.STATE, profile="BR-AL")
    other_state = Body("other-state-secretariat", Sphere.STATE, profile="BR")
    interior = _item(STRICT, manager=Body("interior-municipality", Sphere.MUNICIPAL))
    capital = _item(STRICT, manager=Body("capital-municipality", Sphere.MUNICIPAL, is_state_capital=True))
    s.steps.append(Step("Alagoas state body → non-capital municipal record", None, interior.request(al_state, 10, T0)[0]))
    s.steps.append(Step("Alagoas state body → state-capital municipal record", None, capital.request(al_state, 10, T0)[0]))
    s.steps.append(Step("State body under the federal baseline → same non-capital record", None, interior.request(other_state, 10, T0)[0]))
    s.finding = "The same request has different outcomes depending on the adherent's regulation: rules are profiles, not hard-coded law."
    return s


def s08_eu_framework() -> Scenario:
    s = Scenario(
        "S08", "EU framework agreement with a split maximum (CJEU C-23/20)", "eu", ["EU-CJEU-C23-20", "EU-CJEU-C216-17"],
        "Does the same engine enforce the EU rule that an agreement ends at its maximum?",
    )
    fa = FrameworkAgreement(1_000, {"authority-A", "authority-B"}, split={"authority-A": 600, "authority-B": 400})
    for who, qty, label in [("authority-C", 10, "Unidentified authority calls off 10"), ("authority-A", 600, "A calls off its 600"),
                            ("authority-A", 1, "A calls off 1 more"), ("authority-B", 400, "B calls off its 400"),
                            ("authority-B", 1, "B calls off 1 more")]:
        s.steps.append(Step(label, None, fa.call_off(who, qty)))
    s.metrics = {"exhausted": fa.exhausted, "total_called": fa.total_called}
    s.finding = "Same failure mode, different law: identified buyers, a shared maximum and per-buyer shares."
    return s


def s09_double_financing() -> Scenario:
    s = Scenario(
        "S09", "One receivable, several financiers (Qingdao, First Brands, Tricolor pattern)", "global",
        ["GL-QINGDAO-2014", "GL-FIRSTBRANDS-2025", "GL-TRICOLOR-2025"],
        "Can the same government receivable or document be financed twice?",
    )
    reg = ReceivableRegistry()
    reg.register(Receivable("NE-001", "municipality", "supplier", 1_000_000))
    reg.attach_document("NE-001", "NFE-KEY-1")
    reg.mark_eligible("NE-001", 1_000_000)
    s.steps.append(Step("Financier A advances R$ 6.000", None, reg.finance("NE-001", 600_000)))
    s.steps.append(Step("Financier B advances R$ 4.000", None, reg.finance("NE-001", 400_000)))
    s.steps.append(Step("Financier C advances R$ 0,01", None, reg.finance("NE-001", 1)))
    reg.register(Receivable("NE-002", "municipality", "supplier", 1_000_000))
    s.steps.append(Step("The same invoice is attached to a second receivable", None, reg.attach_document("NE-002", "NFE-KEY-1")))
    s.finding = "Duplicate pledges become impossible inside the registry; financing outside it is not prevented (stated limitation)."
    return s


def s10_art141_queue() -> Scenario:
    s = Scenario(
        "S10", "Chronological payment order (Lei 14.133, art. 141)", "national", ["BR-LAW-14133-ART141"],
        "Is an out-of-order payment visible and justified?",
    )
    q = PaymentQueue()
    q.add(Claim("C-1", "Tesouro", Category.GOODS, date(2026, 1, 10), 50_000))
    q.add(Claim("C-2", "Tesouro", Category.GOODS, date(2026, 2, 5), 80_000))
    s.steps.append(Step("Pay C-2 before C-1 without justification", None, q.pay("C-2")))
    s.steps.append(Step("Pay C-2 before C-1 citing art. 141 §1, II", None, q.pay("C-2", Art141Exception.SMALL_BUSINESS_AT_RISK)))
    s.steps.append(Step("Pay C-1", None, q.pay("C-1")))
    s.finding = "Deviation is allowed only with a listed justification, and the log is what must be published monthly (§3)."
    return s


def s11_overdue() -> Scenario:
    s = Scenario(
        "S11", "When delay becomes a legal event", "global", ["BR-LAW-14133-ART137", "EU-DIR-2011-7"],
        "When does a late payment cross a statutory threshold?",
    )
    inv, today = date(2026, 7, 1), date(2026, 10, 9)
    for regime in ("BR", "EU"):
        flags = overdue_flags(inv, today, regime)
        s.steps.append(Step(f"{regime}: invoice of 2026-07-01, checked on 2026-10-09", None,
                            Decision(not flags, flags[0].split(":")[0] if flags else None, flags[0] if flags else "on time")))
    s.finding = "Each obligation's age is public on-chain, so the moment it crosses a legal threshold is verifiable."
    return s


def s12_antecipagov() -> Scenario:
    s = Scenario(
        "S12", "AntecipaGov limits (IN SEGES/MGI 82/2025)", "national", ["BR-ANTECIPAGOV"],
        "Do the federal advance rules translate into a profile?",
    )
    c = AntecipaGovContract(100_000)
    s.steps.append(Step("Bank X advances 70.000 of 100.000 remaining", None, c.advance("bank-X", 70_000)))
    s.steps.append(Step("Bank Y offers a follow-on", None, c.advance("bank-Y", 5_000)))
    s.steps.append(Step("Bank X advances 30.000 more", None, c.advance("bank-X", 30_000)))
    s.steps.append(Step("Bank X advances 21.000 more", None, c.advance("bank-X", 21_000)))
    s.finding = "A federal programme's financing rules become one more profile over the same single-financing ledger."
    return s


ALL = [s01_tcu_1487_2007, s02_rounding, s03_negative_balance, s04_authorize_after_expiry, s05_field_split_items,
       s06_field_participant, s07_alagoas_art33, s08_eu_framework, s09_double_financing, s10_art141_queue,
       s11_overdue, s12_antecipagov]


def run_all() -> list[Scenario]:
    return [f() for f in ALL]
