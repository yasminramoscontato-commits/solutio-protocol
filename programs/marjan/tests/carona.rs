//! Module 1 — Carona: Law 14.133/2021 art. 86 and Decree 11.462/2023 arts. 31-33,
//! enforced by the compiled program.

mod common;

use common::*;
use marjan::{error::ErrorCode, state::*};
use solana_keypair::Keypair;
use solana_signer::Signer;

type Pk = anchor_lang::prelude::Pubkey;

struct Scenario {
    env: Env,
    manager: Keypair,
    supplier: Keypair,
    ata: Pk,
    item: Pk,
}

/// A state central purchasing body manages a price record with one item.
/// `max_adhesion` is the tender's maximum for non-participants (<= 2x registered).
fn scenario(registered: u64, max_adhesion: u64) -> Scenario {
    let mut env = Env::new();
    let manager = env.new_agency(Sphere::State, "Central de Compras do Estado");
    let supplier = Keypair::new();
    let ata = env.create_ata(&manager, "ARP-001/2026", &supplier.pubkey(), 365);
    let item = env.add_item(&manager, &ata, 1, registered, max_adhesion);
    Scenario {
        env,
        manager,
        supplier,
        ata,
        item,
    }
}

fn standard() -> Scenario {
    scenario(100, 200)
}

#[test]
fn decree_order_supplier_accepts_before_the_manager_authorizes() {
    let Scenario {
        mut env,
        manager,
        supplier,
        ata,
        item,
    } = standard();
    let city = env.new_agency(Sphere::Municipal, "Prefeitura A");
    let req = env.request(&city, &ata, &item, 1, 30).unwrap();

    // Art. 31 §1: the manager cannot authorize before the supplier accepts.
    assert_eq!(
        env.authorize(&manager, &ata, &item, &city, &req, 30, [0u8; 32]),
        Err(code(ErrorCode::InvalidRequestStatus))
    );
    // Only the registered supplier can respond.
    let impostor = Keypair::new();
    assert_eq!(
        env.supplier_respond(&impostor, &ata, &item, &city, &req, true),
        Err(code(ErrorCode::Unauthorized))
    );
    env.supplier_respond(&supplier, &ata, &item, &city, &req, true).unwrap();
    // Only the managing agency can authorize.
    let other_state = env.new_agency(Sphere::State, "Outro Estado");
    assert_eq!(
        env.authorize(&other_state, &ata, &item, &city, &req, 30, [0u8; 32]),
        Err(code(ErrorCode::Unauthorized))
    );
    env.authorize(&manager, &ata, &item, &city, &req, 30, [0u8; 32])
        .unwrap();
    env.formalize(&city, &req).unwrap();

    let r: AdhesionRequest = env.fetch(&req);
    assert_eq!(r.status, RequestStatus::Executed);
    assert_eq!(r.authorized_qty, 30);
    let it: AtaItem = env.fetch(&item);
    assert_eq!(it.committed_capped, 30);
    assert_eq!(it.authorized_total, 30);
}

#[test]
fn pending_requests_reserve_quantity_as_in_the_federal_system() {
    let Scenario { mut env, ata, item, .. } = standard();
    let city = env.new_agency(Sphere::Municipal, "Prefeitura A");
    env.request(&city, &ata, &item, 1, 40).unwrap();
    // Nothing approved yet, but the pending 40 already counts against §4.
    assert_eq!(
        env.request(&city, &ata, &item, 2, 11),
        Err(code(ErrorCode::ExceedsIndividualCap))
    );
    let it: AtaItem = env.fetch(&item);
    assert_eq!(it.committed_capped, 40);
}

