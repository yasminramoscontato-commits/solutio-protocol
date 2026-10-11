//! Registry: root of trust and identities.
//!
//! The registry authority is the program's upgrade authority at initialization
//! (audit H-1), and can be rotated only with both the old and the new key
//! signing. It can revoke an agency and replace the eligibility verifier
//! (audit M-1).

use anchor_lang::prelude::*;

use crate::{constants::*, error::ErrorCode, events::*, program::Marjan, state::*};

#[derive(Accounts)]
pub struct InitRegistry<'info> {
    #[account(mut)]
    pub payer: Signer<'info>,
    /// Must be the program's upgrade authority, so nobody can front-run the
    /// initialization of a fresh deployment.
    pub authority: Signer<'info>,
    #[account(constraint = program.programdata_address()? == Some(program_data.key()) @ ErrorCode::AccountMismatch)]
    pub program: Program<'info, Marjan>,
    #[account(constraint = program_data.upgrade_authority_address == Some(authority.key()) @ ErrorCode::Unauthorized)]
    pub program_data: Account<'info, ProgramData>,
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
    require!(name.len() <= 64, ErrorCode::NameTooLong);
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

// ---------------------------------------------------------------- administration

#[derive(Accounts)]
pub struct RegistryAdmin<'info> {
    pub registry_authority: Signer<'info>,
    #[account(
        mut,
        seeds = [REGISTRY_SEED],
        bump = registry.bump,
        constraint = registry.authority == registry_authority.key() @ ErrorCode::Unauthorized
    )]
    pub registry: Account<'info, Registry>,
}

/// Replaces the eligibility verifier (e.g. after a key compromise).
pub fn handle_set_eligibility_verifier(ctx: Context<RegistryAdmin>, new_verifier: Pubkey) -> Result<()> {
    let registry = &mut ctx.accounts.registry;
    let previous = registry.eligibility_verifier;
    registry.eligibility_verifier = new_verifier;
    emit!(RegistryKeyChanged {
        role: 1,
        previous,
        current: new_verifier
    });
    Ok(())
}

#[derive(Accounts)]
pub struct SetRegistryAuthority<'info> {
    pub registry_authority: Signer<'info>,
    /// The new authority signs too, so the root of trust can never be handed to
    /// a key nobody controls.
    pub new_authority: Signer<'info>,
    #[account(
        mut,
        seeds = [REGISTRY_SEED],
        bump = registry.bump,
        constraint = registry.authority == registry_authority.key() @ ErrorCode::Unauthorized
    )]
    pub registry: Account<'info, Registry>,
}

pub fn handle_set_registry_authority(ctx: Context<SetRegistryAuthority>) -> Result<()> {
    let registry = &mut ctx.accounts.registry;
    let previous = registry.authority;
    registry.authority = ctx.accounts.new_authority.key();
    emit!(RegistryKeyChanged {
        role: 0,
        previous,
        current: registry.authority
    });
    Ok(())
}

#[derive(Accounts)]
pub struct SetAgencyActive<'info> {
    pub registry_authority: Signer<'info>,
    #[account(
        seeds = [REGISTRY_SEED],
        bump = registry.bump,
        constraint = registry.authority == registry_authority.key() @ ErrorCode::Unauthorized
    )]
    pub registry: Account<'info, Registry>,
    #[account(mut, seeds = [AGENCY_SEED, agency.authority.as_ref()], bump = agency.bump)]
    pub agency: Account<'info, Agency>,
}

/// Revokes (or restores) an agency. A revoked agency can sign nothing.
pub fn handle_set_agency_active(ctx: Context<SetAgencyActive>, active: bool) -> Result<()> {
    let agency = &mut ctx.accounts.agency;
    agency.active = active;
    emit!(AgencyStatusChanged {
        agency: agency.key(),
        active
    });
    Ok(())
}
