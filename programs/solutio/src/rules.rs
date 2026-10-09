//! Pure legal and accounting rules.
//!
//! Every rule the program enforces lives here as a plain function with no
//! Solana dependencies, so it can be unit-tested exhaustively and traced back to
//! its legal source (see `docs/LEGAL_RULES.md`). Instruction handlers only load
//! accounts, call these functions, and persist the result.

use crate::state::{AdhesionException, Sphere};

// ---------------------------------------------------------------------------
// Module 1: Carona — adhesion limits
// Law 14.133/2021, art. 86, regulated (federal sphere) by Decree 11.462/2023,
// arts. 31-33. Balance accounting mirrors the federal "Gestão de Atas" tool:
// quantities awaiting authorization already count against the caps.
// ---------------------------------------------------------------------------

/// Art. 31, §2 of Decree 11.462/2023: after authorization, the non-participant
/// must execute the purchase within ninety days, within the record's validity.
pub const EXECUTION_WINDOW_SECS: i64 = 90 * 86_400;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RuleViolation {
    InvalidQuantity,
    ExceedsIndividualCap,
    ExceedsGlobalCap,
    FederalAdhesionForbidden,
    AtaNotInForce,
    AtaNotActive,
    AdhesionsNotAllowed,
    ExceptionNotApplicable,
    ManagerCannotAdhere,
    Overflow,
}

/// Art. 86, §4 (Decree art. 32, I): each non-participating agency may adhere to
/// at most 50% of the quantity registered for the managing and participating
/// agencies.
///
/// Rounding of odd quantities is an open legal question (see LEGAL_RULES.md).
/// We use the floor, the conservative reading: it never authorizes more than
/// the statute allows.
pub fn individual_cap(registered_qty: u64) -> u64 {
    registered_qty / 2
}

/// Art. 86, §5 (Decree art. 32, II): all adhesions to an item together may not
/// exceed twice its registered quantity. The tender may set a lower maximum for
/// non-participants (Decree art. 15, XI); zero means adhesions are not allowed.
pub fn adhesion_cap(registered_qty: u64, max_adhesion_qty: u64) -> Option<u64> {
    Some(registered_qty.checked_mul(2)?.min(max_adhesion_qty))
}

/// Whether a claimed exception to the §5 cap applies (Decree art. 32, §§1-2):
/// - emergency purchase of medicines and medical supplies under a record
///   managed by the Ministry of Health;
/// - state, district or municipal adhesion required for voluntary transfers
///   executing a federal programme.
pub fn exception_applies(
    exception: AdhesionException,
    adherent_sphere: Sphere,
    manager_is_health_ministry: bool,
) -> bool {
    match exception {
        AdhesionException::None => true,
        AdhesionException::HealthEmergency => manager_is_health_ministry,
        AdhesionException::FederalProgramTransfer => adherent_sphere != Sphere::Federal,
    }
}

/// Snapshot of everything needed to decide whether a quantity may be reserved.
#[derive(Debug, Clone, Copy)]
pub struct ReserveCheck {
    pub registered_qty: u64,
    pub max_adhesion_qty: u64,
    /// Quantity committed by adhesions subject to the §5 cap (pending + authorized).
    pub committed_capped: u64,
    /// Quantity this agency already committed on this item (all adhesions).
    pub agency_committed: u64,
    pub requested_qty: u64,
    pub exception: AdhesionException,
    pub manager_sphere: Sphere,
    pub manager_is_health_ministry: bool,
    pub adherent_sphere: Sphere,
    pub adherent_is_manager: bool,
    pub ata_active: bool,
    pub now: i64,
    pub valid_from: i64,
    pub valid_until: i64,
}

/// Result of a lawful reservation: new per-agency total and how much of the
/// quantity counts toward the §5 cap.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Reservation {
    pub agency_committed: u64,
    pub capped_delta: u64,
    pub exempt_delta: u64,
}

