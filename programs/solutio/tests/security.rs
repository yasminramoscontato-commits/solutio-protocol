//! Regression tests for the findings of docs/security-audit-2026-10-09.md.

mod common;

use common::*;
use solana_keypair::Keypair;
use solana_signer::Signer;
use solutio::{constants::RESPONSE_WINDOW_SECS, error::ErrorCode, state::*};

const BRL_10K: u64 = 1_000_000;

/// State record, one item of 100 units, a municipality with an executed adhesion of 8 units (R$ 10.000,00).
struct Executed {
    env: Env,
    city: Keypair,
    supplier: Keypair,
    request: anchor_lang::prelude::Pubkey,
}

fn executed() -> Executed {
    let mut env = Env::new();
    let manager = env.new_agency(Sphere::State, "Central de Compras");
    let supplier = Keypair::new();
    let ata = env.create_ata(&manager, "ARP-SEC", &supplier.pubkey(), 365);
    let item = env.add_item(&manager, &ata, 1, 100, 200);
    let city = env.new_agency(Sphere::Municipal, "Prefeitura A");
    let request = env
        .full_adhesion(&manager, &supplier, &city, &ata, &item, 1, 8)
        .unwrap();
    Executed {
        env,
        city,
        supplier,
        request,
    }
}

#[test]
fn h1_only_the_upgrade_authority_can_initialize_the_registry() {
    let mut env = Env::new_uninitialized();
    let squatter = Keypair::new();
    let v = env.verifier.pubkey();
    assert_eq!(env.init_registry(&squatter, v), Err(code(ErrorCode::Unauthorized)));
    let ra = env.registry_authority.insecure_clone();
    env.init_registry(&ra, v).unwrap();
    let r: Registry = env.fetch(&env.registry());
    assert_eq!(r.authority, ra.pubkey());
}

#[test]
fn m1_the_eligibility_verifier_can_be_replaced() {
    let Executed {
        mut env,
        city,
        supplier,
        request,
    } = executed();
    let ob = env
        .register_obligation(&city, "NE-1", &supplier.pubkey(), BRL_10K, Some(request))
        .unwrap();
    env.attach_doc(&city, &ob, "NFE-1", BRL_10K).unwrap();

    let stranger = Keypair::new();
    assert_eq!(
        env.set_verifier(&stranger, stranger.pubkey()),
        Err(code(ErrorCode::Unauthorized))
    );

    let new_verifier = Keypair::new();
    let ra = env.registry_authority.insecure_clone();
    env.set_verifier(&ra, new_verifier.pubkey()).unwrap();
    // The old (compromised) key no longer works; the new one does.
    assert_eq!(env.mark_eligible(&ob, BRL_10K), Err(code(ErrorCode::Unauthorized)));
    env.mark_eligible_as(&new_verifier, &ob, BRL_10K).unwrap();
}

#[test]
fn m1_registry_authority_rotation_needs_both_keys_and_moves_power() {
    let mut env = Env::new();
    let old = env.registry_authority.insecure_clone();
    let new = Keypair::new();
    env.set_registry_authority(&old, &new).unwrap();
    assert_eq!(env.set_verifier(&old, old.pubkey()), Err(code(ErrorCode::Unauthorized)));
    env.set_verifier(&new, new.pubkey()).unwrap();
}

#[test]
fn m1_a_revoked_agency_cannot_sign_anything() {
    let Executed {
        mut env,
        city,
        supplier,
        request,
    } = executed();
    let ob = env
        .register_obligation(&city, "NE-1", &supplier.pubkey(), 500_000, Some(request))
        .unwrap();
    let ra = env.registry_authority.insecure_clone();
    env.set_agency_active(&ra, &city.pubkey(), false).unwrap();

    assert_eq!(
        env.register_obligation(&city, "NE-2", &supplier.pubkey(), 1, None),
        Err(code(ErrorCode::AgencyInactive))
    );
    assert_eq!(
        env.attach_doc(&city, &ob, "NFE-X", 1),
        Err(code(ErrorCode::AgencyInactive))
    );
    assert_eq!(env.pay(&city, &ob, 1), Err(code(ErrorCode::AgencyInactive)));

    let stranger = Keypair::new();
    assert_eq!(
        env.set_agency_active(&stranger, &city.pubkey(), true),
        Err(code(ErrorCode::Unauthorized))
    );
    env.set_agency_active(&ra, &city.pubkey(), true).unwrap();
    env.pay(&city, &ob, 1).unwrap();
}

