//! Module 1 — Carona: price records, items and adhesions.
//!
//! Legal basis: Law 14.133/2021, art. 86, as regulated in the federal sphere by
//! Decree 11.462/2023, arts. 31-33. Flow (art. 31, §1):
//!
//!   request (reserves quantity) -> supplier accepts -> manager authorizes
//!   (fully or partially) -> agency executes within 90 days -> or it lapses.
//!
//! Caps are checked when the request reserves quantity, the same accounting the
//! federal "Gestão de Atas" tool uses (quantities awaiting authorization already
//! count). The reservation writes to the item account, so Solana serializes
//! competing requests for the same item and each sees the updated balance.
//! Every later step can only keep or release quantity, never add to it.

use anchor_lang::prelude::*;

use crate::{
    constants::*,
    error::ErrorCode,
    events::*,
    rules::{adhesion_cap, check_reservation, execution_deadline, ReserveCheck},
    state::*,
};

fn release(
    item: &mut AtaItem,
    usage: &mut AgencyItemUsage,
    exception: AdhesionException,
    qty: u64,
) -> Result<()> {
    usage.committed = usage.committed.checked_sub(qty).ok_or(ErrorCode::Overflow)?;
    if exception == AdhesionException::None {
        item.committed_capped = item.committed_capped.checked_sub(qty).ok_or(ErrorCode::Overflow)?;
    } else {
        item.committed_exempt = item.committed_exempt.checked_sub(qty).ok_or(ErrorCode::Overflow)?;
    }
    Ok(())
}

// ---------------------------------------------------------------- create_ata

#[derive(Accounts)]
#[instruction(ata_id: [u8; 32])]
pub struct CreateAta<'info> {
    #[account(mut)]
    pub payer: Signer<'info>,
    pub manager_authority: Signer<'info>,
    #[account(
        seeds = [AGENCY_SEED, manager_authority.key().as_ref()],
        bump = manager_agency.bump,
        constraint = manager_agency.active @ ErrorCode::AgencyInactive
    )]
    pub manager_agency: Account<'info, Agency>,
    #[account(
        init,
        payer = payer,
        space = 8 + Ata::INIT_SPACE,
        seeds = [ATA_SEED, manager_agency.key().as_ref(), ata_id.as_ref()],
        bump
    )]
    pub ata: Account<'info, Ata>,
    pub system_program: Program<'info, System>,
}

pub fn handle_create_ata(
    ctx: Context<CreateAta>,
    ata_id: [u8; 32],
    supplier: Pubkey,
    doc_hash: [u8; 32],
    valid_from: i64,
    valid_until: i64,
) -> Result<()> {
    require!(valid_until > valid_from, ErrorCode::InvalidValidity);
    require!(doc_hash != [0u8; 32], ErrorCode::MissingEvidence);
    let manager = &ctx.accounts.manager_agency;
    let ata = &mut ctx.accounts.ata;
    ata.manager_agency = manager.key();
    ata.manager_sphere = manager.sphere;
    ata.manager_is_health_ministry = manager.is_health_ministry;
    ata.manager_is_state_capital = manager.is_state_capital;
    ata.supplier = supplier;
    ata.ata_id = ata_id;
    ata.doc_hash = doc_hash;
    ata.valid_from = valid_from;
    ata.valid_until = valid_until;
    ata.status = AtaStatus::Active;
    ata.status_evidence_hash = [0u8; 32];
    ata.item_count = 0;
    ata.bump = ctx.bumps.ata;
    emit!(AtaCreated {
        ata: ata.key(),
        manager_agency: ata.manager_agency,
        supplier,
    });
    Ok(())
}

// ---------------------------------------------------------------- add_item

