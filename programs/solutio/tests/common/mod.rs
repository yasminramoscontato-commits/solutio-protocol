//! Shared helpers for LiteSVM integration tests. Every test runs the compiled
//! program (`target/deploy/solutio.so`) inside an in-process Solana VM, with
//! real signatures, fees and account constraints.
#![allow(dead_code)]

use anchor_lang::{
    prelude::{Clock, Pubkey},
    solana_program::{bpf_loader_upgradeable, instruction::Instruction, system_program},
    AccountDeserialize, InstructionData, ToAccountMetas,
};
use litesvm::LiteSVM;
use solana_keypair::Keypair;
use solana_message::{Message, VersionedMessage};
use solana_signer::Signer;
use solana_transaction::versioned::VersionedTransaction;
use solutio::{constants::*, error::ErrorCode, state::*};

pub const DAY: i64 = 86_400;
/// R$ 1.250,00 per unit, in cents.
pub const UNIT_PRICE: u64 = 125_000;

pub struct Env {
    pub svm: LiteSVM,
    pub program_id: Pubkey,
    /// Pays every fee and every rent deposit. Agency wallets never hold SOL.
    pub sponsor: Keypair,
    pub registry_authority: Keypair,
    pub verifier: Keypair,
}

pub fn hash(label: &str) -> [u8; 32] {
    // Deterministic, non-zero 32-byte value standing in for a document hash.
    let mut out = [0u8; 32];
    for (i, b) in label.bytes().enumerate() {
        out[i % 32] = out[i % 32].wrapping_mul(31).wrapping_add(b);
    }
    out[31] |= 1;
    out
}

pub fn code(e: ErrorCode) -> u32 {
    e.into()
}

impl Env {
    pub fn new() -> Self {
        let mut env = Self::new_uninitialized();
        let ra = env.registry_authority.insecure_clone();
        let v = env.verifier.pubkey();
        env.init_registry(&ra, v).unwrap();
        env
    }

    /// Program loaded with `registry_authority` as its upgrade authority, registry not initialized.
    pub fn new_uninitialized() -> Self {
        let program_id = solutio::id();
        let mut svm = LiteSVM::new();
        let bytes = include_bytes!(concat!(env!("CARGO_TARGET_TMPDIR"), "/../deploy/solutio.so"));
        svm.add_program(program_id, bytes).unwrap();
        let sponsor = Keypair::new();
        svm.airdrop(&sponsor.pubkey(), 100_000_000_000).unwrap();
        let mut env = Env {
            svm,
            program_id,
            sponsor,
            registry_authority: Keypair::new(),
            verifier: Keypair::new(),
        };
        env.set_time(1_000 * DAY);
        // LiteSVM deploys with no upgrade authority; set it in the ProgramData
        // header (bincode: u32 tag, u64 slot, Option<Pubkey>).
        let pd = env.program_data();
        let mut acc = env.svm.get_account(&pd).unwrap();
        acc.data[12] = 1;
        acc.data[13..45].copy_from_slice(env.registry_authority.pubkey().as_ref());
        env.svm.set_account(pd, acc).unwrap();
        env
    }

    pub fn program_data(&self) -> Pubkey {
        Pubkey::find_program_address(&[self.program_id.as_ref()], &bpf_loader_upgradeable::ID).0
    }

    pub fn init_registry(&mut self, authority: &Keypair, verifier: Pubkey) -> Result<(), u32> {
        let ix = self.ix(
            solutio::instruction::InitRegistry {
                eligibility_verifier: verifier,
            },
            solutio::accounts::InitRegistry {
                payer: self.sponsor.pubkey(),
                authority: authority.pubkey(),
                program: self.program_id,
                program_data: self.program_data(),
                registry: self.registry(),
                system_program: system_program::ID,
            },
        );
        self.send(ix, &[authority])
    }

    pub fn set_verifier(&mut self, signer: &Keypair, new_verifier: Pubkey) -> Result<(), u32> {
        let ix = self.ix(
            solutio::instruction::SetEligibilityVerifier { new_verifier },
            solutio::accounts::RegistryAdmin {
                registry_authority: signer.pubkey(),
                registry: self.registry(),
            },
        );
        self.send(ix, &[signer])
    }