#[test]
fn m2_obligations_cannot_exceed_the_value_of_their_source_adhesion() {
    // 8 units x R$ 1.250,00 = R$ 10.000,00 authorized.
    let Executed {
        mut env,
        city,
        supplier,
        request,
    } = executed();
    env.register_obligation(&city, "NE-1", &supplier.pubkey(), 600_000, Some(request))
        .unwrap();
    env.register_obligation(&city, "NE-2", &supplier.pubkey(), 400_000, Some(request))
        .unwrap();
    assert_eq!(
        env.register_obligation(&city, "NE-3", &supplier.pubkey(), 1, Some(request)),
        Err(code(ErrorCode::ExceedsAdhesionValue))
    );
    let r: AdhesionRequest = env.fetch(&request);
    assert_eq!(r.obligated_amount, BRL_10K);
    assert_eq!(r.unit_price, UNIT_PRICE);
}

#[test]
fn m3_no_execution_while_the_record_is_suspended() {
    let mut env = Env::new();
    let manager = env.new_agency(Sphere::State, "Central de Compras");
    let supplier = Keypair::new();
    let ata = env.create_ata(&manager, "ARP-M3", &supplier.pubkey(), 365);
    let item = env.add_item(&manager, &ata, 1, 100, 200);
    let city = env.new_agency(Sphere::Municipal, "Prefeitura A");
    let req = env.request(&city, &ata, &item, 1, 10).unwrap();
    env.supplier_respond(&supplier, &ata, &item, &city, &req, true).unwrap();
    env.authorize(&manager, &ata, &item, &city, &req, 10, [0u8; 32])
        .unwrap();

    env.set_status(&manager, &ata, AtaStatus::Suspended).unwrap();
    assert_eq!(env.formalize(&city, &req), Err(code(ErrorCode::AtaNotActive)));
    // L-3: no extension while suspended either.
    let r: AdhesionRequest = env.fetch(&req);
    assert_eq!(
        env.extend(&manager, &ata, &item, &city, &req, r.execute_by + DAY),
        Err(code(ErrorCode::AtaNotActive))
    );
    env.set_status(&manager, &ata, AtaStatus::Active).unwrap();
    env.formalize(&city, &req).unwrap();
}

#[test]
fn l2_unanswered_requests_lapse_after_the_response_window() {
    let mut env = Env::new();
    let manager = env.new_agency(Sphere::State, "Central de Compras");
    let supplier = Keypair::new();
    let ata = env.create_ata(&manager, "ARP-L2", &supplier.pubkey(), 365);
    let item = env.add_item(&manager, &ata, 1, 100, 200);
    let city = env.new_agency(Sphere::Municipal, "Prefeitura A");
    let req = env.request(&city, &ata, &item, 1, 50).unwrap();

    assert_eq!(env.expire(&item, &city, &req), Err(code(ErrorCode::ResponseWindowOpen)));
    let now = env.now();
    env.set_time(now + RESPONSE_WINDOW_SECS + 1);
    env.expire(&item, &city, &req).unwrap();
    let it: AtaItem = env.fetch(&item);
    assert_eq!(it.committed_capped, 0, "reserved quantity returns to the item");
    let r: AdhesionRequest = env.fetch(&req);
    assert_eq!(r.status, RequestStatus::Lapsed);
}

#[test]
fn l3_no_items_on_a_cancelled_record() {
    let mut env = Env::new();
    let manager = env.new_agency(Sphere::State, "Central de Compras");
    let ata = env.create_ata(&manager, "ARP-L3", &Keypair::new().pubkey(), 365);
    env.set_status(&manager, &ata, AtaStatus::Cancelled).unwrap();
    assert_eq!(
        env.try_add_item(&manager, &ata, 1, 100, 200),
        Err(code(ErrorCode::AtaNotActive))
    );
}

#[test]
fn l5_long_agency_names_get_their_own_error() {
    let mut env = Env::new();
    let authority = Keypair::new();
    let ix = env.ix(
        solutio::instruction::RegisterAgency {
            sphere: Sphere::Municipal,
            name: "x".repeat(65),
            attributes: AgencyAttributes {
                is_health_ministry: false,
                is_state_capital: false,
                profile: RuleProfile::Baseline,
            },
        },
        solutio::accounts::RegisterAgency {
            payer: env.sponsor.pubkey(),
            registry_authority: env.registry_authority.pubkey(),
            registry: env.registry(),
            agency_authority: authority.pubkey(),
            agency: env.agency(&authority.pubkey()),
            system_program: anchor_lang::solana_program::system_program::ID,
        },
    );
    let ra = env.registry_authority.insecure_clone();
    assert_eq!(env.send(ix, &[&ra]), Err(code(ErrorCode::NameTooLong)));
}

#[test]
fn an_item_from_another_record_is_rejected_as_a_mismatch() {
    let mut env = Env::new();
    let manager = env.new_agency(Sphere::State, "Central de Compras");
    let supplier = Keypair::new();
    let ata_a = env.create_ata(&manager, "ARP-A", &supplier.pubkey(), 365);
    let ata_b = env.create_ata(&manager, "ARP-B", &supplier.pubkey(), 365);
    let _item_a = env.add_item(&manager, &ata_a, 1, 100, 200);
    let item_b = env.add_item(&manager, &ata_b, 1, 100, 200);
    let city = env.new_agency(Sphere::Municipal, "Prefeitura A");
    // Claims record A's rules while writing to record B's item.
    assert_eq!(
        env.request(&city, &ata_a, &item_b, 1, 10),
        Err(code(ErrorCode::AccountMismatch))
    );
}

