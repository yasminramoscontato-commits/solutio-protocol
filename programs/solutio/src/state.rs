use anchor_lang::prelude::*;

use crate::rules::Balances;

/// Level of government an agency belongs to (relevant to Law 14.133/2021, art. 86, §8).
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Copy, Debug, PartialEq, Eq, InitSpace)]
pub enum Sphere {
    Federal,
    State,
    District,
    Municipal,
}

/// Which procurement regulation governs an agency's own conduct as an adherent.
/// Law 14.133/2021 applies to everyone; each sphere regulates art. 86 for itself.
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Copy, Debug, PartialEq, Eq, InitSpace)]
pub enum RuleProfile {
    /// Law 14.133/2021 art. 86 as regulated federally by Decree 11.462/2023.
    Baseline,
    /// State of Alagoas, Decree 95.019/2023 (adds the art. 33 restriction on
    /// adhering to municipal records).
    Alagoas,
}

// ---------------------------------------------------------------------------
// Registry and identities
// ---------------------------------------------------------------------------

/// Singleton configuration. In the MVP the registry authority is a clearly
/// labeled demo issuer standing in for a real credential issuer (for example an
/// audit court or a state government using the Solana Attestation Service).
#[account]
#[derive(InitSpace)]
pub struct Registry {
    pub authority: Pubkey,
    /// Signer allowed to confirm that an obligation is eligible for assignment.
    pub eligibility_verifier: Pubkey,
    pub agency_count: u32,
    pub bump: u8,
}

/// A public agency recognized by the registry. `authority` is the wallet that
/// signs on its behalf (in production, a multisig).
/// Facts about an agency attested by the registry issuer when it is registered.
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Copy, Debug, PartialEq, Eq)]
pub struct AgencyAttributes {
    pub is_health_ministry: bool,
    pub is_state_capital: bool,
    pub profile: RuleProfile,
}

#[account]
#[derive(InitSpace)]
pub struct Agency {
    pub authority: Pubkey,
    pub sphere: Sphere,
    /// Attested by the registry: relevant to the health-emergency exception
    /// (Decree 11.462/2023, art. 32, §1).
    pub is_health_ministry: bool,
    /// Attested by the registry: the municipality is a state capital
    /// (Alagoas Decree 95.019/2023, art. 33).
    pub is_state_capital: bool,
    /// Regulation that governs this agency when it adheres to a record.
    pub profile: RuleProfile,
    #[max_len(64)]
    pub name: String,
    pub active: bool,
    pub bump: u8,
}

// ---------------------------------------------------------------------------
// Module 1: Carona (price records and adhesions)
// ---------------------------------------------------------------------------

/// Status of a price record. Suspension models a supplier sanction during which
/// no new contracts may derive from the record (Decree 11.462/2023, art. 28, §1);
/// cancellation models arts. 28-29.
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Copy, Debug, PartialEq, Eq, InitSpace)]
pub enum AtaStatus {
    Active,
    Suspended,
    Cancelled,
}

/// A price-registration record (Ata de Registro de Preços).
#[account]
#[derive(InitSpace)]
pub struct Ata {
    pub manager_agency: Pubkey,
    pub manager_sphere: Sphere,
    pub manager_is_health_ministry: bool,
    pub manager_is_state_capital: bool,
    /// Wallet of the registered supplier, who must accept each adhesion.
    pub supplier: Pubkey,
    /// Hash of the canonical identifier (e.g. the PNCP control number).
    pub ata_id: [u8; 32],
    /// Hash of the published record document.
    pub doc_hash: [u8; 32],
    pub valid_from: i64,
    pub valid_until: i64,
    pub status: AtaStatus,
    pub status_evidence_hash: [u8; 32],
    pub item_count: u16,
    pub bump: u8,
}

/// One item of a price record. `registered_qty` is immutable once created:
/// there is deliberately no instruction to increase it (Decree art. 23).
#[account]
#[derive(InitSpace)]
pub struct AtaItem {
    pub ata: Pubkey,
    pub item_no: u16,
    /// Quantity registered for the managing and participating agencies.
    pub registered_qty: u64,
    /// Maximum quantity the tender allows for non-participants (Decree art. 15, XI).
    /// Zero means the tender does not allow adhesions.
    pub max_adhesion_qty: u64,
    /// Unit price in integer cents.
    pub unit_price: u64,
    /// Pending + authorized quantity of adhesions subject to the §5 cap.
    pub committed_capped: u64,
    /// Pending + authorized quantity of adhesions under a statutory exception.
    pub committed_exempt: u64,
    /// Quantity authorized by the manager (informational; includes exempt).
    pub authorized_total: u64,
    pub bump: u8,
}

/// Quantity a given agency has committed (pending + authorized) on an item (§4).
#[account]
#[derive(InitSpace)]
pub struct AgencyItemUsage {
    pub item: Pubkey,
    pub agency: Pubkey,
    pub committed: u64,
    pub bump: u8,
}