/// Decides whether a request can reserve quantity. Reservation happens when the
/// request is made, as in the federal system ("saldo para adesões" already
/// discounts quantities awaiting authorization), so the first lawful request
/// gets the balance and later ones see it as taken.
pub fn check_reservation(c: &ReserveCheck) -> Result<Reservation, RuleViolation> {
    if c.requested_qty == 0 {
        return Err(RuleViolation::InvalidQuantity);
    }
    if !c.ata_active {
        return Err(RuleViolation::AtaNotActive);
    }
    if c.adherent_is_manager {
        return Err(RuleViolation::ManagerCannotAdhere);
    }
    if c.now < c.valid_from || c.now > c.valid_until {
        return Err(RuleViolation::AtaNotInForce);
    }
    // Art. 86, §8 (Decree art. 33).
    if c.adherent_sphere == Sphere::Federal && c.manager_sphere != Sphere::Federal {
        return Err(RuleViolation::FederalAdhesionForbidden);
    }
    if c.max_adhesion_qty == 0 {
        return Err(RuleViolation::AdhesionsNotAllowed);
    }
    if !exception_applies(c.exception, c.adherent_sphere, c.manager_is_health_ministry) {
        return Err(RuleViolation::ExceptionNotApplicable);
    }
    // §4 applies to every adhesion, exceptions included.
    let agency_committed = c
        .agency_committed
        .checked_add(c.requested_qty)
        .ok_or(RuleViolation::Overflow)?;
    if agency_committed > individual_cap(c.registered_qty) {
        return Err(RuleViolation::ExceedsIndividualCap);
    }
    if c.exception != AdhesionException::None {
        // Exempt adhesions are tracked separately and do not consume the §5 pool.
        return Ok(Reservation {
            agency_committed,
            capped_delta: 0,
            exempt_delta: c.requested_qty,
        });
    }
    let cap = adhesion_cap(c.registered_qty, c.max_adhesion_qty).ok_or(RuleViolation::Overflow)?;
    let new_capped = c
        .committed_capped
        .checked_add(c.requested_qty)
        .ok_or(RuleViolation::Overflow)?;
    if new_capped > cap {
        return Err(RuleViolation::ExceedsGlobalCap);
    }
    Ok(Reservation {
        agency_committed,
        capped_delta: c.requested_qty,
        exempt_delta: 0,
    })
}

/// Deadline to execute an authorized adhesion: ninety days, never past validity.
pub fn execution_deadline(authorized_at: i64, valid_until: i64) -> i64 {
    authorized_at.saturating_add(EXECUTION_WINDOW_SECS).min(valid_until)
}

// ---------------------------------------------------------------------------
// Module 2: Obligations — verified balances and single financing
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LedgerViolation {
    InvalidAmount,
    ExceedsVerified,
    ExceedsDocumented,
    EligibilityBelowFinanced,
    ExceedsFinanceable,
    NotFinanceable,
    ExceedsOutstanding,
    Overflow,
}

/// All monetary values are integer cents (BRL centavos in the Brazilian profile).
///
/// Meaning of each balance:
/// - `verified`: amount the debtor recognized as owed (liquidation, Law 4.320/1964 art. 63).
/// - `documented`: sum of distinct fiscal documents (e.g. NF-e) attached to the obligation.
/// - `reductions`: withholdings and disallowances (glosas) recorded by the debtor.
/// - `eligible`: amount a designated verifier confirmed as eligible for assignment.
///   Eligibility is a separate legal check; it is never inferred from `verified`.
/// - `financed`: cumulative amount financed against this obligation.
/// - `paid`: amount the debtor has paid.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct Balances {
    pub verified: u64,
    pub documented: u64,
    pub reductions: u64,
    pub eligible: u64,
    pub financed: u64,
    pub paid: u64,
}

impl Balances {
    /// Amount the debtor still owes: verified - reductions - paid.
    pub fn outstanding(&self) -> Option<u64> {
        self.verified
            .checked_sub(self.reductions)?
            .checked_sub(self.paid)
    }