    pub fn set_registry_authority(&mut self, current: &Keypair, new: &Keypair) -> Result<(), u32> {
        let ix = self.ix(
            solutio::instruction::SetRegistryAuthority {},
            solutio::accounts::SetRegistryAuthority {
                registry_authority: current.pubkey(),
                new_authority: new.pubkey(),
                registry: self.registry(),
            },
        );
        self.send(ix, &[current, new])
    }

    pub fn set_agency_active(&mut self, signer: &Keypair, agency_authority: &Pubkey, active: bool) -> Result<(), u32> {
        let ix = self.ix(
            solutio::instruction::SetAgencyActive { active },
            solutio::accounts::SetAgencyActive {
                registry_authority: signer.pubkey(),
                registry: self.registry(),
                agency: self.agency(agency_authority),
            },
        );
        self.send(ix, &[signer])
    }

    pub fn close_request(&mut self, request: &Pubkey, rent_payer: &Pubkey) -> Result<(), u32> {
        let ix = self.ix(
            solutio::instruction::CloseRequest {},
            solutio::accounts::CloseRequest {
                request: *request,
                rent_payer: *rent_payer,
            },
        );
        self.send(ix, &[])
    }

    pub fn close_financing(&mut self, obligation: &Pubkey, financing: &Pubkey, rent_payer: &Pubkey) -> Result<(), u32> {
        let ix = self.ix(
            solutio::instruction::CloseFinancing {},
            solutio::accounts::CloseFinancing {
                obligation: *obligation,
                financing: *financing,
                rent_payer: *rent_payer,
            },
        );
        self.send(ix, &[])
    }

    pub fn close_obligation(&mut self, obligation: &Pubkey, rent_payer: &Pubkey) -> Result<(), u32> {
        let ix = self.ix(
            solutio::instruction::CloseObligation {},
            solutio::accounts::CloseObligation {
                obligation: *obligation,
                rent_payer: *rent_payer,
            },
        );
        self.send(ix, &[])
    }

    pub fn set_time(&mut self, unix: i64) {
        let mut clock: Clock = self.svm.get_sysvar();
        clock.unix_timestamp = unix;
        self.svm.set_sysvar(&clock);
    }

    pub fn now(&self) -> i64 {
        let clock: Clock = self.svm.get_sysvar();
        clock.unix_timestamp
    }

    /// Sends one instruction with the sponsor as fee payer plus extra signers.
    /// Returns the custom program error code on failure (or u32::MAX for
    /// non-custom failures such as an account that already exists).
    pub fn send(&mut self, ix: Instruction, signers: &[&Keypair]) -> Result<(), u32> {
        let mut all: Vec<&Keypair> = vec![&self.sponsor];
        for s in signers {
            if s.pubkey() != self.sponsor.pubkey() {
                all.push(s);
            }
        }
        let blockhash = self.svm.latest_blockhash();
        let msg = Message::new_with_blockhash(&[ix], Some(&self.sponsor.pubkey()), &blockhash);
        let tx = VersionedTransaction::try_new(VersionedMessage::Legacy(msg), &all).unwrap();
        let res = self.svm.send_transaction(tx);
        self.svm.expire_blockhash();
        match res {
            Ok(_) => Ok(()),
            Err(failed) => {
                use solana_transaction_error::TransactionError;
                match failed.err {
                    TransactionError::InstructionError(_, solana_instruction_error::InstructionError::Custom(c)) => {
                        Err(c)
                    }
                    _ => Err(u32::MAX),
                }
            }
        }
    }

    pub fn ix<D: InstructionData, A: ToAccountMetas>(&self, data: D, accounts: A) -> Instruction {
        Instruction::new_with_bytes(self.program_id, &data.data(), accounts.to_account_metas(None))
    }