/// Exception to the §5 cap claimed by an adhesion (Decree 11.462/2023, art. 32).
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Copy, Debug, PartialEq, Eq, InitSpace)]
pub enum AdhesionException {
    None,
    /// §1: emergency purchase of medicines or medical supplies under a record
    /// managed by the Ministry of Health.
    HealthEmergency,
    /// §2: state, district or municipal adhesion required for voluntary
    /// transfers executing a federal programme.
    FederalProgramTransfer,
}

/// Lifecycle of an adhesion, in the order required by Decree 11.462/2023,
/// art. 31, §1: the manager authorizes only after the supplier accepts.
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Copy, Debug, PartialEq, Eq, InitSpace)]
pub enum RequestStatus {
    /// Requested by the agency; quantity reserved.
    Requested,
    /// Accepted by the supplier; awaiting the manager.
    SupplierAccepted,
    /// Authorized by the manager; must be executed within the deadline.
    Authorized,
    /// Executed: contract or commitment note formalized within the deadline.
    Executed,
    /// Declined by the supplier or denied by the manager; quantity released.
    Rejected,
    /// Not executed within the deadline; quantity released.
    Lapsed,
}

#[account]
#[derive(InitSpace)]
pub struct AdhesionRequest {
    pub ata: Pubkey,
    pub item: Pubkey,
    pub adherent_agency: Pubkey,
    pub supplier: Pubkey,
    pub request_id: u64,
    pub requested_qty: u64,
    pub authorized_qty: u64,
    pub exception: AdhesionException,
    pub status: RequestStatus,
    /// Hash of the request (justification of advantage, price compatibility,
    /// and, for exceptions, the supporting evidence).
    pub evidence_hash: [u8; 32],
    /// Hash of the manager's justification (required for partial authorization or denial).
    pub decision_evidence_hash: [u8; 32],
    /// Hash of the contract or commitment note that executed the adhesion.
    pub execution_evidence_hash: [u8; 32],
    pub requested_at: i64,
    pub supplier_decided_at: i64,
    pub authorized_at: i64,
    pub execute_by: i64,
    pub bump: u8,
}

// ---------------------------------------------------------------------------
// Module 2: Obligations and financing
// ---------------------------------------------------------------------------

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Copy, Debug, PartialEq, Eq, InitSpace)]
pub enum ObligationStatus {
    /// Debt recognized by the debtor (liquidation). Not yet financeable.
    Verified,
    /// A designated verifier confirmed an amount eligible for assignment.
    Eligible,
    /// At least part of the eligible amount has been financed.
    Financed,
    /// A reduction left less owed than what was financed. Visible to everyone.
    Impaired,
    /// Fully paid by the debtor.
    Settled,
}

#[account]
#[derive(InitSpace)]
pub struct Obligation {
    pub debtor_agency: Pubkey,
    pub creditor: Pubkey,
    /// Hash of the canonical administrative identifier (e.g. managing unit + commitment number + fiscal year).
    pub obligation_id: [u8; 32],
    pub verified_amount: u64,
    pub documented_amount: u64,
    pub reductions: u64,
    pub eligible_amount: u64,
    pub financed_amount: u64,
    pub paid_amount: u64,
    pub financing_count: u32,
    pub document_count: u32,
    pub status: ObligationStatus,
    pub evidence_hash: [u8; 32],
    pub eligibility_evidence_hash: [u8; 32],
    /// Adhesion this obligation derives from, or the default key if none.
    pub source_request: Pubkey,
    pub bump: u8,
}

impl Obligation {
    pub fn balances(&self) -> Balances {
        Balances {
            verified: self.verified_amount,
            documented: self.documented_amount,
            reductions: self.reductions,
            eligible: self.eligible_amount,
            financed: self.financed_amount,
            paid: self.paid_amount,
        }
    }

    pub fn store(&mut self, b: Balances) {
        self.verified_amount = b.verified;
        self.documented_amount = b.documented;
        self.reductions = b.reductions;
        self.eligible_amount = b.eligible;
        self.financed_amount = b.financed;
        self.paid_amount = b.paid;
    }
}

/// A fiscal document (e.g. NF-e) bound to exactly one obligation. Its address is
/// derived from the hash of the document key, so the same document cannot be
/// attached to a second obligation inside the protocol.
#[account]
#[derive(InitSpace)]
pub struct FiscalDocument {
    pub obligation: Pubkey,
    pub doc_key_hash: [u8; 32],
    pub amount: u64,
    pub bump: u8,
}

#[account]
#[derive(InitSpace)]
pub struct Financing {
    pub obligation: Pubkey,
    pub financier: Pubkey,
    pub financing_no: u32,
    pub amount: u64,
    /// Hash of the evidence that the debtor was notified of the assignment.
    pub notice_evidence_hash: [u8; 32],
    pub created_at: i64,
    pub bump: u8,
}
