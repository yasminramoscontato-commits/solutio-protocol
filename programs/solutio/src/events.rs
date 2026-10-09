//! Every state transition emits an event, so the full history can be rebuilt
//! from the ledger by anyone, without trusting a Solutio interface.
use anchor_lang::prelude::*;

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
    pub global_cap_exempt: bool,
}

#[event]
pub struct AdhesionRequested {
    pub request: Pubkey,
    pub item: Pubkey,
    pub adherent_agency: Pubkey,
    pub qty: u64,
}

#[event]
pub struct AdhesionApproved {
    pub request: Pubkey,
}

#[event]
pub struct AdhesionRejected {
    pub request: Pubkey,
}

#[event]
pub struct AdhesionEffective {
    pub request: Pubkey,
    pub item: Pubkey,
    pub adherent_agency: Pubkey,
    pub qty: u64,
    pub agency_total: u64,
    pub item_total: u64,
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
