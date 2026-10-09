"""Reference engine for shared purchasing instruments.

Pure Python, no dependencies. It mirrors the rules the Solana program enforces
(see programs/solutio/src/rules.rs) and adds the rules that are not on-chain yet
(participants, rectification, EU framework agreements, aggregation alerts).

Every decision cites a rule id from rulebook.json, so a refusal can always be
traced to its legal source.
"""

from __future__ import annotations

from dataclasses import dataclass, field
from enum import Enum
from typing import Optional

DAY = 86_400
EXECUTION_WINDOW = 90 * DAY


class Sphere(Enum):
    FEDERAL = "federal"
    STATE = "state"
    DISTRICT = "district"
    MUNICIPAL = "municipal"


class Exception_(Enum):
    NONE = "none"
    HEALTH_EMERGENCY = "health_emergency"          # art. 86 §7
    FEDERAL_PROGRAMME = "federal_programme"        # art. 86 §6


@dataclass(frozen=True)
class Body:
    """A public body. `profile` is the regulation governing it as an adherent."""

    id: str
    sphere: Sphere
    profile: str = "BR"            # "BR" (federal baseline) or "BR-AL" (Alagoas)
    is_state_capital: bool = False
    is_health_ministry: bool = False


@dataclass(frozen=True)
class Decision:
    ok: bool
    rule: Optional[str]
    reason: str

    @staticmethod
    def allow(reason: str = "allowed") -> "Decision":
        return Decision(True, None, reason)

    @staticmethod
    def deny(rule: str, reason: str) -> "Decision":
        return Decision(False, rule, reason)


@dataclass(frozen=True)
class ControlMode:
    """How strictly a control system applies the rules.

    STRICT is Solutio. LEGACY is an illustrative control that reproduces the
    three outcomes documented by TCU Acórdão 547/2026 (individual cap rounded
    up; global balance allowed to go negative; authorization after expiry).
    It does not claim to be the federal system's internal logic.
    """

    name: str
    rounding: str                  # "floor" or "ceil"
    reserve_pending: bool          # pending requests count against the global cap
    check_expiry_on_authorize: bool


STRICT = ControlMode("Solutio", "floor", True, True)
LEGACY = ControlMode("Legacy control (outcomes documented by TCU 547/2026)", "ceil", False, False)


# ---------------------------------------------------------------------------
# Brazil: price-registration record item (ata de registro de preços)
# ---------------------------------------------------------------------------


@dataclass
class Adhesion:
    id: int
    body: Body
    qty: int
    exception: Exception_
    state: str = "requested"       # requested -> accepted -> authorized -> executed | lapsed | denied
    authorized_qty: int = 0
    execute_by: Optional[int] = None