#[derive(Accounts)]
#[instruction(item_no: u16)]
pub struct AddItem<'info> {
    #[account(mut)]
    pub payer: Signer<'info>,
    pub manager_authority: Signer<'info>,
    #[account(
        seeds = [AGENCY_SEED, manager_authority.key().as_ref()],
        bump = manager_agency.bump,
        constraint = manager_agency.active @ ErrorCode::AgencyInactive
    )]
    pub manager_agency: Account<'info, Agency>,
    #[account(
        mut,
        constraint = ata.manager_agency == manager_agency.key() @ ErrorCode::Unauthorized
    )]
    pub ata: Account<'info, Ata>,
    #[account(
        init,
        payer = payer,
        space = 8 + AtaItem::INIT_SPACE,
        seeds = [ITEM_SEED, ata.key().as_ref(), item_no.to_le_bytes().as_ref()],
        bump
    )]
    pub item: Account<'info, AtaItem>,
    pub system_program: Program<'info, System>,
}

pub fn handle_add_item(
    ctx: Context<AddItem>,
    item_no: u16,
    registered_qty: u64,
    max_adhesion_qty: u64,
    unit_price: u64,
) -> Result<()> {
    require!(registered_qty > 0, ErrorCode::InvalidQuantity);
    // The tender may allow fewer adhesions than the statute, never more.
    let statutory = registered_qty.checked_mul(2).ok_or(ErrorCode::Overflow)?;
    require!(max_adhesion_qty <= statutory, ErrorCode::InvalidMaxAdhesion);
    debug_assert_eq!(adhesion_cap(registered_qty, max_adhesion_qty), Some(max_adhesion_qty));
    let item = &mut ctx.accounts.item;
    item.ata = ctx.accounts.ata.key();
    item.item_no = item_no;
    item.registered_qty = registered_qty;
    item.max_adhesion_qty = max_adhesion_qty;
    item.unit_price = unit_price;
    item.committed_capped = 0;
    item.committed_exempt = 0;
    item.authorized_total = 0;
    item.bump = ctx.bumps.item;
    let ata = &mut ctx.accounts.ata;
    ata.item_count = ata.item_count.checked_add(1).ok_or(ErrorCode::Overflow)?;
    emit!(ItemRegistered {
        item: item.key(),
        registered_qty,
        max_adhesion_qty,
    });
    Ok(())
}

// ---------------------------------------------------------------- set_ata_status

#[derive(Accounts)]
pub struct SetAtaStatus<'info> {
    pub manager_authority: Signer<'info>,
    #[account(
        seeds = [AGENCY_SEED, manager_authority.key().as_ref()],
        bump = manager_agency.bump
    )]
    pub manager_agency: Account<'info, Agency>,
    #[account(
        mut,
        constraint = ata.manager_agency == manager_agency.key() @ ErrorCode::Unauthorized
    )]
    pub ata: Account<'info, Ata>,
}

pub fn handle_set_ata_status(ctx: Context<SetAtaStatus>, status: AtaStatus, evidence_hash: [u8; 32]) -> Result<()> {
    require!(evidence_hash != [0u8; 32], ErrorCode::MissingEvidence);
    let ata = &mut ctx.accounts.ata;
    // Cancellation is final.
    require!(ata.status != AtaStatus::Cancelled, ErrorCode::AtaNotActive);
    ata.status = status;
    ata.status_evidence_hash = evidence_hash;
    emit!(AtaStatusChanged { ata: ata.key(), status });
    Ok(())
}

// ---------------------------------------------------------------- request_adhesion

#[derive(Accounts)]
#[instruction(request_id: u64)]
pub struct RequestAdhesion<'info> {
    /// Pays rent and fees. May be a sponsor: the agency signer needs no SOL.
    #[account(mut)]
    pub payer: Signer<'info>,
    pub adherent_authority: Signer<'info>,
    #[account(
        seeds = [AGENCY_SEED, adherent_authority.key().as_ref()],
        bump = adherent_agency.bump,
        constraint = adherent_agency.active @ ErrorCode::AgencyInactive
    )]
    pub adherent_agency: Account<'info, Agency>,
    pub ata: Account<'info, Ata>,
    #[account(mut, constraint = item.ata == ata.key() @ ErrorCode::AccountMismatch)]
    pub item: Account<'info, AtaItem>,
    #[account(
        init_if_needed,
        payer = payer,
        space = 8 + AgencyItemUsage::INIT_SPACE,
        seeds = [USAGE_SEED, item.key().as_ref(), adherent_agency.key().as_ref()],
        bump
    )]
    pub usage: Account<'info, AgencyItemUsage>,
    #[account(
        init,
        payer = payer,
        space = 8 + AdhesionRequest::INIT_SPACE,
        seeds = [
            REQUEST_SEED,
            item.key().as_ref(),
            adherent_agency.key().as_ref(),
            request_id.to_le_bytes().as_ref()
        ],
        bump
    )]
    pub request: Account<'info, AdhesionRequest>,
    pub system_program: Program<'info, System>,
}

