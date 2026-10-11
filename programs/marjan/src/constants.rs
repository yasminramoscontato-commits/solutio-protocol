use anchor_lang::prelude::*;

#[constant]
pub const REGISTRY_SEED: &[u8] = b"registry";
#[constant]
pub const AGENCY_SEED: &[u8] = b"agency";
#[constant]
pub const ATA_SEED: &[u8] = b"ata";
#[constant]
pub const ITEM_SEED: &[u8] = b"item";
#[constant]
pub const USAGE_SEED: &[u8] = b"usage";
#[constant]
pub const REQUEST_SEED: &[u8] = b"request";
#[constant]
pub const OBLIGATION_SEED: &[u8] = b"obligation";
#[constant]
pub const FISCAL_DOC_SEED: &[u8] = b"fiscal_doc";
#[constant]
pub const FINANCING_SEED: &[u8] = b"financing";

/// Days a pending request (awaiting the supplier or the manager) keeps its
/// reserved quantity before anyone may lapse it. Design choice, not a legal
/// deadline: it stops an unanswered request from holding the balance forever.
pub const RESPONSE_WINDOW_SECS: i64 = 90 * 86_400;
