"""Payment-side rules: single financing per receivable, chronological payment
order, overdue flags and the AntecipaGov profile."""

from __future__ import annotations

from calendar import monthrange
from dataclasses import dataclass, field
from datetime import date
from enum import Enum
from typing import Optional

from .engine import Decision


# ---------------------------------------------------------------------------
# PAY-04: one receivable, one cumulative financing limit
# ---------------------------------------------------------------------------


@dataclass
class Receivable:
    id: str
    debtor: str
    creditor: str
    verified: int                 # liquidated amount, in cents
    eligible: int = 0
    financed: int = 0
    paid: int = 0
    documents: list[str] = field(default_factory=list)


@dataclass
class ReceivableRegistry:
    receivables: dict[str, Receivable] = field(default_factory=dict)
    document_owner: dict[str, str] = field(default_factory=dict)

    def register(self, r: Receivable) -> Decision:
        if r.id in self.receivables:
            return Decision.deny("PAY-04", "receivable already registered")
        if r.verified <= 0:
            return Decision.deny("PAY-01", "only a liquidated, positive amount can be registered")
        self.receivables[r.id] = r
        return Decision.allow("registered as verified")

    def attach_document(self, rid: str, doc_key: str) -> Decision:
        owner = self.document_owner.get(doc_key)
        if owner is not None:
            return Decision.deny("PAY-04", f"document already backs receivable {owner}")
        self.document_owner[doc_key] = rid
        self.receivables[rid].documents.append(doc_key)
        return Decision.allow("document bound")

    def mark_eligible(self, rid: str, amount: int) -> Decision:
        r = self.receivables[rid]
        if not r.documents:
            return Decision.deny("PAY-04", "eligibility needs at least one fiscal document")
        if amount > r.verified or amount < r.financed:
            return Decision.deny("PAY-04", "eligible must stay between financed and verified")
        r.eligible = amount
        return Decision.allow("eligible")

    def finance(self, rid: str, amount: int, notice: bool = True) -> Decision:
        r = self.receivables[rid]
        if not notice:
            return Decision.deny("PAY-05", "assignment requires notice to the debtor")
        if r.paid > 0:
            return Decision.deny("PAY-04", "financing closes once payment starts")
        if r.financed + amount > r.eligible:
            return Decision.deny("PAY-04", f"would exceed eligible balance ({r.eligible - r.financed} left)")
        r.financed += amount
        return Decision.allow(f"financed {amount}; {r.eligible - r.financed} left")


# ---------------------------------------------------------------------------
# PAY-02: chronological order per funding source and category (art. 141)
# ---------------------------------------------------------------------------


class Category(Enum):
    GOODS = "goods"
    LEASES = "leases"
    SERVICES = "services"
    WORKS = "works"


class Art141Exception(Enum):
    """Lei 14.133/2021, art. 141, §1, I-V."""

    EMERGENCY = "I: grave disturbance of order, emergency or public calamity"
    SMALL_BUSINESS_AT_RISK = "II: micro/small business, family farmer, MEI or cooperative at risk of discontinuity"
    STRUCTURAL_SYSTEMS = "III: services needed for structural systems, risk of discontinuity"
    INSOLVENCY = "IV: bankruptcy, judicial recovery or dissolution of the contractor"
    ESSENTIAL = "V: object essential to public assets or core activities, risk of discontinuity"


@dataclass
class Claim:
    id: str
    source: str
    category: Category
    payable_on: date
    amount: int
    paid: bool = False


@dataclass
class PaymentQueue:
    claims: list[Claim] = field(default_factory=list)
    log: list[tuple[str, str]] = field(default_factory=list)

    def add(self, c: Claim) -> None:
        self.claims.append(c)

    def next_in_line(self, source: str, category: Category) -> Optional[Claim]:
        pending = [c for c in self.claims if not c.paid and c.source == source and c.category == category]
        return min(pending, key=lambda c: (c.payable_on, c.id)) if pending else None

    def pay(self, claim_id: str, justification: Optional[Art141Exception] = None) -> Decision:
        c = next(c for c in self.claims if c.id == claim_id)
        head = self.next_in_line(c.source, c.category)
        if head is not None and head.id != c.id:
            if justification is None:
                return Decision.deny("PAY-02", f"{head.id} is earlier in the {c.source}/{c.category.value} queue")
            self.log.append((c.id, justification.value))
            c.paid = True
            return Decision.allow(f"out of order, justified ({justification.name}); must be published")
        c.paid = True
        return Decision.allow("paid in order")


def add_months(d: date, n: int) -> date:
    m = d.month - 1 + n
    y, m = d.year + m // 12, m % 12 + 1
    return date(y, m, min(d.day, monthrange(y, m)[1]))


def overdue_flags(invoice_date: date, today: date, regime: str = "BR", public_health: bool = False) -> list[str]:
    """PAY-03 (BR: more than two months entitles suspension) and PAY-07 (EU: 30/60 days)."""
    days = (today - invoice_date).days
    flags = []
    if regime == "BR":
        if today > add_months(invoice_date, 2):
            flags.append(f"PAY-03: {days} days overdue; supplier may suspend or terminate")
    elif regime == "EU":
        limit = 60 if public_health else 30
        if days > limit:
            flags.append(f"PAY-07: {days} days > {limit}-day limit")
    return flags


# ---------------------------------------------------------------------------
# PAY-06: AntecipaGov (IN SEGES/MGI 82/2025)
# ---------------------------------------------------------------------------


@dataclass
class AntecipaGovContract:
    remaining_to_pay: int
    operations: list[tuple[str, int]] = field(default_factory=list)   # (bank, amount)

    def advance(self, bank: str, amount: int) -> Decision:
        outstanding = sum(a for _, a in self.operations)
        limit = int(0.70 * (self.remaining_to_pay - outstanding))
        if self.operations and bank != self.operations[0][0]:
            return Decision.deny("PAY-06", "a follow-on operation must be with the same bank")
        if amount > limit:
            return Decision.deny("PAY-06", f"exceeds 70% of the remaining balance ({limit})")
        self.operations.append((bank, amount))
        return Decision.allow(f"advance of {amount} with {bank}")