#[test]
fn closing_returns_rent_only_for_terminal_records() {
    let Executed {
        mut env,
        city,
        supplier,
        request,
    } = executed();
    let sponsor = env.sponsor.pubkey();

    // An executed adhesion stays open: obligations cite it.
    assert_eq!(env.close_request(&request, &sponsor), Err(code(ErrorCode::NotClosable)));

    let ob = env
        .register_obligation(&city, "NE-1", &supplier.pubkey(), BRL_10K, Some(request))
        .unwrap();
    env.attach_doc(&city, &ob, "NFE-1", BRL_10K).unwrap();
    env.mark_eligible(&ob, BRL_10K).unwrap();
    let fund = Keypair::new();
    let f0 = env.finance(&fund, &supplier, &ob, 600_000, hash("n0")).unwrap();
    let f1 = env.finance(&fund, &supplier, &ob, 400_000, hash("n1")).unwrap();

    // Nothing closes before settlement.
    assert_eq!(
        env.close_financing(&ob, &f0, &sponsor),
        Err(code(ErrorCode::NotClosable))
    );
    env.pay(&city, &ob, BRL_10K).unwrap();

    // Rent goes back only to whoever paid it.
    let thief = Keypair::new().pubkey();
    assert_eq!(
        env.close_financing(&ob, &f0, &thief),
        Err(code(ErrorCode::AccountMismatch))
    );

    env.close_financing(&ob, &f0, &sponsor).unwrap();
    assert_eq!(
        env.close_obligation(&ob, &sponsor),
        Err(code(ErrorCode::OpenFinancings))
    );
    env.close_financing(&ob, &f1, &sponsor).unwrap();

    let before = env.lamports(&sponsor);
    let ob_rent = env.lamports(&ob);
    env.close_obligation(&ob, &sponsor).unwrap();
    // The sponsor also paid this transaction's fee (5,000 lamports).
    assert_eq!(env.lamports(&sponsor), before + ob_rent - 5_000);
    assert_eq!(env.lamports(&ob), 0);

    // The fiscal document stays: the same invoice can never back a new obligation.
    let again = env
        .register_obligation(&city, "NE-2", &supplier.pubkey(), 1_000, None)
        .unwrap();
    assert!(env.attach_doc(&city, &again, "NFE-1", 1_000).is_err());
}

#[test]
fn a_rejected_request_can_be_closed_by_anyone() {
    let mut env = Env::new();
    let manager = env.new_agency(Sphere::State, "Central de Compras");
    let supplier = Keypair::new();
    let ata = env.create_ata(&manager, "ARP-CL", &supplier.pubkey(), 365);
    let item = env.add_item(&manager, &ata, 1, 100, 200);
    let city = env.new_agency(Sphere::Municipal, "Prefeitura A");
    let req = env.request(&city, &ata, &item, 1, 10).unwrap();
    env.supplier_respond(&supplier, &ata, &item, &city, &req, false)
        .unwrap();
    let sponsor = env.sponsor.pubkey();
    let rent = env.lamports(&req);
    let before = env.lamports(&sponsor);
    env.close_request(&req, &sponsor).unwrap();
    assert_eq!(env.lamports(&sponsor), before + rent - 5_000);
}

#[test]
fn invalid_validity_and_zero_amounts_are_refused() {
    let mut env = Env::new();
    let manager = env.new_agency(Sphere::State, "Central de Compras");
    let manager_agency = env.agency(&manager.pubkey());
    let ata_id = hash("ARP-BAD");
    let now = env.now();
    let ix = env.ix(
        solutio::instruction::CreateAta {
            ata_id,
            supplier: Keypair::new().pubkey(),
            doc_hash: hash("doc"),
            valid_from: now,
            valid_until: now,
        },
        solutio::accounts::CreateAta {
            payer: env.sponsor.pubkey(),
            manager_authority: manager.pubkey(),
            manager_agency,
            ata: env.ata_pda(&manager_agency, &ata_id),
            system_program: anchor_lang::solana_program::system_program::ID,
        },
    );
    assert_eq!(env.send(ix, &[&manager]), Err(code(ErrorCode::InvalidValidity)));

    let city = env.new_agency(Sphere::Municipal, "Prefeitura A");
    assert_eq!(
        env.register_obligation(&city, "NE-0", &Keypair::new().pubkey(), 0, None),
        Err(code(ErrorCode::InvalidAmount))
    );
}