pub fn handle_request_adhesion(
    ctx: Context<RequestAdhesion>,
    request_id: u64,
    qty: u64,
    exception: AdhesionException,
    evidence_hash: [u8; 32],
) -> Result<()> {
    require!(evidence_hash != [0u8; 32], ErrorCode::MissingEvidence);
    let now = Clock::get()?.unix_timestamp;
    let ata = &ctx.accounts.ata;
    let agency = &ctx.accounts.adherent_agency;
    let item = &mut ctx.accounts.item;

    let usage = &mut ctx.accounts.usage;
    if usage.item == Pubkey::default() {
        usage.item = item.key();
        usage.agency = agency.key();
        usage.committed = 0;
        usage.bump = ctx.bumps.usage;
    }

    // Authoritative check: this transaction writes to `item`, so concurrent
    // requests for the same item are serialized and cannot jointly exceed a cap.
    let r = check_reservation(&ReserveCheck {
        registered_qty: item.registered_qty,
        max_adhesion_qty: item.max_adhesion_qty,
        committed_capped: item.committed_capped,
        agency_committed: usage.committed,
        requested_qty: qty,
        exception,
        manager_sphere: ata.manager_sphere,
        manager_is_health_ministry: ata.manager_is_health_ministry,
        manager_is_state_capital: ata.manager_is_state_capital,
        adherent_sphere: agency.sphere,
        adherent_profile: agency.profile,
        adherent_is_manager: agency.key() == ata.manager_agency,
        ata_active: ata.status == AtaStatus::Active,
        now,
        valid_from: ata.valid_from,
        valid_until: ata.valid_until,
    })
    .map_err(ErrorCode::from)?;

    usage.committed = r.agency_committed;
    item.committed_capped = item.committed_capped.checked_add(r.capped_delta).ok_or(ErrorCode::Overflow)?;
    item.committed_exempt = item.committed_exempt.checked_add(r.exempt_delta).ok_or(ErrorCode::Overflow)?;

    let request = &mut ctx.accounts.request;
    request.ata = ata.key();
    request.item = item.key();
    request.adherent_agency = agency.key();
    request.supplier = ata.supplier;
    request.request_id = request_id;
    request.requested_qty = qty;
    request.authorized_qty = 0;
    request.exception = exception;
    request.status = RequestStatus::Requested;
    request.evidence_hash = evidence_hash;
    request.decision_evidence_hash = [0u8; 32];
    request.execution_evidence_hash = [0u8; 32];
    request.requested_at = now;
    request.supplier_decided_at = 0;
    request.authorized_at = 0;
    request.execute_by = 0;
    request.bump = ctx.bumps.request;
    emit!(AdhesionRequested {
        request: request.key(),
        item: item.key(),
        adherent_agency: agency.key(),
        qty,
        exception,
        item_committed_capped: item.committed_capped,
    });
    Ok(())
}

// ---------------------------------------------------------------- supplier_respond

#[derive(Accounts)]
pub struct SupplierRespond<'info> {
    pub supplier: Signer<'info>,
    #[account(constraint = ata.supplier == supplier.key() @ ErrorCode::Unauthorized)]
    pub ata: Account<'info, Ata>,
    #[account(mut, constraint = item.ata == ata.key() @ ErrorCode::AccountMismatch)]
    pub item: Account<'info, AtaItem>,
    #[account(
        mut,
        seeds = [USAGE_SEED, item.key().as_ref(), request.adherent_agency.as_ref()],
        bump = usage.bump
    )]
    pub usage: Account<'info, AgencyItemUsage>,
    #[account(mut, constraint = request.item == item.key() @ ErrorCode::AccountMismatch)]
    pub request: Account<'info, AdhesionRequest>,
}

