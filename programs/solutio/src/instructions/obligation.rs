//! Module 2 — Obligations: verified government payment obligations, their
//! eligibility for assignment, and single financing of the eligible balance.
//!
//! The legal/documentary layer (register, attach documents, confirm
//! eligibility, record reductions and payments) is kept separate from the
//! financial act (`finance`). Eligibility is a distinct step signed by a
//! designated verifier; it is never inferred from the debt being verified.

use anchor_lang::prelude::*;

use crate::{constants::*, error::ErrorCode, events::*, rules, state::*};

// ---------------------------------------------------------------- register_obligation

#[derive(Accounts)]
#[instruction(obligation_id: [u8; 32])]
pub struct RegisterObligation<'info> {
    #[account(mut)]
    pub payer: Signer<'info>,
    pub debtor_authority: Signer<'info>,
    #[account(
        seeds = [AGENCY_SEED, debtor_authority.key().as_ref()],
        bump = debtor_agency.bump,
        constraint = debtor_agency.active @ ErrorCode::AgencyInactive
    )]
    pub debtor_agency: Account<'info, Agency>,
    #[account(
        init,
        payer = payer,
        space = 8 + Obligation::INIT_SPACE,
        seeds = [OBLIGATION_SEED, debtor_agency.key().as_ref(), obligation_id.as_ref()],
        bump
    )]
    pub obligation: Account<'info, Obligation>,
    /// Optional: the executed adhesion this obligation derives from.
    pub source_request: Option<Account<'info, AdhesionRequest>>,
    pub system_program: Program<'info, System>,
}

pub fn handle_register_obligation(
    ctx: Context<RegisterObligation>,
    obligation_id: [u8; 32],
    creditor: Pubkey,
    verified_amount: u64,
    evidence_hash: [u8; 32],
) -> Result<()> {
    require!(verified_amount > 0, ErrorCode::InvalidAmount);
    require!(evidence_hash != [0u8; 32], ErrorCode::MissingEvidence);
    let debtor_key = ctx.accounts.debtor_agency.key();

    let source = match &ctx.accounts.source_request {
        Some(req) => {
            require!(req.status == RequestStatus::Executed, ErrorCode::SourceAdhesionNotEffective);
            require!(
                req.adherent_agency == debtor_key && req.supplier == creditor,
                ErrorCode::SourceAdhesionMismatch
            );
            req.key()
        }
        None => Pubkey::default(),
    };

    let o = &mut ctx.accounts.obligation;
    o.debtor_agency = debtor_key;
    o.creditor = creditor;
    o.obligation_id = obligation_id;
    o.verified_amount = verified_amount;
    o.documented_amount = 0;
    o.reductions = 0;
    o.eligible_amount = 0;
    o.financed_amount = 0;
    o.paid_amount = 0;
    o.financing_count = 0;
    o.document_count = 0;
    o.status = ObligationStatus::Verified;
    o.evidence_hash = evidence_hash;
    o.eligibility_evidence_hash = [0u8; 32];
    o.source_request = source;
    o.bump = ctx.bumps.obligation;
    emit!(ObligationRegistered {
        obligation: o.key(),
        debtor_agency: debtor_key,
        creditor,
        verified_amount,
    });
    Ok(())
}

// ---------------------------------------------------------------- attach_fiscal_document

#[derive(Accounts)]
#[instruction(doc_key_hash: [u8; 32])]
pub struct AttachFiscalDocument<'info> {
    #[account(mut)]
    pub payer: Signer<'info>,
    pub debtor_authority: Signer<'info>,
    #[account(
        seeds = [AGENCY_SEED, debtor_authority.key().as_ref()],
        bump = debtor_agency.bump
    )]
    pub debtor_agency: Account<'info, Agency>,
    #[account(
        mut,
        constraint = obligation.debtor_agency == debtor_agency.key() @ ErrorCode::Unauthorized
    )]
    pub obligation: Account<'info, Obligation>,
    /// Derived from the document key hash: a document can back only one obligation.
    #[account(
        init,
        payer = payer,
        space = 8 + FiscalDocument::INIT_SPACE,
        seeds = [FISCAL_DOC_SEED, doc_key_hash.as_ref()],
        bump
    )]
    pub fiscal_document: Account<'info, FiscalDocument>,
    pub system_program: Program<'info, System>,
}

pub fn handle_attach_fiscal_document(
    ctx: Context<AttachFiscalDocument>,
    doc_key_hash: [u8; 32],
    amount: u64,
) -> Result<()> {
    let o = &mut ctx.accounts.obligation;
    require!(o.status != ObligationStatus::Settled, ErrorCode::ObligationNotFinanceable);
    let next = rules::attach_document(o.balances(), amount).map_err(ErrorCode::from)?;
    o.store(next);
    o.document_count = o.document_count.checked_add(1).ok_or(ErrorCode::Overflow)?;

    let doc = &mut ctx.accounts.fiscal_document;
    doc.obligation = o.key();
    doc.doc_key_hash = doc_key_hash;
    doc.amount = amount;
    doc.bump = ctx.bumps.fiscal_document;
    emit!(FiscalDocumentAttached {
        obligation: o.key(),
        fiscal_document: doc.key(),
        amount,
    });
    Ok(())
}

// ---------------------------------------------------------------- mark_eligible

#[derive(Accounts)]
pub struct MarkEligible<'info> {
    pub verifier: Signer<'info>,
    #[account(
        seeds = [REGISTRY_SEED],
        bump = registry.bump,
        constraint = registry.eligibility_verifier == verifier.key() @ ErrorCode::Unauthorized
    )]
    pub registry: Account<'info, Registry>,
    #[account(mut)]
    pub obligation: Account<'info, Obligation>,
}

