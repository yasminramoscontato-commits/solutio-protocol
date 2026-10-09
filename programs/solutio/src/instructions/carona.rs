//! Module 1 — Carona: price records, items and adhesions with art. 86 caps.
//!
//! Flow: the adherent agency requests -> the managing agency approves -> the
//! supplier accepts. Acceptance is the authoritative moment: the caps are
//! re-checked against the current on-chain totals in the same transaction that
//! updates them, so concurrent requests can never jointly exceed a cap.

use anchor_lang::prelude::*;

use crate::{
    constants::*,
    error::ErrorCode,
    events::*,
    rules::{check_adhesion, AdhesionCheck},
    state::*,
};

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
    let ata = &mut ctx.accounts.ata;
    ata.manager_agency = ctx.accounts.manager_agency.key();
    ata.manager_sphere = ctx.accounts.manager_agency.sphere;
    ata.supplier = supplier;
    ata.ata_id = ata_id;
    ata.doc_hash = doc_hash;
    ata.valid_from = valid_from;
    ata.valid_until = valid_until;
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
    unit_price: u64,
    global_cap_exempt: bool,
) -> Result<()> {
    require!(registered_qty > 0, ErrorCode::InvalidQuantity);
    let item = &mut ctx.accounts.item;
    item.ata = ctx.accounts.ata.key();
    item.item_no = item_no;
    item.registered_qty = registered_qty;
    item.unit_price = unit_price;
    item.adhered_total = 0;
    item.global_cap_exempt = global_cap_exempt;
    item.bump = ctx.bumps.item;
    let ata = &mut ctx.accounts.ata;
    ata.item_count = ata.item_count.checked_add(1).ok_or(ErrorCode::Overflow)?;
    emit!(ItemRegistered {
        item: item.key(),
        registered_qty,
        global_cap_exempt,
    });
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
    #[account(constraint = item.ata == ata.key() @ ErrorCode::AccountMismatch)]
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
    evidence_hash: [u8; 32],
) -> Result<()> {
    require!(evidence_hash != [0u8; 32], ErrorCode::MissingEvidence);
    let now = Clock::get()?.unix_timestamp;
    let ata = &ctx.accounts.ata;
    let item = &ctx.accounts.item;
    let agency = &ctx.accounts.adherent_agency;

    let usage = &mut ctx.accounts.usage;
    if usage.item == Pubkey::default() {
        usage.item = item.key();
        usage.agency = agency.key();
        usage.consumed = 0;
        usage.bump = ctx.bumps.usage;
    }

    // Fail fast against current state. The authoritative check runs again at acceptance.
    check_adhesion(&AdhesionCheck {
        registered_qty: item.registered_qty,
        item_adhered_total: item.adhered_total,
        agency_consumed: usage.consumed,
        requested_qty: qty,
        global_cap_exempt: item.global_cap_exempt,
        manager_sphere: ata.manager_sphere,
        adherent_sphere: agency.sphere,
        adherent_is_manager: agency.key() == ata.manager_agency,
        now,
        valid_from: ata.valid_from,
        valid_until: ata.valid_until,
    })
    .map_err(ErrorCode::from)?;

    let request = &mut ctx.accounts.request;
    request.ata = ata.key();
    request.item = item.key();
    request.adherent_agency = agency.key();
    request.supplier = ata.supplier;
    request.request_id = request_id;
    request.qty = qty;
    request.status = RequestStatus::Requested;
    request.evidence_hash = evidence_hash;
    request.requested_at = now;
    request.decided_at = 0;
    request.bump = ctx.bumps.request;
    emit!(AdhesionRequested {
        request: request.key(),
        item: item.key(),
        adherent_agency: agency.key(),
        qty,
    });
    Ok(())
}

// ---------------------------------------------------------------- approve / reject (manager)

