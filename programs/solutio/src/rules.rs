//! Pure legal and accounting rules.
//!
//! Every rule the program enforces lives here as a plain function with no
//! Solana dependencies, so it can be unit-tested exhaustively and traced back to
//! its legal source (see `docs/LEGAL_RULES.md`). Instruction handlers only load
//! accounts, call these functions, and persist the result.

use crate::state::Sphere;

// ---------------------------------------------------------------------------
// Module 1: Carona — adhesion limits (Law 14.133/2021, art. 86)
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RuleViolation {
    InvalidQuantity,
    ExceedsIndividualCap,
    ExceedsGlobalCap,
    FederalAdhesionForbidden,
    AtaNotInForce,
    ManagerCannotAdhere,
    Overflow,
}

/// Art. 86, §4: each non-participating agency may adhere to at most 50% of the
/// quantity registered for the managing and participating agencies.
///
/// Rounding of odd quantities is an open legal question (see LEGAL_RULES.md).
/// We use the floor, the conservative reading: it never authorizes more than
/// the statute allows.
pub fn individual_cap(registered_qty: u64) -> u64 {
    registered_qty / 2
}

/// Art. 86, §5: the sum of all adhesions to an item may not exceed twice the
/// quantity registered for that item, regardless of how many agencies adhere.
pub fn global_cap(registered_qty: u64) -> Option<u64> {
    registered_qty.checked_mul(2)
}

/// Snapshot of everything needed to decide whether an adhesion is lawful.
#[derive(Debug, Clone, Copy)]
pub struct AdhesionCheck {
    pub registered_qty: u64,
    pub item_adhered_total: u64,
    pub agency_consumed: u64,
    pub requested_qty: u64,
    pub global_cap_exempt: bool,
    pub manager_sphere: Sphere,
    pub adherent_sphere: Sphere,
    pub adherent_is_manager: bool,
    pub now: i64,
    pub valid_from: i64,
    pub valid_until: i64,
}