    /// Remaining amount that may still be financed.
    pub fn financeable(&self) -> u64 {
        if self.paid > 0 {
            // MVP policy: financing closes once the debtor starts paying.
            return 0;
        }
        self.eligible.saturating_sub(self.financed)
    }

    /// True when a reduction left less owed than what has been financed.
    pub fn is_impaired(&self) -> bool {
        match self.verified.checked_sub(self.reductions) {
            Some(net) => self.financed > net,
            None => true,
        }
    }

    /// Structural invariants that must hold after every successful operation.
    /// `financed <= eligible` may only break through a reduction (a government
    /// act the protocol records but cannot refuse), which marks the obligation impaired.
    pub fn invariants_hold(&self) -> bool {
        let sums_ok = self
            .reductions
            .checked_add(self.paid)
            .map(|s| s <= self.verified)
            .unwrap_or(false);
        sums_ok
            && self.documented <= self.verified
            && self.eligible <= self.documented
            && (self.financed <= self.eligible || self.is_impaired() || self.reductions > 0)
    }
}

pub fn attach_document(b: Balances, amount: u64) -> Result<Balances, LedgerViolation> {
    if amount == 0 {
        return Err(LedgerViolation::InvalidAmount);
    }
    let documented = b
        .documented
        .checked_add(amount)
        .ok_or(LedgerViolation::Overflow)?;
    if documented > b.verified {
        return Err(LedgerViolation::ExceedsVerified);
    }
    Ok(Balances { documented, ..b })
}

pub fn mark_eligible(b: Balances, eligible: u64) -> Result<Balances, LedgerViolation> {
    if eligible == 0 {
        return Err(LedgerViolation::InvalidAmount);
    }
    if eligible > b.documented {
        return Err(LedgerViolation::ExceedsDocumented);
    }
    let outstanding = b.outstanding().ok_or(LedgerViolation::Overflow)?;
    if eligible > outstanding {
        return Err(LedgerViolation::ExceedsOutstanding);
    }
    if eligible < b.financed {
        return Err(LedgerViolation::EligibilityBelowFinanced);
    }
    Ok(Balances { eligible, ..b })
}

pub fn finance(b: Balances, amount: u64) -> Result<Balances, LedgerViolation> {
    if amount == 0 {
        return Err(LedgerViolation::InvalidAmount);
    }
    if b.eligible == 0 || b.paid > 0 || b.is_impaired() {
        return Err(LedgerViolation::NotFinanceable);
    }
    if amount > b.financeable() {
        return Err(LedgerViolation::ExceedsFinanceable);
    }
    let financed = b
        .financed
        .checked_add(amount)
        .ok_or(LedgerViolation::Overflow)?;
    Ok(Balances { financed, ..b })
}

/// Records a withholding or disallowance. Never refused for being inconvenient
/// to financiers: it is an act of the debtor. Eligibility is capped at the new
/// outstanding amount; if financing now exceeds it, the obligation is impaired.
pub fn apply_reduction(b: Balances, amount: u64) -> Result<Balances, LedgerViolation> {
    if amount == 0 {
        return Err(LedgerViolation::InvalidAmount);
    }
    let reductions = b
        .reductions
        .checked_add(amount)
        .ok_or(LedgerViolation::Overflow)?;
    let consumed = reductions
        .checked_add(b.paid)
        .ok_or(LedgerViolation::Overflow)?;
    if consumed > b.verified {
        return Err(LedgerViolation::ExceedsOutstanding);
    }
    let next = Balances { reductions, ..b };
    let outstanding = next.outstanding().ok_or(LedgerViolation::Overflow)?;
    Ok(Balances {
        eligible: next.eligible.min(outstanding),
        ..next
    })
}