    pub fn fetch<T: AccountDeserialize>(&self, key: &Pubkey) -> T {
        let acc = self.svm.get_account(key).expect("account exists");
        let mut data: &[u8] = &acc.data;
        T::try_deserialize(&mut data).unwrap()
    }

    pub fn lamports(&self, key: &Pubkey) -> u64 {
        self.svm.get_account(key).map(|a| a.lamports).unwrap_or(0)
    }

    // ---- PDAs ----
    pub fn pda(&self, seeds: &[&[u8]]) -> Pubkey {
        Pubkey::find_program_address(seeds, &self.program_id).0
    }
    pub fn registry(&self) -> Pubkey {
        self.pda(&[REGISTRY_SEED])
    }
    pub fn agency(&self, authority: &Pubkey) -> Pubkey {
        self.pda(&[AGENCY_SEED, authority.as_ref()])
    }
    pub fn ata_pda(&self, manager_agency: &Pubkey, ata_id: &[u8; 32]) -> Pubkey {
        self.pda(&[ATA_SEED, manager_agency.as_ref(), ata_id.as_ref()])
    }
    pub fn item_pda(&self, ata: &Pubkey, item_no: u16) -> Pubkey {
        self.pda(&[ITEM_SEED, ata.as_ref(), item_no.to_le_bytes().as_ref()])
    }
    pub fn usage_pda(&self, item: &Pubkey, agency: &Pubkey) -> Pubkey {
        self.pda(&[USAGE_SEED, item.as_ref(), agency.as_ref()])
    }
    pub fn request_pda(&self, item: &Pubkey, agency: &Pubkey, request_id: u64) -> Pubkey {
        self.pda(&[
            REQUEST_SEED,
            item.as_ref(),
            agency.as_ref(),
            request_id.to_le_bytes().as_ref(),
        ])
    }
    pub fn obligation_pda(&self, debtor_agency: &Pubkey, id: &[u8; 32]) -> Pubkey {
        self.pda(&[OBLIGATION_SEED, debtor_agency.as_ref(), id.as_ref()])
    }
    pub fn fiscal_doc_pda(&self, doc_key_hash: &[u8; 32]) -> Pubkey {
        self.pda(&[FISCAL_DOC_SEED, doc_key_hash.as_ref()])
    }
    pub fn financing_pda(&self, obligation: &Pubkey, n: u32) -> Pubkey {
        self.pda(&[FINANCING_SEED, obligation.as_ref(), n.to_le_bytes().as_ref()])
    }

    // ---- high-level actions ----

    /// Registers an agency whose signing wallet holds zero SOL.
    pub fn new_agency(&mut self, sphere: Sphere, name: &str) -> Keypair {
        self.new_agency_ex(sphere, name, false)
    }

    pub fn new_agency_ex(&mut self, sphere: Sphere, name: &str, is_health_ministry: bool) -> Keypair {
        let attributes = AgencyAttributes {
            is_health_ministry,
            is_state_capital: false,
            profile: RuleProfile::Baseline,
        };
        self.new_agency_with(sphere, name, attributes)
    }

    pub fn new_agency_with(&mut self, sphere: Sphere, name: &str, attributes: AgencyAttributes) -> Keypair {
        let authority = Keypair::new();
        let ix = self.ix(
            solutio::instruction::RegisterAgency {
                sphere,
                name: name.to_string(),
                attributes,
            },
            solutio::accounts::RegisterAgency {
                payer: self.sponsor.pubkey(),
                registry_authority: self.registry_authority.pubkey(),
                registry: self.registry(),
                agency_authority: authority.pubkey(),
                agency: self.agency(&authority.pubkey()),
                system_program: system_program::ID,
            },
        );
        let ra = self.registry_authority.insecure_clone();
        self.send(ix, &[&ra]).unwrap();
        authority
    }