/// Returns the new (agency_consumed, item_adhered_total) if the adhesion is lawful.
pub fn check_adhesion(c: &AdhesionCheck) -> Result<(u64, u64), RuleViolation> {
    if c.requested_qty == 0 {
        return Err(RuleViolation::InvalidQuantity);
    }
    if c.adherent_is_manager {
        return Err(RuleViolation::ManagerCannotAdhere);
    }
    if c.now < c.valid_from || c.now > c.valid_until {
        return Err(RuleViolation::AtaNotInForce);
    }
    // §8: federal bodies may not adhere to non-federal price records.
    if c.adherent_sphere == Sphere::Federal && c.manager_sphere != Sphere::Federal {
        return Err(RuleViolation::FederalAdhesionForbidden);
    }
    // §4: individual cap. Applies even when the item is exempt from §5.
    let new_agency = c
        .agency_consumed
        .checked_add(c.requested_qty)
        .ok_or(RuleViolation::Overflow)?;
    if new_agency > individual_cap(c.registered_qty) {
        return Err(RuleViolation::ExceedsIndividualCap);
    }
    // §5: global cap, unless a statutory exception applies (§§6-7).
    let new_total = c
        .item_adhered_total
        .checked_add(c.requested_qty)
        .ok_or(RuleViolation::Overflow)?;
    if !c.global_cap_exempt {
        let cap = global_cap(c.registered_qty).ok_or(RuleViolation::Overflow)?;
        if new_total > cap {
            return Err(RuleViolation::ExceedsGlobalCap);
        }
    }
    Ok((new_agency, new_total))
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

    fn base() -> AdhesionCheck {
        AdhesionCheck {
            registered_qty: 100,
            item_adhered_total: 0,
            agency_consumed: 0,
            requested_qty: 10,
            global_cap_exempt: false,
            manager_sphere: Sphere::State,
            adherent_sphere: Sphere::Municipal,
            adherent_is_manager: false,
            now: 1_000,
            valid_from: 0,
            valid_until: 2_000,
        }
    }

    #[test]
    fn art86_par4_individual_cap_is_half_rounded_down() {
        assert_eq!(individual_cap(100), 50);
        assert_eq!(individual_cap(75), 37);
        assert_eq!(individual_cap(1), 0);
        let ok = AdhesionCheck { requested_qty: 50, ..base() };
        assert_eq!(check_adhesion(&ok), Ok((50, 50)));
        let over = AdhesionCheck { requested_qty: 51, ..base() };
        assert_eq!(check_adhesion(&over), Err(RuleViolation::ExceedsIndividualCap));
        let cumulative = AdhesionCheck { agency_consumed: 45, requested_qty: 6, ..base() };
        assert_eq!(check_adhesion(&cumulative), Err(RuleViolation::ExceedsIndividualCap));
    }

    #[test]
    fn art86_par5_global_cap_is_twice_registered() {
        let at_cap = AdhesionCheck { item_adhered_total: 150, requested_qty: 50, ..base() };
        assert_eq!(check_adhesion(&at_cap), Ok((50, 200)));
        let over = AdhesionCheck { item_adhered_total: 151, requested_qty: 50, ..base() };
        assert_eq!(check_adhesion(&over), Err(RuleViolation::ExceedsGlobalCap));
    }

    #[test]
    fn art86_exception_lifts_par5_but_not_par4() {
        let exempt = AdhesionCheck {
            item_adhered_total: 500,
            requested_qty: 50,
            global_cap_exempt: true,
            ..base()
        };
        assert_eq!(check_adhesion(&exempt), Ok((50, 550)));
        let still_par4 = AdhesionCheck { requested_qty: 51, ..exempt };
        assert_eq!(check_adhesion(&still_par4), Err(RuleViolation::ExceedsIndividualCap));
    }

    #[test]
    fn art86_par8_federal_cannot_adhere_to_non_federal() {
        for manager in [Sphere::State, Sphere::District, Sphere::Municipal] {
            let c = AdhesionCheck {
                manager_sphere: manager,
                adherent_sphere: Sphere::Federal,
                ..base()
            };
            assert_eq!(check_adhesion(&c), Err(RuleViolation::FederalAdhesionForbidden));
        }
        let fed_to_fed = AdhesionCheck {
            manager_sphere: Sphere::Federal,
            adherent_sphere: Sphere::Federal,
            ..base()
        };
        assert!(check_adhesion(&fed_to_fed).is_ok());
        let muni_to_fed = AdhesionCheck {
            manager_sphere: Sphere::Federal,
            adherent_sphere: Sphere::Municipal,
            ..base()
        };
        assert!(check_adhesion(&muni_to_fed).is_ok());
    }

    #[test]
    fn validity_window_and_basic_guards() {
        assert_eq!(
            check_adhesion(&AdhesionCheck { now: 2_001, ..base() }),
            Err(RuleViolation::AtaNotInForce)
        );
        assert_eq!(
            check_adhesion(&AdhesionCheck { now: -1, ..base() }),
            Err(RuleViolation::AtaNotInForce)
        );
        assert_eq!(
            check_adhesion(&AdhesionCheck { requested_qty: 0, ..base() }),
            Err(RuleViolation::InvalidQuantity)
        );
        assert_eq!(
            check_adhesion(&AdhesionCheck { adherent_is_manager: true, ..base() }),
            Err(RuleViolation::ManagerCannotAdhere)
        );
        assert_eq!(
            check_adhesion(&AdhesionCheck { agency_consumed: u64::MAX, ..base() }),
            Err(RuleViolation::Overflow)
        );
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

    /// Randomized check: no sequence of accepted adhesions ever breaks §4 or §5.
    #[test]
    fn randomized_adhesion_sequences_never_break_caps() {
        let mut rng = Lcg(42);
        for _ in 0..2_000 {
            let registered = 1 + rng.below(500);
            let agencies = 1 + rng.below(8) as usize;
            let mut consumed = vec![0u64; agencies];
            let mut total = 0u64;
            for _ in 0..40 {
                let a = rng.below(agencies as u64) as usize;
                let c = AdhesionCheck {
                    registered_qty: registered,
                    item_adhered_total: total,
                    agency_consumed: consumed[a],
                    requested_qty: 1 + rng.below(registered),
                    ..base()
                };
                if let Ok((new_agency, new_total)) = check_adhesion(&c) {
                    consumed[a] = new_agency;
                    total = new_total;
                }
                assert!(consumed[a] <= individual_cap(registered));
                assert!(total <= registered * 2);
                assert_eq!(total, consumed.iter().sum::<u64>());
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