pub fn apply_payment(b: Balances, amount: u64) -> Result<Balances, LedgerViolation> {
    if amount == 0 {
        return Err(LedgerViolation::InvalidAmount);
    }
    let outstanding = b.outstanding().ok_or(LedgerViolation::Overflow)?;
    if amount > outstanding {
        return Err(LedgerViolation::ExceedsOutstanding);
    }
    let paid = b.paid.checked_add(amount).ok_or(LedgerViolation::Overflow)?;
    Ok(Balances { paid, ..b })
}

// ---------------------------------------------------------------------------
// Tests: legal rule -> function -> test (traceability matrix in LEGAL_RULES.md)
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    fn base() -> ReserveCheck {
        ReserveCheck {
            registered_qty: 100,
            max_adhesion_qty: 200,
            committed_capped: 0,
            agency_committed: 0,
            requested_qty: 10,
            exception: AdhesionException::None,
            manager_sphere: Sphere::State,
            manager_is_health_ministry: false,
            adherent_sphere: Sphere::Municipal,
            adherent_is_manager: false,
            ata_active: true,
            now: 1_000,
            valid_from: 0,
            valid_until: 2_000,
        }
    }

    fn reserved(qty: u64, agency: u64) -> Result<Reservation, RuleViolation> {
        Ok(Reservation { agency_committed: agency, capped_delta: qty, exempt_delta: 0 })
    }

    #[test]
    fn art86_par4_individual_cap_is_half_rounded_down() {
        assert_eq!(individual_cap(100), 50);
        assert_eq!(individual_cap(75), 37);
        assert_eq!(individual_cap(1), 0);
        assert_eq!(check_reservation(&ReserveCheck { requested_qty: 50, ..base() }), reserved(50, 50));
        assert_eq!(
            check_reservation(&ReserveCheck { requested_qty: 51, ..base() }),
            Err(RuleViolation::ExceedsIndividualCap)
        );
        // Pending and authorized quantities of the same agency count together.
        assert_eq!(
            check_reservation(&ReserveCheck { agency_committed: 45, requested_qty: 6, ..base() }),
            Err(RuleViolation::ExceedsIndividualCap)
        );
    }

    #[test]
    fn art86_par5_cap_is_twice_registered_or_the_tender_maximum() {
        assert_eq!(adhesion_cap(100, 200), Some(200));
        assert_eq!(adhesion_cap(100, 500), Some(200), "the tender cannot raise the statutory cap");
        assert_eq!(adhesion_cap(100, 80), Some(80));
        let at_cap = ReserveCheck { committed_capped: 150, requested_qty: 50, ..base() };
        assert_eq!(check_reservation(&at_cap), reserved(50, 50));
        let over = ReserveCheck { committed_capped: 151, requested_qty: 50, ..base() };
        assert_eq!(check_reservation(&over), Err(RuleViolation::ExceedsGlobalCap));
        let tender_limit = ReserveCheck { max_adhesion_qty: 80, committed_capped: 40, requested_qty: 41, ..base() };
        assert_eq!(check_reservation(&tender_limit), Err(RuleViolation::ExceedsGlobalCap));
        let closed = ReserveCheck { max_adhesion_qty: 0, ..base() };
        assert_eq!(check_reservation(&closed), Err(RuleViolation::AdhesionsNotAllowed));
    }

    #[test]
    fn exceptions_lift_par5_only_and_only_when_applicable() {
        // Health emergency: only under a record managed by the Ministry of Health.
        let health = ReserveCheck {
            exception: AdhesionException::HealthEmergency,
            committed_capped: 200,
            requested_qty: 50,
            ..base()
        };
        assert_eq!(check_reservation(&health), Err(RuleViolation::ExceptionNotApplicable));
        let health_ms = ReserveCheck { manager_is_health_ministry: true, ..health };
        assert_eq!(
            check_reservation(&health_ms),
            Ok(Reservation { agency_committed: 50, capped_delta: 0, exempt_delta: 50 })
        );
        // §4 still applies under an exception.
        assert_eq!(
            check_reservation(&ReserveCheck { requested_qty: 51, ..health_ms }),
            Err(RuleViolation::ExceedsIndividualCap)
        );
        // Federal-programme transfers: state, district or municipal adherents only.
        let transfer = ReserveCheck {
            exception: AdhesionException::FederalProgramTransfer,
            committed_capped: 200,
            ..base()
        };
        assert!(check_reservation(&transfer).is_ok());
        let federal_transfer = ReserveCheck {
            adherent_sphere: Sphere::Federal,
            manager_sphere: Sphere::Federal,
            ..transfer
        };
        assert_eq!(check_reservation(&federal_transfer), Err(RuleViolation::ExceptionNotApplicable));
    }

    #[test]
    fn art86_par8_federal_cannot_adhere_to_non_federal() {
        for manager in [Sphere::State, Sphere::District, Sphere::Municipal] {
            let c = ReserveCheck { manager_sphere: manager, adherent_sphere: Sphere::Federal, ..base() };
            assert_eq!(check_reservation(&c), Err(RuleViolation::FederalAdhesionForbidden));
        }
        let fed_to_fed = ReserveCheck { manager_sphere: Sphere::Federal, adherent_sphere: Sphere::Federal, ..base() };
        assert!(check_reservation(&fed_to_fed).is_ok());
        let muni_to_fed = ReserveCheck { manager_sphere: Sphere::Federal, ..base() };
        assert!(check_reservation(&muni_to_fed).is_ok());
    }

    #[test]
    fn status_validity_and_basic_guards() {
        assert_eq!(check_reservation(&ReserveCheck { ata_active: false, ..base() }), Err(RuleViolation::AtaNotActive));
        assert_eq!(check_reservation(&ReserveCheck { now: 2_001, ..base() }), Err(RuleViolation::AtaNotInForce));
        assert_eq!(check_reservation(&ReserveCheck { now: -1, ..base() }), Err(RuleViolation::AtaNotInForce));
        assert_eq!(check_reservation(&ReserveCheck { requested_qty: 0, ..base() }), Err(RuleViolation::InvalidQuantity));
        assert_eq!(
            check_reservation(&ReserveCheck { adherent_is_manager: true, ..base() }),
            Err(RuleViolation::ManagerCannotAdhere)
        );
        assert_eq!(
            check_reservation(&ReserveCheck { agency_committed: u64::MAX, ..base() }),
            Err(RuleViolation::Overflow)
        );
    }

    #[test]
    fn execution_deadline_is_ninety_days_capped_by_validity() {
        assert_eq!(execution_deadline(0, i64::MAX), 90 * 86_400);
        assert_eq!(execution_deadline(0, 10 * 86_400), 10 * 86_400);
    }

    /// Deterministic pseudo-random generator so the randomized tests are reproducible.
    struct Lcg(u64);
    impl Lcg {
        fn next(&mut self) -> u64 {
            self.0 = self.0.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
            self.0 >> 33
        }
        fn below(&mut self, n: u64) -> u64 {
            self.next() % n
        }
    }

    /// Randomized check: across random reservations and releases (denials,
    /// partial authorizations, lapses), §4 and §5 hold at every step.
    #[test]
    fn randomized_reservations_and_releases_never_break_caps() {
        let mut rng = Lcg(42);
        for _ in 0..2_000 {
            let registered = 1 + rng.below(500);
            let max_adh = rng.below(registered * 2 + 50);
            let agencies = 1 + rng.below(8) as usize;
            let mut committed = vec![0u64; agencies];
            let mut capped = 0u64;
            for _ in 0..60 {
                let a = rng.below(agencies as u64) as usize;
                if rng.below(4) == 0 && committed[a] > 0 {
                    // Release part of this agency's commitment.
                    let r = 1 + rng.below(committed[a]);
                    committed[a] -= r;
                    capped -= r;
                } else {
                    let c = ReserveCheck {
                        registered_qty: registered,
                        max_adhesion_qty: max_adh,
                        committed_capped: capped,
                        agency_committed: committed[a],
                        requested_qty: 1 + rng.below(registered),
                        ..base()
                    };
                    if let Ok(r) = check_reservation(&c) {
                        committed[a] = r.agency_committed;
                        capped += r.capped_delta;
                    }
                }
                assert!(committed[a] <= individual_cap(registered));
                assert!(capped <= adhesion_cap(registered, max_adh).unwrap());
                assert_eq!(capped, committed.iter().sum::<u64>());
            }
        }
    }

    fn verified(amount: u64) -> Balances {
        Balances { verified: amount, ..Default::default() }
    }

    #[test]
    fn eligibility_is_separate_from_verification() {
        let b = verified(1_000);
        // Verified but never marked eligible: cannot be financed.
        assert_eq!(finance(b, 1), Err(LedgerViolation::NotFinanceable));
        // Eligibility requires fiscal documents backing it.
        assert_eq!(mark_eligible(b, 500), Err(LedgerViolation::ExceedsDocumented));
        let b = attach_document(b, 600).unwrap();
        assert_eq!(attach_document(b, 401), Err(LedgerViolation::ExceedsVerified));
        let b = mark_eligible(b, 600).unwrap();
        assert_eq!(b.financeable(), 600);
    }

    #[test]
    fn the_same_balance_cannot_be_financed_twice() {
        let b = mark_eligible(attach_document(verified(1_000), 1_000).unwrap(), 1_000).unwrap();
        let b = finance(b, 700).unwrap();
        assert_eq!(finance(b, 301), Err(LedgerViolation::ExceedsFinanceable));
        let b = finance(b, 300).unwrap();
        assert_eq!(b.financeable(), 0);
        assert_eq!(finance(b, 1), Err(LedgerViolation::ExceedsFinanceable));
        assert_eq!(mark_eligible(b, 900), Err(LedgerViolation::EligibilityBelowFinanced));
    }

    #[test]
    fn reductions_cap_eligibility_and_flag_impairment() {
        let b = mark_eligible(attach_document(verified(1_000), 1_000).unwrap(), 1_000).unwrap();
        let b = finance(b, 900).unwrap();
        let b = apply_reduction(b, 200).unwrap();
        assert_eq!(b.eligible, 800);
        assert!(b.is_impaired());
        assert_eq!(finance(b, 1), Err(LedgerViolation::NotFinanceable));
        assert_eq!(apply_reduction(b, 801), Err(LedgerViolation::ExceedsOutstanding));
    }

    #[test]
    fn payments_close_financing_and_settle() {
        let b = mark_eligible(attach_document(verified(1_000), 1_000).unwrap(), 1_000).unwrap();
        let b = finance(b, 400).unwrap();
        let b = apply_payment(b, 400).unwrap();
        assert_eq!(b.financeable(), 0);
        assert_eq!(finance(b, 1), Err(LedgerViolation::NotFinanceable));
        assert_eq!(apply_payment(b, 601), Err(LedgerViolation::ExceedsOutstanding));
        let b = apply_payment(b, 600).unwrap();
        assert_eq!(b.outstanding(), Some(0));
    }

    /// Randomized check: across random operation sequences the financed total
    /// never exceeds eligibility, except after a recorded reduction (impairment).
    #[test]
    fn randomized_ledger_sequences_keep_invariants() {
        let mut rng = Lcg(7);
        for _ in 0..2_000 {
            let mut b = verified(1 + rng.below(1_000_000));
            for _ in 0..30 {
                let amt = 1 + rng.below(b.verified);
                let next = match rng.below(5) {
                    0 => attach_document(b, amt),
                    1 => mark_eligible(b, amt),
                    2 => finance(b, amt),
                    3 => apply_reduction(b, amt),
                    _ => apply_payment(b, amt),
                };
                if let Ok(n) = next {
                    if n.financed > b.financed {
                        // A financing step must respect eligibility at that moment.
                        assert!(n.financed <= n.eligible);
                    }
                    b = n;
                }
                assert!(b.invariants_hold(), "invariant broken: {:?}", b);
            }
        }
    }
}