#[test]
fn partial_authorization_releases_the_remainder_and_requires_justification() {
    let Scenario {
        mut env,
        manager,
        supplier,
        ata,
        item,
    } = standard();
    let city = env.new_agency(Sphere::Municipal, "Prefeitura A");
    let req = env.request(&city, &ata, &item, 1, 17).unwrap();
    env.supplier_respond(&supplier, &ata, &item, &city, &req, true).unwrap();
    assert_eq!(
        env.authorize(&manager, &ata, &item, &city, &req, 10, [0u8; 32]),
        Err(code(ErrorCode::MissingEvidence)),
        "partial authorization needs a recorded justification"
    );
    assert_eq!(
        env.authorize(&manager, &ata, &item, &city, &req, 18, [0u8; 32]),
        Err(code(ErrorCode::InvalidQuantity))
    );
    env.authorize(&manager, &ata, &item, &city, &req, 10, hash("motivacao-parcial"))
        .unwrap();
    let it: AtaItem = env.fetch(&item);
    assert_eq!(it.committed_capped, 10);
    let usage: AgencyItemUsage = env.fetch(&env.usage_pda(&item, &env.agency(&city.pubkey())));
    assert_eq!(usage.committed, 10);
}

#[test]
fn declined_and_denied_requests_release_their_reservation() {
    let Scenario {
        mut env,
        manager,
        supplier,
        ata,
        item,
    } = standard();
    let city = env.new_agency(Sphere::Municipal, "Prefeitura A");
    let declined = env.request(&city, &ata, &item, 1, 50).unwrap();
    env.supplier_respond(&supplier, &ata, &item, &city, &declined, false)
        .unwrap();
    let denied = env.request(&city, &ata, &item, 2, 50).unwrap();
    env.supplier_respond(&supplier, &ata, &item, &city, &denied, true)
        .unwrap();
    env.deny(&manager, &ata, &item, &city, &denied).unwrap();
    assert_eq!(
        env.authorize(&manager, &ata, &item, &city, &denied, 50, [0u8; 32]),
        Err(code(ErrorCode::InvalidRequestStatus))
    );
    let it: AtaItem = env.fetch(&item);
    assert_eq!(it.committed_capped, 0);
    // The full 50% is available again.
    env.request(&city, &ata, &item, 3, 50).unwrap();
}

#[test]
fn art86_par4_each_agency_is_capped_at_half_the_registered_quantity() {
    let Scenario {
        mut env,
        manager,
        supplier,
        ata,
        item,
    } = standard();
    let city = env.new_agency(Sphere::Municipal, "Prefeitura A");
    assert_eq!(
        env.request(&city, &ata, &item, 1, 51),
        Err(code(ErrorCode::ExceedsIndividualCap))
    );
    env.full_adhesion(&manager, &supplier, &city, &ata, &item, 2, 30)
        .unwrap();
    env.full_adhesion(&manager, &supplier, &city, &ata, &item, 3, 20)
        .unwrap();
    assert_eq!(
        env.request(&city, &ata, &item, 4, 1),
        Err(code(ErrorCode::ExceedsIndividualCap))
    );
}

#[test]
fn art86_par5_total_adhesions_are_capped_at_twice_the_registered_quantity() {
    let Scenario {
        mut env,
        manager,
        supplier,
        ata,
        item,
    } = standard();
    for (i, name) in ["A", "B", "C", "D"].iter().enumerate() {
        let city = env.new_agency(Sphere::Municipal, name);
        env.full_adhesion(&manager, &supplier, &city, &ata, &item, i as u64, 50)
            .unwrap();
    }
    let fifth = env.new_agency(Sphere::Municipal, "E");
    assert_eq!(
        env.request(&fifth, &ata, &item, 9, 1),
        Err(code(ErrorCode::ExceedsGlobalCap))
    );
}

#[test]
fn the_tender_can_set_a_lower_maximum_or_forbid_adhesions() {
    let Scenario {
        mut env,
        manager,
        ata,
        item,
        ..
    } = scenario(100, 60);
    let a = env.new_agency(Sphere::Municipal, "A");
    let b = env.new_agency(Sphere::Municipal, "B");
    env.request(&a, &ata, &item, 1, 50).unwrap();
    assert_eq!(
        env.request(&b, &ata, &item, 1, 11),
        Err(code(ErrorCode::ExceedsGlobalCap))
    );
    // A maximum above the statutory 2x is refused at registration.
    assert_eq!(
        env.try_add_item(&manager, &ata, 2, 100, 201),
        Err(code(ErrorCode::InvalidMaxAdhesion))
    );
    // Zero means the tender does not allow adhesions.
    let closed = env.add_item(&manager, &ata, 3, 100, 0);
    assert_eq!(
        env.request(&a, &ata, &closed, 2, 1),
        Err(code(ErrorCode::AdhesionsNotAllowed))
    );
}

