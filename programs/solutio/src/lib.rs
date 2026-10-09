//! Solutio — verifiable legal limits and single-financing control for public
//! procurement obligations, on Solana.
//!
//! Module 1 (Carona) enforces Law 14.133/2021 art. 86 adhesion caps, following
//! the procedure of Decree 11.462/2023 arts. 31-33.
//! Module 2 (Obligations) tracks verified government payment obligations and
//! guarantees, within the protocol, that an eligible balance is never financed twice.
//!
//! All legal and accounting rules live in `rules.rs` as pure, tested functions.

pub mod constants;
pub mod error;
pub mod events;
pub mod instructions;
pub mod rules;
pub mod state;

use anchor_lang::prelude::*;

pub use constants::*;
pub use instructions::*;
pub use state::*;

declare_id!("5cmBDMRdqAfrhmkMmLVxTBCNBHxh5sneyvWXJViyjz9E");

#[program]
pub mod solutio {
    use super::*;

    // ---- registry ----
    pub fn init_registry(ctx: Context<InitRegistry>, eligibility_verifier: Pubkey) -> Result<()> {
        instructions::registry::handle_init_registry(ctx, eligibility_verifier)
    }

    pub fn register_agency(
        ctx: Context<RegisterAgency>,
        sphere: Sphere,
        name: String,
        attributes: AgencyAttributes,
    ) -> Result<()> {
        instructions::registry::handle_register_agency(ctx, sphere, name, attributes)
    }

    // ---- module 1: carona ----
    pub fn create_ata(
        ctx: Context<CreateAta>,
        ata_id: [u8; 32],
        supplier: Pubkey,
        doc_hash: [u8; 32],
        valid_from: i64,
        valid_until: i64,
    ) -> Result<()> {
        instructions::carona::handle_create_ata(ctx, ata_id, supplier, doc_hash, valid_from, valid_until)
    }

    pub fn add_item(
        ctx: Context<AddItem>,
        item_no: u16,
        registered_qty: u64,
        max_adhesion_qty: u64,
        unit_price: u64,
    ) -> Result<()> {
        instructions::carona::handle_add_item(ctx, item_no, registered_qty, max_adhesion_qty, unit_price)
    }

    pub fn set_ata_status(ctx: Context<SetAtaStatus>, status: AtaStatus, evidence_hash: [u8; 32]) -> Result<()> {
        instructions::carona::handle_set_ata_status(ctx, status, evidence_hash)
    }

    pub fn request_adhesion(
        ctx: Context<RequestAdhesion>,
        request_id: u64,
        qty: u64,
        exception: AdhesionException,
        evidence_hash: [u8; 32],
    ) -> Result<()> {
        instructions::carona::handle_request_adhesion(ctx, request_id, qty, exception, evidence_hash)
    }

    pub fn supplier_respond(ctx: Context<SupplierRespond>, accept: bool) -> Result<()> {
        instructions::carona::handle_supplier_respond(ctx, accept)
    }

    pub fn authorize_adhesion(ctx: Context<ManagerDecision>, authorized_qty: u64, evidence_hash: [u8; 32]) -> Result<()> {
        instructions::carona::handle_authorize_adhesion(ctx, authorized_qty, evidence_hash)
    }

    pub fn deny_adhesion(ctx: Context<ManagerDecision>, evidence_hash: [u8; 32]) -> Result<()> {
        instructions::carona::handle_deny_adhesion(ctx, evidence_hash)
    }

    pub fn extend_execution(ctx: Context<ManagerDecision>, new_execute_by: i64) -> Result<()> {
        instructions::carona::handle_extend_execution(ctx, new_execute_by)
    }

    pub fn formalize_adhesion(ctx: Context<FormalizeAdhesion>, evidence_hash: [u8; 32]) -> Result<()> {
        instructions::carona::handle_formalize_adhesion(ctx, evidence_hash)
    }

    pub fn expire_adhesion(ctx: Context<ExpireAdhesion>) -> Result<()> {
        instructions::carona::handle_expire_adhesion(ctx)
    }

    // ---- module 2: obligations ----
    pub fn register_obligation(
        ctx: Context<RegisterObligation>,
        obligation_id: [u8; 32],
        creditor: Pubkey,
        verified_amount: u64,
        evidence_hash: [u8; 32],
    ) -> Result<()> {
        instructions::obligation::handle_register_obligation(ctx, obligation_id, creditor, verified_amount, evidence_hash)
    }

    pub fn attach_fiscal_document(
        ctx: Context<AttachFiscalDocument>,
        doc_key_hash: [u8; 32],
        amount: u64,
    ) -> Result<()> {
        instructions::obligation::handle_attach_fiscal_document(ctx, doc_key_hash, amount)
    }

    pub fn mark_eligible(ctx: Context<MarkEligible>, eligible_amount: u64, evidence_hash: [u8; 32]) -> Result<()> {
        instructions::obligation::handle_mark_eligible(ctx, eligible_amount, evidence_hash)
    }

    pub fn finance(
        ctx: Context<Finance>,
        financing_no: u32,
        amount: u64,
        notice_evidence_hash: [u8; 32],
    ) -> Result<()> {
        instructions::obligation::handle_finance(ctx, financing_no, amount, notice_evidence_hash)
    }

    pub fn record_reduction(ctx: Context<DebtorUpdate>, amount: u64, evidence_hash: [u8; 32]) -> Result<()> {
        instructions::obligation::handle_record_reduction(ctx, amount, evidence_hash)
    }

    pub fn record_payment(ctx: Context<DebtorUpdate>, amount: u64, evidence_hash: [u8; 32]) -> Result<()> {
        instructions::obligation::handle_record_payment(ctx, amount, evidence_hash)
    }
}
