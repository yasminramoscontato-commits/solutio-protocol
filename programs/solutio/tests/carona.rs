//! Module 1 — Carona: Law 14.133/2021, art. 86, enforced by the compiled program.

mod common;

use common::*;
use solana_signer::Signer;
use solutio::{error::ErrorCode, state::*};

struct Scenario {
    env: Env,
    manager: solana_keypair::Keypair,
    supplier: solana_keypair::Keypair,
    ata: anchor_lang::prelude::Pubkey,
    item: anchor_lang::prelude::Pubkey,
}

/// A state central purchasing body manages a price record with one item of
/// 100 registered units; the supplier must accept every adhesion.
fn scenario(registered: u64, exempt: bool) -> Scenario {
    let mut env = Env::new();
    let manager = env.new_agency(Sphere::State, "Central de Compras do Estado");
    let supplier = solana_keypair::Keypair::new();
    let ata = env.create_ata(&manager, "ARP-001/2026", &supplier.pubkey(), 365);
    let item = env.add_item(&manager, &ata, 1, registered, exempt);
    Scenario { env, manager, supplier, ata, item }
}

#[test]
fn adhesion_needs_request_approval_and_supplier_acceptance() {
    let Scenario { mut env, manager, supplier, ata, item } = scenario(100, false);
    let city = env.new_agency(Sphere::Municipal, "Prefeitura A");

    let req = env.request(&city, &ata, &item, 1, 30).unwrap();
    // Supplier cannot accept before the manager approves.
    assert_eq!(env.accept(&supplier, &ata, &item, &city, &req), Err(code(ErrorCode::InvalidRequestStatus)));
    // Someone other than the registered supplier cannot accept.
    env.approve(&manager, &ata, &req).unwrap();
    let impostor = solana_keypair::Keypair::new();
    assert_eq!(env.accept(&impostor, &ata, &item, &city, &req), Err(code(ErrorCode::Unauthorized)));
    // The registered supplier accepts: the adhesion becomes effective.
    env.accept(&supplier, &ata, &item, &city, &req).unwrap();

    let r: AdhesionRequest = env.fetch(&req);
    assert_eq!(r.status, RequestStatus::Effective);
    let it: AtaItem = env.fetch(&item);
    assert_eq!(it.adhered_total, 30);
    // An effective request cannot be accepted twice.
    assert_eq!(env.accept(&supplier, &ata, &item, &city, &req), Err(code(ErrorCode::InvalidRequestStatus)));
}

#[test]
fn only_the_managing_agency_can_approve() {
    let Scenario { mut env, ata, item, .. } = scenario(100, false);
    let city = env.new_agency(Sphere::Municipal, "Prefeitura A");
    let other_state = env.new_agency(Sphere::State, "Outro Estado");
    let req = env.request(&city, &ata, &item, 1, 10).unwrap();
    assert_eq!(env.approve(&other_state, &ata, &req), Err(code(ErrorCode::Unauthorized)));
}

#[test]
fn rejected_requests_never_consume_balance() {
    let Scenario { mut env, manager, supplier, ata, item } = scenario(100, false);
    let city = env.new_agency(Sphere::Municipal, "Prefeitura A");
    let req = env.request(&city, &ata, &item, 1, 50).unwrap();
    env.approve(&manager, &ata, &req).unwrap();
    env.reject(&manager, &ata, &req).unwrap();
    assert_eq!(env.accept(&supplier, &ata, &item, &city, &req), Err(code(ErrorCode::InvalidRequestStatus)));
    let it: AtaItem = env.fetch(&item);
    assert_eq!(it.adhered_total, 0);
}

#[test]
fn art86_par4_each_agency_is_capped_at_half_the_registered_quantity() {
    let Scenario { mut env, manager, supplier, ata, item } = scenario(100, false);
    let city = env.new_agency(Sphere::Municipal, "Prefeitura A");
    // 51 of 100 in a single request: rejected up front.
    assert_eq!(env.request(&city, &ata, &item, 1, 51), Err(code(ErrorCode::ExceedsIndividualCap)));
    // 30 + 20 = 50 is lawful; a further unit is not.
    env.full_adhesion(&manager, &supplier, &city, &ata, &item, 2, 30).unwrap();
    env.full_adhesion(&manager, &supplier, &city, &ata, &item, 3, 20).unwrap();
    assert_eq!(env.request(&city, &ata, &item, 4, 1), Err(code(ErrorCode::ExceedsIndividualCap)));
    let usage: AgencyItemUsage = env.fetch(&env.usage_pda(&item, &env.agency(&city.pubkey())));
    assert_eq!(usage.consumed, 50);
}

#[test]
fn art86_par5_total_adhesions_are_capped_at_twice_the_registered_quantity() {
    let Scenario { mut env, manager, supplier, ata, item } = scenario(100, false);
    for (i, name) in ["A", "B", "C", "D"].iter().enumerate() {
        let city = env.new_agency(Sphere::Municipal, name);
        env.full_adhesion(&manager, &supplier, &city, &ata, &item, i as u64, 50).unwrap();
    }
    let it: AtaItem = env.fetch(&item);
    assert_eq!(it.adhered_total, 200);
    let fifth = env.new_agency(Sphere::Municipal, "E");
    assert_eq!(env.request(&fifth, &ata, &item, 9, 1), Err(code(ErrorCode::ExceedsGlobalCap)));
}