/// Two agencies race for the last 50 units. The first lawful request reserves
/// them; the second is refused immediately. If the first is denied, the units
/// return and the second agency can try again.
#[test]
fn race_for_the_last_units_only_one_can_win() {
    let Scenario {
        mut env,
        manager,
        supplier,
        ata,
        item,
    } = standard();
    for (i, name) in ["A", "B", "C"].iter().enumerate() {
        let city = env.new_agency(Sphere::Municipal, name);
        env.full_adhesion(&manager, &supplier, &city, &ata, &item, i as u64, 50)
            .unwrap();
    }
    let d = env.new_agency(Sphere::Municipal, "D");
    let e = env.new_agency(Sphere::Municipal, "E");
    let req_d = env.request(&d, &ata, &item, 1, 50).unwrap();
    assert_eq!(
        env.request(&e, &ata, &item, 1, 50),
        Err(code(ErrorCode::ExceedsGlobalCap))
    );

    env.deny(&manager, &ata, &item, &d, &req_d).unwrap();
    env.request(&e, &ata, &item, 2, 50).unwrap();
    let it: AtaItem = env.fetch(&item);
    assert_eq!(it.committed_capped, 200);
}

#[test]
fn health_emergency_exception_requires_a_ministry_of_health_record() {
    let Scenario { mut env, ata, item, .. } = standard();
    let city = env.new_agency(Sphere::Municipal, "Prefeitura A");
    assert_eq!(
        env.request_ex(&city, &ata, &item, 1, 10, AdhesionException::HealthEmergency),
        Err(code(ErrorCode::ExceptionNotApplicable))
    );

    // A record managed by the Ministry of Health, with the §5 pool exhausted.
    let ms = env.new_agency_ex(Sphere::Federal, "Ministerio da Saude", true);
    let supplier = Keypair::new();
    let ms_ata = env.create_ata(&ms, "ARP-MS-01", &supplier.pubkey(), 365);
    let ms_item = env.add_item(&ms, &ms_ata, 1, 100, 200);
    for (i, name) in ["A", "B", "C", "D"].iter().enumerate() {
        let c = env.new_agency(Sphere::Municipal, name);
        env.full_adhesion(&ms, &supplier, &c, &ms_ata, &ms_item, i as u64, 50)
            .unwrap();
    }
    let emergency = env.new_agency(Sphere::State, "Secretaria Estadual de Saude");
    assert_eq!(
        env.request(&emergency, &ms_ata, &ms_item, 1, 10),
        Err(code(ErrorCode::ExceedsGlobalCap))
    );
    env.request_ex(&emergency, &ms_ata, &ms_item, 2, 50, AdhesionException::HealthEmergency)
        .unwrap();
    // §4 still applies under the exception.
    assert_eq!(
        env.request_ex(&emergency, &ms_ata, &ms_item, 3, 1, AdhesionException::HealthEmergency),
        Err(code(ErrorCode::ExceedsIndividualCap))
    );
    let it: AtaItem = env.fetch(&ms_item);
    assert_eq!(it.committed_capped, 200);
    assert_eq!(it.committed_exempt, 50);
}

#[test]
fn federal_programme_transfer_exception_is_for_subnational_agencies_only() {
    let mut env = Env::new();
    let manager = env.new_agency(Sphere::Federal, "Central de Compras Federal");
    let supplier = Keypair::new();
    let ata = env.create_ata(&manager, "ARP-FED-1", &supplier.pubkey(), 365);
    let item = env.add_item(&manager, &ata, 1, 100, 200);
    let ministry = env.new_agency(Sphere::Federal, "Ministerio X");
    assert_eq!(
        env.request_ex(&ministry, &ata, &item, 1, 10, AdhesionException::FederalProgramTransfer),
        Err(code(ErrorCode::ExceptionNotApplicable))
    );
    let city = env.new_agency(Sphere::Municipal, "Prefeitura A");
    env.request_ex(&city, &ata, &item, 1, 10, AdhesionException::FederalProgramTransfer)
        .unwrap();

    // Art. 86 §6 lifts §5 only for records of the federal Executive: a state
    // programme cannot claim it, even when the state requires the adhesion.
    let state = env.new_agency(Sphere::State, "Central de Compras do Estado");
    let state_ata = env.create_ata(&state, "ARP-EST-1", &supplier.pubkey(), 365);
    let state_item = env.add_item(&state, &state_ata, 1, 100, 200);
    assert_eq!(
        env.request_ex(
            &city,
            &state_ata,
            &state_item,
            1,
            10,
            AdhesionException::FederalProgramTransfer
        ),
        Err(code(ErrorCode::ExceptionNotApplicable))
    );
}