    pub fn create_ata(&mut self, manager: &Keypair, label: &str, supplier: &Pubkey, days_valid: i64) -> Pubkey {
        let ata_id = hash(label);
        let manager_agency = self.agency(&manager.pubkey());
        let ata = self.ata_pda(&manager_agency, &ata_id);
        let now = self.now();
        let ix = self.ix(
            solutio::instruction::CreateAta {
                ata_id,
                supplier: *supplier,
                doc_hash: hash(&format!("{label}-doc")),
                valid_from: now - DAY,
                valid_until: now + days_valid * DAY,
            },
            solutio::accounts::CreateAta {
                payer: self.sponsor.pubkey(),
                manager_authority: manager.pubkey(),
                manager_agency,
                ata,
                system_program: system_program::ID,
            },
        );
        self.send(ix, &[manager]).unwrap();
        ata
    }

    pub fn try_add_item(
        &mut self,
        manager: &Keypair,
        ata: &Pubkey,
        item_no: u16,
        qty: u64,
        max_adhesion: u64,
    ) -> Result<Pubkey, u32> {
        let item = self.item_pda(ata, item_no);
        let ix = self.ix(
            solutio::instruction::AddItem {
                item_no,
                registered_qty: qty,
                max_adhesion_qty: max_adhesion,
                unit_price: UNIT_PRICE,
            },
            solutio::accounts::AddItem {
                payer: self.sponsor.pubkey(),
                manager_authority: manager.pubkey(),
                manager_agency: self.agency(&manager.pubkey()),
                ata: *ata,
                item,
                system_program: system_program::ID,
            },
        );
        self.send(ix, &[manager]).map(|_| item)
    }

    pub fn add_item(&mut self, manager: &Keypair, ata: &Pubkey, item_no: u16, qty: u64, max_adhesion: u64) -> Pubkey {
        self.try_add_item(manager, ata, item_no, qty, max_adhesion).unwrap()
    }

    pub fn set_status(&mut self, manager: &Keypair, ata: &Pubkey, status: AtaStatus) -> Result<(), u32> {
        let ix = self.ix(
            solutio::instruction::SetAtaStatus {
                status,
                evidence_hash: hash("despacho-status"),
            },
            solutio::accounts::SetAtaStatus {
                manager_authority: manager.pubkey(),
                manager_agency: self.agency(&manager.pubkey()),
                ata: *ata,
            },
        );
        self.send(ix, &[manager])
    }

    pub fn request_ex(
        &mut self,
        adherent: &Keypair,
        ata: &Pubkey,
        item: &Pubkey,
        request_id: u64,
        qty: u64,
        exception: AdhesionException,
    ) -> Result<Pubkey, u32> {
        let agency = self.agency(&adherent.pubkey());
        let request = self.request_pda(item, &agency, request_id);
        let ix = self.ix(
            solutio::instruction::RequestAdhesion {
                request_id,
                qty,
                exception,
                evidence_hash: hash("oficio-justificativa"),
            },
            solutio::accounts::RequestAdhesion {
                payer: self.sponsor.pubkey(),
                adherent_authority: adherent.pubkey(),
                adherent_agency: agency,
                ata: *ata,
                item: *item,
                usage: self.usage_pda(item, &agency),
                request,
                system_program: system_program::ID,
            },
        );
        self.send(ix, &[adherent]).map(|_| request)
    }

    pub fn request(
        &mut self,
        adherent: &Keypair,
        ata: &Pubkey,
        item: &Pubkey,
        request_id: u64,
        qty: u64,
    ) -> Result<Pubkey, u32> {
        self.request_ex(adherent, ata, item, request_id, qty, AdhesionException::None)
    }

    pub fn supplier_respond(
        &mut self,
        supplier: &Keypair,
        ata: &Pubkey,
        item: &Pubkey,
        adherent: &Keypair,
        request: &Pubkey,
        accept: bool,
    ) -> Result<(), u32> {
        let agency = self.agency(&adherent.pubkey());
        let ix = self.ix(
            solutio::instruction::SupplierRespond { accept },
            solutio::accounts::SupplierRespond {
                supplier: supplier.pubkey(),
                ata: *ata,
                item: *item,
                usage: self.usage_pda(item, &agency),
                request: *request,
            },
        );
        self.send(ix, &[supplier])
    }

