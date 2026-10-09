use anchor_lang::prelude::*;

use crate::{constants::*, error::ErrorCode, events::*, state::*};

#[derive(Accounts)]
pub struct InitRegistry<'info> {
    #[account(mut)]
    pub payer: Signer<'info>,
    pub authority: Signer<'info>,
    #[account(
        init,
        payer = payer,
        space = 8 + Registry::INIT_SPACE,
        seeds = [REGISTRY_SEED],
        bump
    )]
    pub registry: Account<'info, Registry>,
    pub system_program: Program<'info, System>,
}

pub fn handle_init_registry(ctx: Context<InitRegistry>, eligibility_verifier: Pubkey) -> Result<()> {
    let registry = &mut ctx.accounts.registry;
    registry.authority = ctx.accounts.authority.key();
    registry.eligibility_verifier = eligibility_verifier;
    registry.agency_count = 0;
    registry.bump = ctx.bumps.registry;
    Ok(())
}

#[derive(Accounts)]
pub struct RegisterAgency<'info> {
    #[account(mut)]
    pub payer: Signer<'info>,
    pub registry_authority: Signer<'info>,
    #[account(
        mut,
        seeds = [REGISTRY_SEED],
        bump = registry.bump,
        constraint = registry.authority == registry_authority.key() @ ErrorCode::Unauthorized
    )]
    pub registry: Account<'info, Registry>,
    /// CHECK: the wallet that will sign for the agency; any key is accepted.
    pub agency_authority: UncheckedAccount<'info>,
    #[account(
        init,
        payer = payer,
        space = 8 + Agency::INIT_SPACE,
        seeds = [AGENCY_SEED, agency_authority.key().as_ref()],
        bump
    )]
    pub agency: Account<'info, Agency>,
    pub system_program: Program<'info, System>,
}

pub fn handle_register_agency(
    ctx: Context<RegisterAgency>,
    sphere: Sphere,
    name: String,
    attributes: AgencyAttributes,
) -> Result<()> {
    require!(name.len() <= 64, ErrorCode::InvalidQuantity);
    let agency = &mut ctx.accounts.agency;
    agency.authority = ctx.accounts.agency_authority.key();
    agency.sphere = sphere;
    agency.is_health_ministry = attributes.is_health_ministry;
    agency.is_state_capital = attributes.is_state_capital;
    agency.profile = attributes.profile;
    agency.name = name;
    agency.active = true;
    agency.bump = ctx.bumps.agency;
    let registry = &mut ctx.accounts.registry;
    registry.agency_count = registry.agency_count.checked_add(1).ok_or(ErrorCode::Overflow)?;
    emit!(AgencyRegistered {
        agency: agency.key(),
        authority: agency.authority,
    });
    Ok(())
}
