//! Module 2 — Obligations: verified balances, separate eligibility, single financing.

mod common;

use common::*;
use solana_keypair::Keypair;
use solana_signer::Signer;
use solutio::{error::ErrorCode, state::*};

const BRL_10K: u64 = 1_000_000; // R$ 10.000,00 in cents

struct Scenario {
    env: Env,
    city: Keypair,
    supplier: Keypair,
    obligation: anchor_lang::prelude::Pubkey,
}

/// A municipality adhered to a state price record, received the goods and
/// recognized a R$ 10.000,00 debt to the supplier (liquidation).
fn scenario() -> Scenario {
    let mut env = Env::new();
    let manager = env.new_agency(Sphere::State, "Central de Compras do Estado");
    let supplier = Keypair::new();
    let ata = env.create_ata(&manager, "ARP-001/2026", &supplier.pubkey(), 365);
    let item = env.add_item(&manager, &ata, 1, 100, false);
    let city = env.new_agency(Sphere::Municipal, "Prefeitura A");
    let req = env.full_adhesion(&manager, &supplier, &city, &ata, &item, 1, 8).unwrap();
    let obligation = env
        .register_obligation(&city, "UG-1/2026NE000123", &supplier.pubkey(), BRL_10K, Some(req))
        .unwrap();
    Scenario { env, city, supplier, obligation }
}

#[test]
fn full_lifecycle_finances_once_then_settles() {
    let Scenario { mut env, city, supplier, obligation } = scenario();
    env.attach_doc(&city, &obligation, "NFE-35261012345678000199550010000001231000001234", BRL_10K).unwrap();
    env.mark_eligible(&obligation, BRL_10K).unwrap();

    let fund_a = Keypair::new();
    let fund_b = Keypair::new();
    env.finance(&fund_a, &supplier, &obligation, 600_000, hash("notificacao-1")).unwrap();
    env.finance(&fund_b, &supplier, &obligation, 400_000, hash("notificacao-2")).unwrap();
    // The eligible balance is exhausted: any further financing is refused.
    assert_eq!(
        env.finance(&fund_a, &supplier, &obligation, 1, hash("notificacao-3")),
        Err(code(ErrorCode::ExceedsFinanceableBalance))
    );

    let o: Obligation = env.fetch(&obligation);
    assert_eq!(o.status, ObligationStatus::Financed);
    assert_eq!(o.financed_amount, BRL_10K);
    assert_eq!(o.financing_count, 2);

    env.pay(&city, &obligation, 250_000).unwrap();
    env.pay(&city, &obligation, 750_000).unwrap();
    let o: Obligation = env.fetch(&obligation);
    assert_eq!(o.status, ObligationStatus::Settled);
    assert_eq!(env.pay(&city, &obligation, 1), Err(code(ErrorCode::ExceedsOutstanding)));
}

#[test]
fn verified_is_not_eligible_eligibility_is_a_separate_signed_step() {
    let Scenario { mut env, city, supplier, obligation } = scenario();
    let fund = Keypair::new();
    // Verified (liquidated) but not confirmed eligible: cannot be financed.
    assert_eq!(
        env.finance(&fund, &supplier, &obligation, 1, hash("n")),
        Err(code(ErrorCode::ObligationNotFinanceable))
    );
    // Eligibility must be backed by fiscal documents.
    assert_eq!(env.mark_eligible(&obligation, 500_000), Err(code(ErrorCode::ExceedsDocumentedAmount)));
    env.attach_doc(&city, &obligation, "NFE-A", 500_000).unwrap();
    // Only the designated verifier can confirm eligibility.
    let stranger = Keypair::new();
    assert_eq!(env.mark_eligible_as(&stranger, &obligation, 500_000), Err(code(ErrorCode::Unauthorized)));
    env.mark_eligible(&obligation, 500_000).unwrap();
    // Financing is capped by eligibility, not by the verified amount.
    assert_eq!(
        env.finance(&fund, &supplier, &obligation, 500_001, hash("n")),
        Err(code(ErrorCode::ExceedsFinanceableBalance))
    );
    env.finance(&fund, &supplier, &obligation, 500_000, hash("n")).unwrap();
}

#[test]
fn the_same_fiscal_document_cannot_back_two_obligations() {
    let Scenario { mut env, city, supplier, obligation } = scenario();
    let nfe = "NFE-35261012345678000199550010000001231000001234";
    env.attach_doc(&city, &obligation, nfe, 400_000).unwrap();
    let second = env
        .register_obligation(&city, "UG-1/2026NE000999", &supplier.pubkey(), 400_000, None)
        .unwrap();
    // The document's address already exists, so the program refuses to bind it again.
    assert!(env.attach_doc(&city, &second, nfe, 400_000).is_err());
    // Documents can never add up to more than the verified amount.
    assert_eq!(
        env.attach_doc(&city, &obligation, "NFE-B", 600_001),
        Err(code(ErrorCode::ExceedsVerifiedAmount))
    );
}

