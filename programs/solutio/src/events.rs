//! Every state transition emits an event, so the full history can be rebuilt
//! from the ledger by anyone, without trusting a Solutio interface.
use anchor_lang::prelude::*;

use crate::state::{AdhesionException, AtaStatus};

#[event]
pub struct AgencyRegistered {
    pub agency: Pubkey,
    pub authority: Pubkey,
}

#[event]
pub struct AtaCreated {
    pub ata: Pubkey,
    pub manager_agency: Pubkey,
    pub supplier: Pubkey,
}

#[event]
pub struct ItemRegistered {
    pub item: Pubkey,
    pub registered_qty: u64,
    pub max_adhesion_qty: u64,
}

#[event]
pub struct AtaStatusChanged {
    pub ata: Pubkey,
    pub status: AtaStatus,
}

#[event]
pub struct AdhesionRequested {
    pub request: Pubkey,
    pub item: Pubkey,
    pub adherent_agency: Pubkey,
    pub qty: u64,
    pub exception: AdhesionException,
    pub item_committed_capped: u64,
}

#[event]
pub struct SupplierResponded {
    pub request: Pubkey,
    pub accepted: bool,
}

#[event]
pub struct AdhesionAuthorized {
    pub request: Pubkey,
    pub authorized_qty: u64,
    pub execute_by: i64,
}

#[event]
pub struct AdhesionDenied {
    pub request: Pubkey,
}

#[event]
pub struct ExecutionExtended {
    pub request: Pubkey,
    pub execute_by: i64,
}

#[event]
pub struct AdhesionExecuted {
    pub request: Pubkey,
}

#[event]
pub struct AdhesionLapsed {
    pub request: Pubkey,
    pub released_qty: u64,
}

#[event]
pub struct ObligationRegistered {
    pub obligation: Pubkey,
    pub debtor_agency: Pubkey,
    pub creditor: Pubkey,
    pub verified_amount: u64,
}

#[event]
pub struct FiscalDocumentAttached {
    pub obligation: Pubkey,
    pub fiscal_document: Pubkey,
    pub amount: u64,
}

#[event]
pub struct EligibilityConfirmed {
    pub obligation: Pubkey,
    pub eligible_amount: u64,
}

#[event]
pub struct ObligationFinanced {
    pub obligation: Pubkey,
    pub financing: Pubkey,
    pub financier: Pubkey,
    pub amount: u64,
    pub financed_total: u64,
}

#[event]
pub struct ReductionRecorded {
    pub obligation: Pubkey,
    pub amount: u64,
    pub impaired: bool,
}

#[event]
pub struct PaymentRecorded {
    pub obligation: Pubkey,
    pub amount: u64,
    pub settled: bool,
}

/// role: 0 = registry authority, 1 = eligibility verifier.
#[event]
pub struct RegistryKeyChanged {
    pub role: u8,
    pub previous: Pubkey,
    pub current: Pubkey,
}

#[event]
pub struct AgencyStatusChanged {
    pub agency: Pubkey,
    pub active: bool,
}

/// kind: 0 = adhesion request, 1 = financing, 2 = obligation.
#[event]
pub struct AccountClosed {
    pub account: Pubkey,
    pub kind: u8,
    pub refunded_to: Pubkey,
    pub lamports: u64,
}