pub fn handle_mark_eligible(
    ctx: Context<MarkEligible>,
    eligible_amount: u64,
    evidence_hash: [u8; 32],
) -> Result<()> {
    require!(evidence_hash != [0u8; 32], ErrorCode::MissingEvidence);
    let o = &mut ctx.accounts.obligation;
    require!(
        matches!(
            o.status,
            ObligationStatus::Verified | ObligationStatus::Eligible | ObligationStatus::Financed
        ),
        ErrorCode::ObligationNotFinanceable
    );
    let next = rules::mark_eligible(o.balances(), eligible_amount).map_err(ErrorCode::from)?;
    o.store(next);
    o.eligibility_evidence_hash = evidence_hash;
    o.status = if o.financed_amount > 0 {
        ObligationStatus::Financed
    } else {
        ObligationStatus::Eligible
    };
    emit!(EligibilityConfirmed {
        obligation: o.key(),
        eligible_amount,
    });
    Ok(())
}

// ---------------------------------------------------------------- finance

#[derive(Accounts)]
#[instruction(financing_no: u32)]
pub struct Finance<'info> {
    #[account(mut)]
    pub payer: Signer<'info>,
    pub financier: Signer<'info>,
    /// The creditor (supplier) must consent to the assignment.
    pub creditor: Signer<'info>,
    #[account(
        mut,
        constraint = obligation.creditor == creditor.key() @ ErrorCode::Unauthorized,
        constraint = obligation.financing_count == financing_no @ ErrorCode::AccountMismatch
    )]
    pub obligation: Account<'info, Obligation>,
    #[account(
        init,
        payer = payer,
        space = 8 + Financing::INIT_SPACE,
        seeds = [FINANCING_SEED, obligation.key().as_ref(), financing_no.to_le_bytes().as_ref()],
        bump
    )]
    pub financing: Account<'info, Financing>,
    pub system_program: Program<'info, System>,
}

pub fn handle_finance(
    ctx: Context<Finance>,
    financing_no: u32,
    amount: u64,
    notice_evidence_hash: [u8; 32],
) -> Result<()> {
    // Assignment is only effective against the debtor once notified (Civil Code
    // art. 290). We require evidence of that notice; we cannot prove delivery.
    require!(notice_evidence_hash != [0u8; 32], ErrorCode::MissingEvidence);
    let o = &mut ctx.accounts.obligation;
    require!(
        matches!(o.status, ObligationStatus::Eligible | ObligationStatus::Financed),
        ErrorCode::ObligationNotFinanceable
    );
    // This transaction writes to `obligation`, so competing financings of the
    // same obligation are serialized and each one sees the updated balance.
    let next = rules::finance(o.balances(), amount).map_err(ErrorCode::from)?;
    o.store(next);
    o.financing_count = o.financing_count.checked_add(1).ok_or(ErrorCode::Overflow)?;
    o.status = ObligationStatus::Financed;

    let f = &mut ctx.accounts.financing;
    f.obligation = o.key();
    f.financier = ctx.accounts.financier.key();
    f.financing_no = financing_no;
    f.amount = amount;
    f.notice_evidence_hash = notice_evidence_hash;
    f.created_at = Clock::get()?.unix_timestamp;
    f.bump = ctx.bumps.financing;
    emit!(ObligationFinanced {
        obligation: o.key(),
        financing: f.key(),
        financier: f.financier,
        amount,
        financed_total: o.financed_amount,
    });
    Ok(())
}

// ---------------------------------------------------------------- reductions and payments (debtor)

#[derive(Accounts)]
pub struct DebtorUpdate<'info> {
    pub debtor_authority: Signer<'info>,
    #[account(
        seeds = [AGENCY_SEED, debtor_authority.key().as_ref()],
        bump = debtor_agency.bump
    )]
    pub debtor_agency: Account<'info, Agency>,
    #[account(
        mut,
        constraint = obligation.debtor_agency == debtor_agency.key() @ ErrorCode::Unauthorized
    )]
    pub obligation: Account<'info, Obligation>,
}

/// Withholdings and disallowances are acts of the debtor. The protocol records
/// them even when they hurt a financier, and marks the obligation impaired.
pub fn handle_record_reduction(
    ctx: Context<DebtorUpdate>,
    amount: u64,
    evidence_hash: [u8; 32],
) -> Result<()> {
    require!(evidence_hash != [0u8; 32], ErrorCode::MissingEvidence);
    let o = &mut ctx.accounts.obligation;
    require!(o.status != ObligationStatus::Settled, ErrorCode::ExceedsOutstanding);
    let next = rules::apply_reduction(o.balances(), amount).map_err(ErrorCode::from)?;
    o.store(next);
    let impaired = next.is_impaired();
    if impaired {
        o.status = ObligationStatus::Impaired;
    } else if next.outstanding() == Some(0) {
        o.status = ObligationStatus::Settled;
    }
    emit!(ReductionRecorded {
        obligation: o.key(),
        amount,
        impaired,
    });
    Ok(())
}

pub fn handle_record_payment(
    ctx: Context<DebtorUpdate>,
    amount: u64,
    evidence_hash: [u8; 32],
) -> Result<()> {
    require!(evidence_hash != [0u8; 32], ErrorCode::MissingEvidence);
    let o = &mut ctx.accounts.obligation;
    require!(o.status != ObligationStatus::Settled, ErrorCode::ExceedsOutstanding);
    let next = rules::apply_payment(o.balances(), amount).map_err(ErrorCode::from)?;
    o.store(next);
    let settled = next.outstanding() == Some(0);
    if settled {
        o.status = ObligationStatus::Settled;
    }
    emit!(PaymentRecorded {
        obligation: o.key(),
        amount,
        settled,
    });
    Ok(())
}