#[test]
fn financing_requires_the_creditor_and_notice_evidence() {
    let Scenario { mut env, city, supplier, obligation } = scenario();
    env.attach_doc(&city, &obligation, "NFE-A", BRL_10K).unwrap();
    env.mark_eligible(&obligation, BRL_10K).unwrap();
    let fund = Keypair::new();
    let not_the_creditor = Keypair::new();
    assert_eq!(
        env.finance(&fund, &not_the_creditor, &obligation, 1_000, hash("n")),
        Err(code(ErrorCode::Unauthorized))
    );
    assert_eq!(
        env.finance(&fund, &supplier, &obligation, 1_000, [0u8; 32]),
        Err(code(ErrorCode::MissingEvidence))
    );
}

#[test]
fn a_disallowance_after_financing_marks_the_obligation_impaired() {
    let Scenario { mut env, city, supplier, obligation } = scenario();
    env.attach_doc(&city, &obligation, "NFE-A", BRL_10K).unwrap();
    env.mark_eligible(&obligation, BRL_10K).unwrap();
    let fund = Keypair::new();
    env.finance(&fund, &supplier, &obligation, 900_000, hash("n")).unwrap();
    // The debtor disallows R$ 2.000,00. The protocol records it; it cannot refuse a government act.
    env.reduce(&city, &obligation, 200_000).unwrap();
    let o: Obligation = env.fetch(&obligation);
    assert_eq!(o.status, ObligationStatus::Impaired);
    assert_eq!(o.eligible_amount, 800_000);
    assert_eq!(
        env.finance(&fund, &supplier, &obligation, 1, hash("n")),
        Err(code(ErrorCode::ObligationNotFinanceable))
    );
}

#[test]
fn financing_closes_once_the_debtor_starts_paying() {
    let Scenario { mut env, city, supplier, obligation } = scenario();
    env.attach_doc(&city, &obligation, "NFE-A", BRL_10K).unwrap();
    env.mark_eligible(&obligation, BRL_10K).unwrap();
    env.pay(&city, &obligation, 100_000).unwrap();
    let fund = Keypair::new();
    assert!(env.finance(&fund, &supplier, &obligation, 1, hash("n")).is_err());
}

#[test]
fn an_obligation_can_only_derive_from_an_effective_adhesion() {
    let mut env = Env::new();
    let manager = env.new_agency(Sphere::State, "Central");
    let supplier = Keypair::new();
    let ata = env.create_ata(&manager, "ARP-002", &supplier.pubkey(), 365);
    let item = env.add_item(&manager, &ata, 1, 100, false);
    let city = env.new_agency(Sphere::Municipal, "Prefeitura A");
    let other_city = env.new_agency(Sphere::Municipal, "Prefeitura B");

    // Requested but not yet approved and accepted.
    let pending = env.request(&city, &ata, &item, 1, 10).unwrap();
    assert_eq!(
        env.register_obligation(&city, "NE-1", &supplier.pubkey(), 1_000, Some(pending)),
        Err(code(ErrorCode::SourceAdhesionNotEffective))
    );

    // Effective, but the obligation is claimed by a different debtor or creditor.
    env.approve(&manager, &ata, &pending).unwrap();
    env.accept(&supplier, &ata, &item, &city, &pending).unwrap();
    assert_eq!(
        env.register_obligation(&other_city, "NE-2", &supplier.pubkey(), 1_000, Some(pending)),
        Err(code(ErrorCode::SourceAdhesionMismatch))
    );
    let wrong_supplier = Keypair::new();
    assert_eq!(
        env.register_obligation(&city, "NE-3", &wrong_supplier.pubkey(), 1_000, Some(pending)),
        Err(code(ErrorCode::SourceAdhesionMismatch))
    );
    env.register_obligation(&city, "NE-4", &supplier.pubkey(), 1_000, Some(pending)).unwrap();
}

#[test]
fn only_the_debtor_can_record_payments_and_reductions() {
    let Scenario { mut env, obligation, .. } = scenario();
    let other_city = env.new_agency(Sphere::Municipal, "Prefeitura B");
    assert_eq!(env.pay(&other_city, &obligation, 1_000), Err(code(ErrorCode::Unauthorized)));
    assert_eq!(env.reduce(&other_city, &obligation, 1_000), Err(code(ErrorCode::Unauthorized)));
}