#[test]
fn alagoas_profile_blocks_state_adhesion_to_non_capital_municipal_records() {
    let mut env = Env::new();
    let supplier = Keypair::new();
    let plain = AgencyAttributes {
        is_health_ministry: false,
        is_state_capital: false,
        profile: RuleProfile::Baseline,
    };
    let interior = env.new_agency(Sphere::Municipal, "Prefeitura do Interior");
    let capital = env.new_agency_with(
        Sphere::Municipal,
        "Prefeitura da Capital",
        AgencyAttributes {
            is_state_capital: true,
            ..plain
        },
    );
    let interior_ata = env.create_ata(&interior, "ARP-MUN-1", &supplier.pubkey(), 365);
    let interior_item = env.add_item(&interior, &interior_ata, 1, 100, 200);
    let capital_ata = env.create_ata(&capital, "ARP-MUN-2", &supplier.pubkey(), 365);
    let capital_item = env.add_item(&capital, &capital_ata, 1, 100, 200);

    // Decree 95.019/2023 art. 33 governs Alagoas state agencies.
    let al_secretariat = env.new_agency_with(
        Sphere::State,
        "Secretaria Estadual (AL)",
        AgencyAttributes {
            profile: RuleProfile::Alagoas,
            ..plain
        },
    );
    assert_eq!(
        env.request(&al_secretariat, &interior_ata, &interior_item, 1, 10),
        Err(code(ErrorCode::MunicipalAdhesionForbidden))
    );
    env.request(&al_secretariat, &capital_ata, &capital_item, 1, 10)
        .unwrap();

    // A state agency under the federal baseline has no such restriction.
    let other_state = env.new_agency(Sphere::State, "Secretaria Estadual (baseline)");
    env.request(&other_state, &interior_ata, &interior_item, 1, 10).unwrap();
}

#[test]
fn art86_par8_federal_agency_cannot_adhere_to_a_state_record() {
    let Scenario { mut env, ata, item, .. } = standard();
    let federal = env.new_agency(Sphere::Federal, "Ministerio X");
    assert_eq!(
        env.request(&federal, &ata, &item, 1, 10),
        Err(code(ErrorCode::FederalAdhesionForbidden))
    );
}

#[test]
fn the_manager_cannot_adhere_to_its_own_record() {
    let Scenario {
        mut env,
        manager,
        ata,
        item,
        ..
    } = standard();
    assert_eq!(
        env.request(&manager, &ata, &item, 1, 10),
        Err(code(ErrorCode::ManagerCannotAdhere))
    );
}