/// Two agencies race for the last 50 units. Both requests are lawful when made
/// and both are approved; only the first acceptance can succeed, because the
/// acceptance re-checks the cap against the current total.
#[test]
fn race_for_the_last_units_only_one_can_win() {
    let Scenario { mut env, manager, supplier, ata, item } = scenario(100, false);
    for (i, name) in ["A", "B", "C"].iter().enumerate() {
        let city = env.new_agency(Sphere::Municipal, name);
        env.full_adhesion(&manager, &supplier, &city, &ata, &item, i as u64, 50).unwrap();
    }
    let d = env.new_agency(Sphere::Municipal, "D");
    let e = env.new_agency(Sphere::Municipal, "E");
    let req_d = env.request(&d, &ata, &item, 1, 50).unwrap();
    let req_e = env.request(&e, &ata, &item, 1, 50).unwrap();
    env.approve(&manager, &ata, &req_d).unwrap();
    env.approve(&manager, &ata, &req_e).unwrap();

    env.accept(&supplier, &ata, &item, &d, &req_d).unwrap();
    assert_eq!(env.accept(&supplier, &ata, &item, &e, &req_e), Err(code(ErrorCode::ExceedsGlobalCap)));

    let it: AtaItem = env.fetch(&item);
    assert_eq!(it.adhered_total, 200);
    let r: AdhesionRequest = env.fetch(&req_e);
    assert_eq!(r.status, RequestStatus::Approved, "the losing request stays pending, unchanged");
}

#[test]
fn statutory_exception_lifts_par5_but_par4_still_applies() {
    let Scenario { mut env, manager, supplier, ata, item } = scenario(100, true);
    for i in 0..5u64 {
        let city = env.new_agency(Sphere::Municipal, &format!("Cidade {i}"));
        env.full_adhesion(&manager, &supplier, &city, &ata, &item, i, 50).unwrap();
    }
    let it: AtaItem = env.fetch(&item);
    assert_eq!(it.adhered_total, 250, "above 2x is lawful under the exception");
    let greedy = env.new_agency(Sphere::Municipal, "Cidade 6");
    assert_eq!(env.request(&greedy, &ata, &item, 1, 51), Err(code(ErrorCode::ExceedsIndividualCap)));
}

#[test]
fn art86_par8_federal_agency_cannot_adhere_to_a_state_record() {
    let Scenario { mut env, ata, item, .. } = scenario(100, false);
    let federal = env.new_agency(Sphere::Federal, "Ministerio X");
    assert_eq!(env.request(&federal, &ata, &item, 1, 10), Err(code(ErrorCode::FederalAdhesionForbidden)));
}

#[test]
fn federal_agency_can_adhere_to_a_federal_record() {
    let mut env = Env::new();
    let manager = env.new_agency(Sphere::Federal, "Central de Compras Federal");
    let supplier = solana_keypair::Keypair::new();
    let ata = env.create_ata(&manager, "ARP-FED-1", &supplier.pubkey(), 365);
    let item = env.add_item(&manager, &ata, 1, 100, false);
    let federal = env.new_agency(Sphere::Federal, "Ministerio X");
    env.full_adhesion(&manager, &supplier, &federal, &ata, &item, 1, 10).unwrap();
}

#[test]
fn the_manager_cannot_adhere_to_its_own_record() {
    let Scenario { mut env, manager, ata, item, .. } = scenario(100, false);
    assert_eq!(env.request(&manager, &ata, &item, 1, 10), Err(code(ErrorCode::ManagerCannotAdhere)));
}

#[test]
fn expired_record_rejects_requests_and_late_acceptances() {
    let Scenario { mut env, manager, supplier, ata, item } = scenario(100, false);
    let city = env.new_agency(Sphere::Municipal, "Prefeitura A");
    let req = env.request(&city, &ata, &item, 1, 10).unwrap();
    env.approve(&manager, &ata, &req).unwrap();
    // The record expires before the supplier accepts.
    let later = env.now() + 400 * DAY;
    env.set_time(later);
    assert_eq!(env.accept(&supplier, &ata, &item, &city, &req), Err(code(ErrorCode::AtaNotInForce)));
    assert_eq!(env.request(&city, &ata, &item, 2, 10), Err(code(ErrorCode::AtaNotInForce)));
}

#[test]
fn agency_wallets_sign_without_holding_any_sol() {
    let Scenario { mut env, manager, supplier, ata, item } = scenario(100, false);
    let city = env.new_agency(Sphere::Municipal, "Prefeitura A");
    assert_eq!(env.lamports(&city.pubkey()), 0);
    assert_eq!(env.lamports(&manager.pubkey()), 0);
    env.full_adhesion(&manager, &supplier, &city, &ata, &item, 1, 10).unwrap();
    // Every fee and rent deposit was paid by the sponsor.
    assert_eq!(env.lamports(&city.pubkey()), 0);
    assert_eq!(env.lamports(&manager.pubkey()), 0);
    assert_eq!(env.lamports(&supplier.pubkey()), 0);
}