#[derive(Accounts)]
pub struct DecideAdhesion<'info> {
    pub manager_authority: Signer<'info>,
    #[account(
        seeds = [AGENCY_SEED, manager_authority.key().as_ref()],
        bump = manager_agency.bump,
        constraint = manager_agency.active @ ErrorCode::AgencyInactive
    )]
    pub manager_agency: Account<'info, Agency>,
    #[account(constraint = ata.manager_agency == manager_agency.key() @ ErrorCode::Unauthorized)]
    pub ata: Account<'info, Ata>,
    #[account(mut, constraint = request.ata == ata.key() @ ErrorCode::AccountMismatch)]
    pub request: Account<'info, AdhesionRequest>,
}

pub fn handle_approve_adhesion(ctx: Context<DecideAdhesion>) -> Result<()> {
    let request = &mut ctx.accounts.request;
    require!(request.status == RequestStatus::Requested, ErrorCode::InvalidRequestStatus);
    request.status = RequestStatus::Approved;
    request.decided_at = Clock::get()?.unix_timestamp;
    emit!(AdhesionApproved { request: request.key() });
    Ok(())
}

pub fn handle_reject_adhesion(ctx: Context<DecideAdhesion>) -> Result<()> {
    let request = &mut ctx.accounts.request;
    require!(
        request.status == RequestStatus::Requested || request.status == RequestStatus::Approved,
        ErrorCode::InvalidRequestStatus
    );
    request.status = RequestStatus::Rejected;
    request.decided_at = Clock::get()?.unix_timestamp;
    emit!(AdhesionRejected { request: request.key() });
    Ok(())
}

// ---------------------------------------------------------------- accept_adhesion (supplier)

#[derive(Accounts)]
pub struct AcceptAdhesion<'info> {
    pub supplier: Signer<'info>,
    #[account(constraint = ata.supplier == supplier.key() @ ErrorCode::Unauthorized)]
    pub ata: Account<'info, Ata>,
    #[account(mut, constraint = item.ata == ata.key() @ ErrorCode::AccountMismatch)]
    pub item: Account<'info, AtaItem>,
    #[account(constraint = adherent_agency.key() == request.adherent_agency @ ErrorCode::AccountMismatch)]
    pub adherent_agency: Account<'info, Agency>,
    #[account(
        mut,
        seeds = [USAGE_SEED, item.key().as_ref(), adherent_agency.key().as_ref()],
        bump = usage.bump
    )]
    pub usage: Account<'info, AgencyItemUsage>,
    #[account(
        mut,
        constraint = request.item == item.key() @ ErrorCode::AccountMismatch,
        constraint = request.ata == ata.key() @ ErrorCode::AccountMismatch
    )]
    pub request: Account<'info, AdhesionRequest>,
}

pub fn handle_accept_adhesion(ctx: Context<AcceptAdhesion>) -> Result<()> {
    let now = Clock::get()?.unix_timestamp;
    let ata = &ctx.accounts.ata;
    let agency = &ctx.accounts.adherent_agency;
    let request = &mut ctx.accounts.request;
    require!(request.status == RequestStatus::Approved, ErrorCode::InvalidRequestStatus);

    let item = &mut ctx.accounts.item;
    let usage = &mut ctx.accounts.usage;

    // Authoritative check against the totals as they are right now. Because this
    // transaction writes to `item`, Solana serializes it with any other
    // transaction writing to the same item, so the check cannot be raced.
    let (new_agency_total, new_item_total) = check_adhesion(&AdhesionCheck {
        registered_qty: item.registered_qty,
        item_adhered_total: item.adhered_total,
        agency_consumed: usage.consumed,
        requested_qty: request.qty,
        global_cap_exempt: item.global_cap_exempt,
        manager_sphere: ata.manager_sphere,
        adherent_sphere: agency.sphere,
        adherent_is_manager: agency.key() == ata.manager_agency,
        now,
        valid_from: ata.valid_from,
        valid_until: ata.valid_until,
    })
    .map_err(ErrorCode::from)?;

    usage.consumed = new_agency_total;
    item.adhered_total = new_item_total;
    request.status = RequestStatus::Effective;
    request.decided_at = now;
    emit!(AdhesionEffective {
        request: request.key(),
        item: item.key(),
        adherent_agency: agency.key(),
        qty: request.qty,
        agency_total: new_agency_total,
        item_total: new_item_total,
    });
    Ok(())
}