    fn decision_accounts(
        &self,
        manager: &Keypair,
        ata: &Pubkey,
        item: &Pubkey,
        adherent: &Keypair,
        request: &Pubkey,
    ) -> solutio::accounts::ManagerDecision {
        let agency = self.agency(&adherent.pubkey());
        solutio::accounts::ManagerDecision {
            manager_authority: manager.pubkey(),
            manager_agency: self.agency(&manager.pubkey()),
            ata: *ata,
            item: *item,
            usage: self.usage_pda(item, &agency),
            request: *request,
        }
    }

    pub fn authorize(
        &mut self,
        manager: &Keypair,
        ata: &Pubkey,
        item: &Pubkey,
        adherent: &Keypair,
        request: &Pubkey,
        qty: u64,
        evidence: [u8; 32],
    ) -> Result<(), u32> {
        let accounts = self.decision_accounts(manager, ata, item, adherent, request);
        let ix = self.ix(
            solutio::instruction::AuthorizeAdhesion {
                authorized_qty: qty,
                evidence_hash: evidence,
            },
            accounts,
        );
        self.send(ix, &[manager])
    }

    pub fn deny(
        &mut self,
        manager: &Keypair,
        ata: &Pubkey,
        item: &Pubkey,
        adherent: &Keypair,
        request: &Pubkey,
    ) -> Result<(), u32> {
        let accounts = self.decision_accounts(manager, ata, item, adherent, request);
        let ix = self.ix(
            solutio::instruction::DenyAdhesion {
                evidence_hash: hash("motivacao"),
            },
            accounts,
        );
        self.send(ix, &[manager])
    }

    pub fn extend(
        &mut self,
        manager: &Keypair,
        ata: &Pubkey,
        item: &Pubkey,
        adherent: &Keypair,
        request: &Pubkey,
        new_execute_by: i64,
    ) -> Result<(), u32> {
        let accounts = self.decision_accounts(manager, ata, item, adherent, request);
        let ix = self.ix(solutio::instruction::ExtendExecution { new_execute_by }, accounts);
        self.send(ix, &[manager])
    }

    pub fn formalize(&mut self, adherent: &Keypair, request: &Pubkey) -> Result<(), u32> {
        let r: AdhesionRequest = self.fetch(request);
        let ix = self.ix(
            solutio::instruction::FormalizeAdhesion {
                evidence_hash: hash("nota-de-empenho"),
            },
            solutio::accounts::FormalizeAdhesion {
                adherent_authority: adherent.pubkey(),
                adherent_agency: self.agency(&adherent.pubkey()),
                ata: r.ata,
                request: *request,
            },
        );
        self.send(ix, &[adherent])
    }

    /// Permissionless: only the sponsor (fee payer) signs.
    pub fn expire(&mut self, item: &Pubkey, adherent: &Keypair, request: &Pubkey) -> Result<(), u32> {
        let agency = self.agency(&adherent.pubkey());
        let ix = self.ix(
            solutio::instruction::ExpireAdhesion {},
            solutio::accounts::ExpireAdhesion {
                item: *item,
                usage: self.usage_pda(item, &agency),
                request: *request,
            },
        );
        self.send(ix, &[])
    }

    /// request -> supplier accepts -> manager authorizes in full -> agency executes.
    pub fn full_adhesion(
        &mut self,
        manager: &Keypair,
        supplier: &Keypair,
        adherent: &Keypair,
        ata: &Pubkey,
        item: &Pubkey,
        request_id: u64,
        qty: u64,
    ) -> Result<Pubkey, u32> {
        let req = self.request(adherent, ata, item, request_id, qty)?;
        self.supplier_respond(supplier, ata, item, adherent, &req, true)?;
        self.authorize(manager, ata, item, adherent, &req, qty, [0u8; 32])?;
        self.formalize(adherent, &req)?;
        Ok(req)
    }

    // ---- obligations ----

