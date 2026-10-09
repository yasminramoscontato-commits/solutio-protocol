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
#[account]
#[derive(InitSpace)]
pub struct Agency {
    pub authority: Pubkey,
    pub sphere: Sphere,
    #[max_len(64)]
    pub name: String,
    pub active: bool,
    pub bump: u8,
}

// ---------------------------------------------------------------------------
// Module 1: Carona (price records and adhesions)
// ---------------------------------------------------------------------------

/// A price-registration record (Ata de Registro de Preços).
#[account]
#[derive(InitSpace)]
pub struct Ata {
    pub manager_agency: Pubkey,
    pub manager_sphere: Sphere,
    /// Wallet of the registered supplier, who must accept each adhesion.
    pub supplier: Pubkey,
    /// Hash of the canonical identifier (e.g. the PNCP control number).
    pub ata_id: [u8; 32],
    /// Hash of the published record document.
    pub doc_hash: [u8; 32],
    pub valid_from: i64,
    pub valid_until: i64,
    pub item_count: u16,
    pub bump: u8,
}

/// One item of a price record. `registered_qty` is immutable once created:
/// there is deliberately no instruction to increase it (no "acréscimos").
#[account]
#[derive(InitSpace)]
pub struct AtaItem {
    pub ata: Pubkey,
    pub item_no: u16,
    pub registered_qty: u64,
    /// Unit price in integer cents.
    pub unit_price: u64,
    /// Sum of all effective adhesions to this item.
    pub adhered_total: u64,
    /// True when a statutory exception lifts the §5 global cap.
    pub global_cap_exempt: bool,
    pub bump: u8,
}

/// Cumulative quantity a given agency has adhered to on a given item (§4).
#[account]
#[derive(InitSpace)]
pub struct AgencyItemUsage {
    pub item: Pubkey,
    pub agency: Pubkey,
    pub consumed: u64,
    pub bump: u8,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Copy, Debug, PartialEq, Eq, InitSpace)]
pub enum RequestStatus {
    Requested,
    Approved,
    Effective,
    Rejected,
}

#[account]
#[derive(InitSpace)]
pub struct AdhesionRequest {
    pub ata: Pubkey,
    pub item: Pubkey,
    pub adherent_agency: Pubkey,
    pub supplier: Pubkey,
    pub request_id: u64,
    pub qty: u64,
    pub status: RequestStatus,
    /// Hash of the agency's request (ofício / justification).
    pub evidence_hash: [u8; 32],
    pub requested_at: i64,
    pub decided_at: i64,
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