/// Decree 11.462/2023, art. 31, III and §1: the supplier accepts (or declines)
/// before the manager may authorize. Here acceptance is a signature, not an
/// uploaded document.
pub fn handle_supplier_respond(ctx: Context<SupplierRespond>, accept: bool) -> Result<()> {
    let request = &mut ctx.accounts.request;
    require!(request.status == RequestStatus::Requested, ErrorCode::InvalidRequestStatus);
    request.supplier_decided_at = Clock::get()?.unix_timestamp;
    if accept {
        request.status = RequestStatus::SupplierAccepted;
    } else {
        release(&mut ctx.accounts.item, &mut ctx.accounts.usage, request.exception, request.requested_qty)?;
        request.status = RequestStatus::Rejected;
    }
    emit!(SupplierResponded { request: request.key(), accepted: accept });
    Ok(())
}

// ---------------------------------------------------------------- manager decisions

#[derive(Accounts)]
pub struct ManagerDecision<'info> {
    pub manager_authority: Signer<'info>,
    #[account(
        seeds = [AGENCY_SEED, manager_authority.key().as_ref()],
        bump = manager_agency.bump,
        constraint = manager_agency.active @ ErrorCode::AgencyInactive
    )]
    pub manager_agency: Account<'info, Agency>,
    #[account(constraint = ata.manager_agency == manager_agency.key() @ ErrorCode::Unauthorized)]
    pub ata: Account<'info, Ata>,
    #[account(mut, constraint = item.ata == ata.key() @ ErrorCode::AccountMismatch)]
    pub item: Account<'info, AtaItem>,
    #[account(
        mut,
        seeds = [USAGE_SEED, item.key().as_ref(), request.adherent_agency.as_ref()],
        bump = usage.bump
    )]
    pub usage: Account<'info, AgencyItemUsage>,
    #[account(mut, constraint = request.item == item.key() @ ErrorCode::AccountMismatch)]
    pub request: Account<'info, AdhesionRequest>,
}

/// Authorizes all or part of the requested quantity (the federal tool allows
/// "aceitar parcialmente" with a justification). The unauthorized remainder is
/// released. Sets the 90-day execution deadline (art. 31, §2).
pub fn handle_authorize_adhesion(
    ctx: Context<ManagerDecision>,
    authorized_qty: u64,
    evidence_hash: [u8; 32],
) -> Result<()> {
    let now = Clock::get()?.unix_timestamp;
    let ata = &ctx.accounts.ata;
    require!(ata.status == AtaStatus::Active, ErrorCode::AtaNotActive);
    require!(now >= ata.valid_from && now <= ata.valid_until, ErrorCode::AtaNotInForce);
    let request = &mut ctx.accounts.request;
    require!(request.status == RequestStatus::SupplierAccepted, ErrorCode::InvalidRequestStatus);
    require!(
        authorized_qty > 0 && authorized_qty <= request.requested_qty,
        ErrorCode::InvalidQuantity
    );
    if authorized_qty < request.requested_qty {
        require!(evidence_hash != [0u8; 32], ErrorCode::MissingEvidence);
        let remainder = request.requested_qty - authorized_qty;
        release(&mut ctx.accounts.item, &mut ctx.accounts.usage, request.exception, remainder)?;
    }
    let item = &mut ctx.accounts.item;
    item.authorized_total = item.authorized_total.checked_add(authorized_qty).ok_or(ErrorCode::Overflow)?;
    request.authorized_qty = authorized_qty;
    request.status = RequestStatus::Authorized;
    request.decision_evidence_hash = evidence_hash;
    request.authorized_at = now;
    request.execute_by = execution_deadline(now, ata.valid_until);
    emit!(AdhesionAuthorized {
        request: request.key(),
        authorized_qty,
        execute_by: request.execute_by,
    });
    Ok(())
}