#[test]
fn ninety_day_execution_window_extension_and_lapse() {
    let Scenario {
        mut env,
        manager,
        supplier,
        ata,
        item,
    } = standard();
    let a = env.new_agency(Sphere::Municipal, "A");
    let b = env.new_agency(Sphere::Municipal, "B");
    let req_a = env.request(&a, &ata, &item, 1, 40).unwrap();
    let req_b = env.request(&b, &ata, &item, 1, 40).unwrap();
    for (agency, req) in [(&a, &req_a), (&b, &req_b)] {
        env.supplier_respond(&supplier, &ata, &item, agency, req, true).unwrap();
        env.authorize(&manager, &ata, &item, agency, req, 40, [0u8; 32])
            .unwrap();
    }
    let r: AdhesionRequest = env.fetch(&req_a);
    assert_eq!(r.execute_by, r.authorized_at + 90 * DAY);

    // Nobody can lapse an adhesion before its deadline.
    assert_eq!(
        env.expire(&item, &a, &req_a),
        Err(code(ErrorCode::ExecutionDeadlineNotReached))
    );
    // The manager extends A's deadline (art. 31 §3), but never past validity.
    let ata_acc: Ata = env.fetch(&ata);
    assert_eq!(
        env.extend(&manager, &ata, &item, &a, &req_a, ata_acc.valid_until + 1),
        Err(code(ErrorCode::InvalidDeadline))
    );
    env.extend(&manager, &ata, &item, &a, &req_a, r.execute_by + 30 * DAY)
        .unwrap();

    let t = env.now() + 100 * DAY;
    env.set_time(t);
    // B missed its deadline: it can no longer execute, and anyone can lapse it.
    assert_eq!(env.formalize(&b, &req_b), Err(code(ErrorCode::ExecutionDeadlinePassed)));
    env.expire(&item, &b, &req_b).unwrap();
    // A is still within its extended deadline.
    env.formalize(&a, &req_a).unwrap();

    let it: AtaItem = env.fetch(&item);
    assert_eq!(it.committed_capped, 40, "B's 40 units returned to the pool");
    let rb: AdhesionRequest = env.fetch(&req_b);
    assert_eq!(rb.status, RequestStatus::Lapsed);
}

#[test]
fn suspended_or_cancelled_records_accept_no_new_adhesions() {
    let Scenario {
        mut env,
        manager,
        supplier,
        ata,
        item,
    } = standard();
    let city = env.new_agency(Sphere::Municipal, "Prefeitura A");
    let req = env.request(&city, &ata, &item, 1, 10).unwrap();
    env.supplier_respond(&supplier, &ata, &item, &city, &req, true).unwrap();

    env.set_status(&manager, &ata, AtaStatus::Suspended).unwrap();
    assert_eq!(
        env.request(&city, &ata, &item, 2, 10),
        Err(code(ErrorCode::AtaNotActive))
    );
    assert_eq!(
        env.authorize(&manager, &ata, &item, &city, &req, 10, [0u8; 32]),
        Err(code(ErrorCode::AtaNotActive))
    );
    env.set_status(&manager, &ata, AtaStatus::Active).unwrap();
    env.authorize(&manager, &ata, &item, &city, &req, 10, [0u8; 32])
        .unwrap();

    env.set_status(&manager, &ata, AtaStatus::Cancelled).unwrap();
    assert_eq!(
        env.request(&city, &ata, &item, 3, 10),
        Err(code(ErrorCode::AtaNotActive))
    );
    // Cancellation is final.
    assert_eq!(
        env.set_status(&manager, &ata, AtaStatus::Active),
        Err(code(ErrorCode::AtaNotActive))
    );
}

#[test]
fn expired_record_rejects_requests_and_late_authorizations() {
    let Scenario {
        mut env,
        manager,
        supplier,
        ata,
        item,
    } = standard();
    let city = env.new_agency(Sphere::Municipal, "Prefeitura A");
    let req = env.request(&city, &ata, &item, 1, 10).unwrap();
    env.supplier_respond(&supplier, &ata, &item, &city, &req, true).unwrap();
    let later = env.now() + 400 * DAY;
    env.set_time(later);
    assert_eq!(
        env.authorize(&manager, &ata, &item, &city, &req, 10, [0u8; 32]),
        Err(code(ErrorCode::AtaNotInForce))
    );
    assert_eq!(
        env.request(&city, &ata, &item, 2, 10),
        Err(code(ErrorCode::AtaNotInForce))
    );
}

#[test]
fn agency_wallets_sign_without_holding_any_sol() {
    let Scenario {
        mut env,
        manager,
        supplier,
        ata,
        item,
    } = standard();
    let city = env.new_agency(Sphere::Municipal, "Prefeitura A");
    assert_eq!(env.lamports(&city.pubkey()), 0);
    env.full_adhesion(&manager, &supplier, &city, &ata, &item, 1, 10)
        .unwrap();
    // Every fee and rent deposit was paid by the sponsor.
    assert_eq!(env.lamports(&city.pubkey()), 0);
    assert_eq!(env.lamports(&manager.pubkey()), 0);
    assert_eq!(env.lamports(&supplier.pubkey()), 0);
}
