use anchor_lang::prelude::*;

use crate::rules::{LedgerViolation, RuleViolation};

#[error_code]
pub enum ErrorCode {
    // ---- identity / authorization ----
    #[msg("Signer is not authorized for this action")]
    Unauthorized,
    #[msg("Agency is not active in the registry")]
    AgencyInactive,
    #[msg("Supplied account does not belong to this record")]
    AccountMismatch,

    // ---- carona (Law 14.133/2021, art. 86) ----
    #[msg("Quantity must be greater than zero")]
    InvalidQuantity,
    #[msg("Art. 86 §4: adhesion would exceed 50% of the registered quantity for this agency")]
    ExceedsIndividualCap,
    #[msg("Art. 86 §5: adhesions would exceed twice the registered quantity of this item")]
    ExceedsGlobalCap,
    #[msg("Art. 86 §8: federal agencies cannot adhere to state, district or municipal price records")]
    FederalAdhesionForbidden,
    #[msg("Price record is not in force at this time")]
    AtaNotInForce,
    #[msg("The managing agency cannot adhere to its own price record")]
    ManagerCannotAdhere,
    #[msg("Adhesion request is not in the required status")]
    InvalidRequestStatus,
    #[msg("Invalid validity window")]
    InvalidValidity,
    #[msg("Price record is suspended or cancelled")]
    AtaNotActive,
    #[msg("The tender does not allow adhesions to this item")]
    AdhesionsNotAllowed,
    #[msg("The claimed exception to the art. 86 §5 cap does not apply")]
    ExceptionNotApplicable,
    #[msg("Maximum quantity for adhesions cannot exceed twice the registered quantity")]
    InvalidMaxAdhesion,
    #[msg("Decree 11.462/2023 art. 31 §2: the execution deadline has passed")]
    ExecutionDeadlinePassed,
    #[msg("The execution deadline has not passed yet")]
    ExecutionDeadlineNotReached,
    #[msg("Invalid execution deadline")]
    InvalidDeadline,
    #[msg("Alagoas Decree 95.019/2023 art. 33: state agencies cannot adhere to municipal records other than those of state capitals")]
    MunicipalAdhesionForbidden,

    // ---- obligations / financing ----
    #[msg("Amount must be greater than zero")]
    InvalidAmount,
    #[msg("Amount exceeds the verified (liquidated) amount of the obligation")]
    ExceedsVerifiedAmount,
    #[msg("Eligible amount cannot exceed the amount backed by attached fiscal documents")]
    ExceedsDocumentedAmount,
    #[msg("Eligible amount cannot be set below the amount already financed")]
    EligibilityBelowFinanced,
    #[msg("Financing would exceed the obligation's remaining financeable balance")]
    ExceedsFinanceableBalance,
    #[msg("Obligation is not open for financing in its current status")]
    ObligationNotFinanceable,
    #[msg("Amount exceeds the obligation's outstanding balance")]
    ExceedsOutstanding,
    #[msg("Evidence hash is required")]
    MissingEvidence,
    #[msg("Source adhesion has not been executed")]
    SourceAdhesionNotEffective,
    #[msg("Source adhesion does not match this debtor or creditor")]
    SourceAdhesionMismatch,

    #[msg("Arithmetic overflow")]
    Overflow,

    // ---- appended after the first devnet deployment, so earlier codes keep their numbers ----
    #[msg("Agency name is longer than 64 bytes")]
    NameTooLong,
    #[msg("Obligations citing this adhesion would exceed its authorized value")]
    ExceedsAdhesionValue,
    #[msg("Only terminal records can be closed")]
    NotClosable,
    #[msg("Close every financing of this obligation first")]
    OpenFinancings,
    #[msg("The response window of this pending request has not passed yet")]
    ResponseWindowOpen,
}

impl From<RuleViolation> for ErrorCode {
    fn from(v: RuleViolation) -> Self {
        match v {
            RuleViolation::InvalidQuantity => ErrorCode::InvalidQuantity,
            RuleViolation::ExceedsIndividualCap => ErrorCode::ExceedsIndividualCap,
            RuleViolation::ExceedsGlobalCap => ErrorCode::ExceedsGlobalCap,
            RuleViolation::FederalAdhesionForbidden => ErrorCode::FederalAdhesionForbidden,
            RuleViolation::AtaNotInForce => ErrorCode::AtaNotInForce,
            RuleViolation::AtaNotActive => ErrorCode::AtaNotActive,
            RuleViolation::AdhesionsNotAllowed => ErrorCode::AdhesionsNotAllowed,
            RuleViolation::ExceptionNotApplicable => ErrorCode::ExceptionNotApplicable,
            RuleViolation::ManagerCannotAdhere => ErrorCode::ManagerCannotAdhere,
            RuleViolation::MunicipalAdhesionForbidden => ErrorCode::MunicipalAdhesionForbidden,
            RuleViolation::Overflow => ErrorCode::Overflow,
        }
    }
}

impl From<LedgerViolation> for ErrorCode {
    fn from(v: LedgerViolation) -> Self {
        match v {
            LedgerViolation::InvalidAmount => ErrorCode::InvalidAmount,
            LedgerViolation::ExceedsVerified => ErrorCode::ExceedsVerifiedAmount,
            LedgerViolation::ExceedsDocumented => ErrorCode::ExceedsDocumentedAmount,
            LedgerViolation::EligibilityBelowFinanced => ErrorCode::EligibilityBelowFinanced,
            LedgerViolation::ExceedsFinanceable => ErrorCode::ExceedsFinanceableBalance,
            LedgerViolation::NotFinanceable => ErrorCode::ObligationNotFinanceable,
            LedgerViolation::ExceedsOutstanding => ErrorCode::ExceedsOutstanding,
            LedgerViolation::Overflow => ErrorCode::Overflow,
        }
    }
}