@dataclass
class PriceRecordItem:
    manager: Body
    participant_quotas: dict[str, int]        # body id -> registered quantity (manager included if it estimated)
    valid_from: int
    valid_until: int
    max_adhesion: Optional[int] = None        # tender maximum for non-participants (Decree art. 15, XI)
    object_key: str = ""                      # e.g. CATMAT/CATSER code, for aggregation alerts
    mode: ControlMode = STRICT
    active: bool = True
    participant_used: dict[str, int] = field(default_factory=dict)
    adhesions: list[Adhesion] = field(default_factory=list)

    # ---- quantities ----
    @property
    def registered(self) -> int:
        return sum(self.participant_quotas.values())

    def individual_cap(self) -> int:
        half_num, half_den = self.registered, 2
        if self.mode.rounding == "floor":
            return half_num // half_den
        return -(-half_num // half_den)

    def global_cap(self) -> int:
        cap = 2 * self.registered
        return cap if self.max_adhesion is None else min(cap, self.max_adhesion)

    def _counted(self, states: tuple[str, ...], exempt: bool) -> int:
        total = 0
        for a in self.adhesions:
            if a.state not in states or (a.exception != Exception_.NONE) != exempt:
                continue
            total += a.authorized_qty if a.state in ("authorized", "executed") else a.qty
        return total

    def committed_capped(self) -> int:
        states = ("requested", "accepted", "authorized", "executed") if self.mode.reserve_pending else ("authorized", "executed")
        return self._counted(states, exempt=False)

    def available_for_adhesion(self) -> int:
        """May be negative only under LEGACY: that is the defect TCU found."""
        return self.global_cap() - self._counted(("requested", "accepted", "authorized", "executed"), exempt=False)

    def body_committed(self, body: Body) -> int:
        return sum(
            (a.authorized_qty if a.state in ("authorized", "executed") else a.qty)
            for a in self.adhesions
            if a.body.id == body.id and a.state in ("requested", "accepted", "authorized", "executed")
        )

    # ---- participants (PROC-11, PROC-12) ----
    def participant_use(self, body: Body, qty: int, now: int) -> Decision:
        if qty <= 0:
            return Decision.deny("PROC-11", "quantity must be positive")
        if body.id not in self.participant_quotas:
            return Decision.deny("PROC-11", f"{body.id} is not a participant of this record")
        if not (self.valid_from <= now <= self.valid_until) or not self.active:
            return Decision.deny("PROC-05", "record not in force")
        used = self.participant_used.get(body.id, 0)
        if used + qty > self.participant_quotas[body.id]:
            return Decision.deny("PROC-11", f"exceeds own quota ({self.participant_quotas[body.id] - used} left)")
        self.participant_used[body.id] = used + qty
        return Decision.allow(f"supply authorized from own quota ({self.participant_quotas[body.id] - used - qty} left)")

    def participant_rectify(self, body: Body, released: int) -> Decision:
        used = self.participant_used.get(body.id, 0)
        if released <= 0 or released > used:
            return Decision.deny("PROC-12", "can only release quantity previously authorized")
        self.participant_used[body.id] = used - released
        return Decision.allow(f"{released} restored to {body.id}'s quota")

    # ---- non-participants (carona) ----
    def request(self, body: Body, qty: int, now: int, exception: Exception_ = Exception_.NONE) -> tuple[Decision, Optional[Adhesion]]:
        if qty <= 0:
            return Decision.deny("PROC-01", "quantity must be positive"), None
        if not self.active:
            return Decision.deny("PROC-05", "record suspended or cancelled"), None
        if body.id == self.manager.id:
            return Decision.deny("PROC-07", "the manager cannot adhere to its own record"), None
        if body.id in self.participant_quotas:
            return Decision.deny("PROC-07", "a participant uses its own quota, not an adhesion"), None
        if not (self.valid_from <= now <= self.valid_until):
            return Decision.deny("PROC-05", "record not in force"), None
        if body.sphere == Sphere.FEDERAL and self.manager.sphere != Sphere.FEDERAL:
            return Decision.deny("PROC-08", "federal body cannot adhere to a non-federal record"), None
        if (
            body.profile == "BR-AL"
            and body.sphere == Sphere.STATE
            and self.manager.sphere == Sphere.MUNICIPAL
            and not self.manager.is_state_capital
        ):
            return Decision.deny("PROC-09", "Alagoas state body cannot adhere to a non-capital municipal record"), None
        if self.global_cap() == 0:
            return Decision.deny("PROC-02", "the tender does not allow adhesions"), None
        if exception == Exception_.HEALTH_EMERGENCY and not self.manager.is_health_ministry:
            return Decision.deny("PROC-10", "health exception requires a Ministry of Health record"), None
        if exception == Exception_.FEDERAL_PROGRAMME and not (
            body.sphere != Sphere.FEDERAL and self.manager.sphere == Sphere.FEDERAL
        ):
            return Decision.deny("PROC-10", "federal-programme exception requires a federal record and a subnational adherent"), None
        if self.body_committed(body) + qty > self.individual_cap():
            return Decision.deny("PROC-01", f"exceeds 50% cap ({self.individual_cap()} units)"), None
        # STRICT counts pending requests (they reserve); LEGACY counts only
        # authorized quantities, so concurrent pending requests all pass.
        if exception == Exception_.NONE and self.committed_capped() + qty > self.global_cap():
            return Decision.deny("PROC-02", f"exceeds global cap ({max(self.available_for_adhesion(), 0)} available)"), None
        a = Adhesion(len(self.adhesions) + 1, body, qty, exception)
        self.adhesions.append(a)
        return Decision.allow("quantity reserved" if self.mode.reserve_pending else "request filed"), a

    def supplier_accept(self, a: Adhesion) -> Decision:
        if a.state != "requested":
            return Decision.deny("PROC-04", "only a pending request can be accepted")
        a.state = "accepted"
        return Decision.allow("supplier accepted by signature")

    def authorize(self, a: Adhesion, qty: int, now: int) -> Decision:
        if a.state != "accepted":
            return Decision.deny("PROC-04", "supplier acceptance must come first")
        if not 0 < qty <= a.qty:
            return Decision.deny("PROC-01", "authorized quantity must be within the request")
        if self.mode.check_expiry_on_authorize and not (self.valid_from <= now <= self.valid_until):
            return Decision.deny("PROC-05", "record expired: authorization refused")
        # STRICT reserved the quantity at request time, so authorization can only
        # keep or release it. LEGACY never re-checks here, which is how balances
        # end up negative when several pending requests are approved.
        a.state, a.authorized_qty = "authorized", qty
        a.execute_by = min(now + EXECUTION_WINDOW, self.valid_until)
        return Decision.allow(f"authorized {qty}; execute by day {a.execute_by // DAY}")

    def execute(self, a: Adhesion, now: int) -> Decision:
        if a.state != "authorized":
            return Decision.deny("PROC-06", "only an authorized adhesion can be executed")
        if a.execute_by is not None and now > a.execute_by:
            return Decision.deny("PROC-06", "90-day execution deadline passed")
        a.state = "executed"
        return Decision.allow("executed (contract or commitment note)")

    def lapse(self, a: Adhesion, now: int) -> Decision:
        if a.state != "authorized" or a.execute_by is None or now <= a.execute_by:
            return Decision.deny("PROC-06", "nothing to lapse")
        a.state = "lapsed"
        return Decision.allow(f"lapsed; {a.authorized_qty} returned to the balance")


def object_aggregation(body: Body, items: list[PriceRecordItem]) -> dict:
    """PROC-15 advisory: a body's share of an object's total registered quantity
    across several items with the same `object_key`."""
    registered = sum(i.registered for i in items)
    taken = sum(i.body_committed(body) for i in items)
    per_item = [round(100 * i.body_committed(body) / i.registered, 1) for i in items]
    share = taken / registered if registered else 0.0
    return {
        "rule": "PROC-15",
        "body": body.id,
        "items": len(items),
        "taken": taken,
        "registered_across_items": registered,
        "share_of_object": round(100 * share, 1),
        "per_item_share": per_item,
        "alert": share > 0.5,
    }


# ---------------------------------------------------------------------------
# European Union: framework agreement (Directive 2014/24/EU, art. 33)
# ---------------------------------------------------------------------------


@dataclass
class FrameworkAgreement:
    maximum: int
    identified_authorities: set[str]
    split: Optional[dict[str, int]] = None
    called: dict[str, int] = field(default_factory=dict)

    @property
    def total_called(self) -> int:
        return sum(self.called.values())

    @property
    def exhausted(self) -> bool:
        return self.total_called >= self.maximum

    def call_off(self, authority: str, qty: int) -> Decision:
        if authority not in self.identified_authorities:
            return Decision.deny("EU-02", f"{authority} is not identified in the notice")
        if self.exhausted:
            return Decision.deny("EU-01", "maximum reached: the agreement has no further effect")
        if self.total_called + qty > self.maximum:
            return Decision.deny("EU-01", f"exceeds the maximum ({self.maximum - self.total_called} left)")
        if self.split is not None:
            own = self.split.get(authority, 0)
            if self.called.get(authority, 0) + qty > own:
                return Decision.deny("EU-03", f"exceeds {authority}'s share ({own - self.called.get(authority, 0)} left)")
        self.called[authority] = self.called.get(authority, 0) + qty
        return Decision.allow(f"call-off accepted ({self.maximum - self.total_called} left)")