    pub fn register_obligation(
        &mut self,
        debtor: &Keypair,
        label: &str,
        creditor: &Pubkey,
        amount: u64,
        source_request: Option<Pubkey>,
    ) -> Result<Pubkey, u32> {
        let id = hash(label);
        let debtor_agency = self.agency(&debtor.pubkey());
        let obligation = self.obligation_pda(&debtor_agency, &id);
        let ix = self.ix(
            solutio::instruction::RegisterObligation {
                obligation_id: id,
                creditor: *creditor,
                verified_amount: amount,
                evidence_hash: hash(&format!("{label}-liquidacao")),
            },
            solutio::accounts::RegisterObligation {
                payer: self.sponsor.pubkey(),
                debtor_authority: debtor.pubkey(),
                debtor_agency,
                obligation,
                source_request,
                system_program: system_program::ID,
            },
        );
        self.send(ix, &[debtor]).map(|_| obligation)
    }

    pub fn attach_doc(&mut self, debtor: &Keypair, obligation: &Pubkey, nfe_key: &str, amount: u64) -> Result<(), u32> {
        let h = hash(nfe_key);
        let ix = self.ix(
            solutio::instruction::AttachFiscalDocument {
                doc_key_hash: h,
                amount,
            },
            solutio::accounts::AttachFiscalDocument {
                payer: self.sponsor.pubkey(),
                debtor_authority: debtor.pubkey(),
                debtor_agency: self.agency(&debtor.pubkey()),
                obligation: *obligation,
                fiscal_document: self.fiscal_doc_pda(&h),
                system_program: system_program::ID,
            },
        );
        self.send(ix, &[debtor])
    }

    pub fn mark_eligible_as(&mut self, verifier: &Keypair, obligation: &Pubkey, amount: u64) -> Result<(), u32> {
        let ix = self.ix(
            solutio::instruction::MarkEligible {
                eligible_amount: amount,
                evidence_hash: hash("parecer-elegibilidade"),
            },
            solutio::accounts::MarkEligible {
                verifier: verifier.pubkey(),
                registry: self.registry(),
                obligation: *obligation,
            },
        );
        self.send(ix, &[verifier])
    }

    pub fn mark_eligible(&mut self, obligation: &Pubkey, amount: u64) -> Result<(), u32> {
        let v = self.verifier.insecure_clone();
        self.mark_eligible_as(&v, obligation, amount)
    }

    pub fn finance(
        &mut self,
        financier: &Keypair,
        creditor: &Keypair,
        obligation: &Pubkey,
        amount: u64,
        notice: [u8; 32],
    ) -> Result<Pubkey, u32> {
        let o: Obligation = self.fetch(obligation);
        let n = o.financing_count;
        let financing = self.financing_pda(obligation, n);
        let ix = self.ix(
            solutio::instruction::Finance {
                financing_no: n,
                amount,
                notice_evidence_hash: notice,
            },
            solutio::accounts::Finance {
                payer: self.sponsor.pubkey(),
                financier: financier.pubkey(),
                creditor: creditor.pubkey(),
                obligation: *obligation,
                financing,
                system_program: system_program::ID,
            },
        );
        self.send(ix, &[financier, creditor]).map(|_| financing)
    }

    pub fn reduce(&mut self, debtor: &Keypair, obligation: &Pubkey, amount: u64) -> Result<(), u32> {
        let ix = self.ix(
            solutio::instruction::RecordReduction {
                amount,
                evidence_hash: hash("glosa"),
            },
            solutio::accounts::DebtorUpdate {
                debtor_authority: debtor.pubkey(),
                debtor_agency: self.agency(&debtor.pubkey()),
                obligation: *obligation,
            },
        );
        self.send(ix, &[debtor])
    }

    pub fn pay(&mut self, debtor: &Keypair, obligation: &Pubkey, amount: u64) -> Result<(), u32> {
        let ix = self.ix(
            solutio::instruction::RecordPayment {
                amount,
                evidence_hash: hash("ordem-bancaria"),
            },
            solutio::accounts::DebtorUpdate {
                debtor_authority: debtor.pubkey(),
                debtor_agency: self.agency(&debtor.pubkey()),
                obligation: *obligation,
            },
        );
        self.send(ix, &[debtor])
    }
}