pub fn handle_deny_adhesion(ctx: Context<ManagerDecision>, evidence_hash: [u8; 32]) -> Result<()> {
    require!(evidence_hash != [0u8; 32], ErrorCode::MissingEvidence);
    let request = &mut ctx.accounts.request;
    require!(
        matches!(request.status, RequestStatus::Requested | RequestStatus::SupplierAccepted),
        ErrorCode::InvalidRequestStatus
    );
    release(&mut ctx.accounts.item, &mut ctx.accounts.usage, request.exception, request.requested_qty)?;
    request.status = RequestStatus::Rejected;
    request.decision_evidence_hash = evidence_hash;
    emit!(AdhesionDenied { request: request.key() });
    Ok(())
}

/// Art. 31, §3: the manager may exceptionally extend the execution deadline,
/// never beyond the record's validity.
pub fn handle_extend_execution(ctx: Context<ManagerDecision>, new_execute_by: i64) -> Result<()> {
    let ata = &ctx.accounts.ata;
    let request = &mut ctx.accounts.request;
    require!(request.status == RequestStatus::Authorized, ErrorCode::InvalidRequestStatus);
    require!(
        new_execute_by > request.execute_by && new_execute_by <= ata.valid_until,
        ErrorCode::InvalidDeadline
    );
    request.execute_by = new_execute_by;
    emit!(ExecutionExtended { request: request.key(), execute_by: new_execute_by });
    Ok(())
}

// ---------------------------------------------------------------- formalize (adherent)

#[derive(Accounts)]
pub struct FormalizeAdhesion<'info> {
    pub adherent_authority: Signer<'info>,
    #[account(
        seeds = [AGENCY_SEED, adherent_authority.key().as_ref()],
        bump = adherent_agency.bump,
        constraint = adherent_agency.key() == request.adherent_agency @ ErrorCode::Unauthorized
    )]
    pub adherent_agency: Account<'info, Agency>,
    #[account(mut)]
    pub request: Account<'info, AdhesionRequest>,
}

/// Records that the purchase was executed (contract or commitment note,
/// Decree art. 34) within the deadline.
pub fn handle_formalize_adhesion(ctx: Context<FormalizeAdhesion>, evidence_hash: [u8; 32]) -> Result<()> {
    require!(evidence_hash != [0u8; 32], ErrorCode::MissingEvidence);
    let now = Clock::get()?.unix_timestamp;
    let request = &mut ctx.accounts.request;
    require!(request.status == RequestStatus::Authorized, ErrorCode::InvalidRequestStatus);
    require!(now <= request.execute_by, ErrorCode::ExecutionDeadlinePassed);
    request.status = RequestStatus::Executed;
    request.execution_evidence_hash = evidence_hash;
    emit!(AdhesionExecuted { request: request.key() });
    Ok(())
}

// ---------------------------------------------------------------- expire (permissionless)

#[derive(Accounts)]
pub struct ExpireAdhesion<'info> {
    #[account(mut, constraint = item.key() == request.item @ ErrorCode::AccountMismatch)]
    pub item: Account<'info, AtaItem>,
    #[account(
        mut,
        seeds = [USAGE_SEED, item.key().as_ref(), request.adherent_agency.as_ref()],
        bump = usage.bump
    )]
    pub usage: Account<'info, AgencyItemUsage>,
    #[account(mut)]
    pub request: Account<'info, AdhesionRequest>,
}

/// Anyone may lapse an authorized adhesion that was not executed in time,
/// releasing its quantity back to the item. No authority is trusted for this.
pub fn handle_expire_adhesion(ctx: Context<ExpireAdhesion>) -> Result<()> {
    let now = Clock::get()?.unix_timestamp;
    let request = &mut ctx.accounts.request;
    require!(request.status == RequestStatus::Authorized, ErrorCode::InvalidRequestStatus);
    require!(now > request.execute_by, ErrorCode::ExecutionDeadlineNotReached);
    let qty = request.authorized_qty;
    release(&mut ctx.accounts.item, &mut ctx.accounts.usage, request.exception, qty)?;
    let item = &mut ctx.accounts.item;
    item.authorized_total = item.authorized_total.checked_sub(qty).ok_or(ErrorCode::Overflow)?;
    request.status = RequestStatus::Lapsed;
    emit!(AdhesionLapsed { request: request.key(), released_qty: qty });
    Ok(())
}
