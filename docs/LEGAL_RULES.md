# Legal rules implemented — traceability matrix

Each row links a legal source to the code that enforces it and to the tests that prove the behavior. Profiles: **Baseline — Law 14.133/2021, art. 86, as regulated in the federal sphere by Decree 11.462/2023, arts. 31–33**, and **Alagoas — State Decree 95.019/2023** (see the profile section below). Balance accounting mirrors the federal *Gestão de Atas* tool (Compras.gov.br and SIASGnet). Unit tests live in `programs/marjan/src/rules.rs`; integration tests in `programs/marjan/tests/`.

Sources consulted: Law 14.133/2021; Decree 11.462/2023; Alagoas Decrees 95.019/2023 and 90.391/2023; *Manual de Gestão de Atas de Registro de Preços* (Compras.gov.br); *Guia Prático – Gestão de Ata SRP/SIASGnet* (legacy guide written under the revoked Decree 7.892/2013, used here only for its balance formulas).

## Module 1 — Carona

| # | Rule | Source | Enforced in | Tests |
| --- | --- | --- | --- | --- |
| C1 | Each non-participating agency may adhere to at most 50% of the quantity registered for the managing and participating agencies | Law art. 86 §4; Decree art. 32, I | `rules::individual_cap`, `rules::check_reservation` | `art86_par4_individual_cap_is_half_rounded_down`; `art86_par4_each_agency_is_capped_at_half_the_registered_quantity` |
| C2 | All adhesions to an item together may not exceed twice its registered quantity | Law art. 86 §5; Decree art. 32, II | `rules::adhesion_cap`, `rules::check_reservation` | `art86_par5_cap_is_twice_registered_or_the_tender_maximum`; `art86_par5_total_adhesions_are_capped_at_twice_the_registered_quantity`; `race_for_the_last_units_only_one_can_win` |
| C3 | The tender may set a lower maximum for non-participants, or forbid adhesions | Decree art. 15, XI; "Permitir adesões" flag in the federal tool | `AtaItem.max_adhesion_qty`, `add_item` | `the_tender_can_set_a_lower_maximum_or_forbid_adhesions` |
| C4 | Quantities awaiting authorization already count against the caps | Federal tool formula: *saldo para adesões = máximo − (aguardando autorização + autorizada)* | reservation at `request_adhesion` | `pending_requests_reserve_quantity_as_in_the_federal_system`; `randomized_reservations_and_releases_never_break_caps` |
| C5 | The manager authorizes only after the supplier accepts | Law art. 86 §2, III; Decree art. 31, III and §1 | `request_adhesion` → `supplier_respond` → `authorize_adhesion` | `decree_order_supplier_accepts_before_the_manager_authorizes` |
| C6 | The manager may authorize part of the quantity, with justification; denial also requires justification | *Manual de Gestão de Atas* ("aceitar parcialmente", "negar") | `authorize_adhesion`, `deny_adhesion` | `partial_authorization_releases_the_remainder_and_requires_justification`; `declined_and_denied_requests_release_their_reservation` |
| C7 | Emergency purchases of medicines and medical supplies under a Ministry of Health record are exempt from the §5 cap | Law art. 86 §7; Decree art. 32 §1 | `rules::exception_applies` (`HealthEmergency`), `Agency.is_health_ministry` | `exceptions_lift_par5_only_and_only_when_applicable`; `health_emergency_exception_requires_a_ministry_of_health_record` |
| C8 | Subnational adhesions required for voluntary transfers of a federal programme are exempt from the §5 cap — only under records managed by the **federal** Executive | Law art. 86 §6; Decree art. 32 §2 | `rules::exception_applies` (`FederalProgramTransfer` requires a federal manager and a non-federal adherent) | `exceptions_lift_par5_only_and_only_when_applicable`; `federal_programme_transfer_exception_is_for_subnational_agencies_only` |
| C9 | Exceptions never lift the §4 individual cap | Law art. 86 §§4, 6–7 | `rules::check_reservation` | `health_emergency_exception_requires_a_ministry_of_health_record` |
| C10 | Federal bodies may not adhere to state, district or municipal records | Law art. 86 §8; Decree art. 33 | `rules::check_reservation` | `art86_par8_federal_cannot_adhere_to_non_federal`; `art86_par8_federal_agency_cannot_adhere_to_a_state_record` |
| C11 | After authorization, the purchase must be executed within 90 days, within the record's validity | Decree art. 31 §2 | `rules::execution_deadline`, `formalize_adhesion`, permissionless `expire_adhesion` | `execution_deadline_is_ninety_days_capped_by_validity`; `ninety_day_execution_window_extension_and_lapse` |
| C12 | The deadline may be extended exceptionally, never past the record's validity | Decree art. 31 §3 | `extend_execution` | `ninety_day_execution_window_extension_and_lapse` |
| C13 | No adhesion outside the record's validity | Law art. 84; Decree art. 22 | `rules::check_reservation`, `authorize_adhesion` | `status_validity_and_basic_guards`; `expired_record_rejects_requests_and_late_authorizations` |
| C14 | A suspended record (supplier sanction) or a cancelled record accepts no new contracts; cancellation is final | Decree arts. 28 §1, 28–29 | `set_ata_status`, `AtaStatus` | `suspended_or_cancelled_records_accept_no_new_adhesions` |
| C15 | Registered quantities cannot be increased | Decree art. 23 | no instruction modifies `registered_qty` | by construction |
| C16 | The managing agency does not adhere to its own record | Law art. 6, XLIX; federal tool rule | `rules::check_reservation` | `the_manager_cannot_adhere_to_its_own_record` |
| C18 | Alagoas state agencies may not adhere to records managed by municipal agencies, except those of state capitals | Alagoas Decree 95.019/2023, art. 33 | `rules::profile_allows`, `Agency.profile`, `Agency.is_state_capital` | `alagoas_art33_state_cannot_adhere_to_municipal_records_except_capitals`; `alagoas_profile_blocks_state_adhesion_to_non_capital_municipal_records` |
| C19 | No execution (contract or commitment note) while the record is suspended or cancelled; the deadline keeps running | Decree 11.462/2023 art. 28 §1; Decree AL 95.019/2023 art. 28 §1 (reading: executing an authorized adhesion is a new contract) | `formalize_adhesion` | `m3_no_execution_while_the_record_is_suspended` |
| C20 | A request left unanswered by the supplier or the manager for 90 days can be lapsed by anyone and its quantity returns | Design choice (no legal deadline exists); prevents an unanswered request from holding the balance | `expire_adhesion`, `RESPONSE_WINDOW_SECS` | `l2_unanswered_requests_lapse_after_the_response_window` |
| C17 | An obligation derived from an adhesion requires the adhesion to have been executed | Decree art. 34 (contract or commitment note) | `register_obligation` | `an_obligation_can_only_derive_from_an_executed_adhesion` |

## Profile: Alagoas (Decree 95.019/2023)

Alagoas regulates arts. 82–86 for its direct, autarchic and foundational administration with a text that closely follows the federal decree. Article-by-article comparison of what matters for Marjan:

| Topic | Alagoas 95.019/2023 | Federal 11.462/2023 | In Marjan |
| --- | --- | --- | --- |
| Order: supplier accepts, then manager authorizes; 90 days to execute; extension within validity | art. 31 §§1–3 | art. 31 §§1–3 | Same code path (C5, C11, C12) |
| 50% per agency; 2x in total | art. 32, I–II | art. 32, I–II | Same (C1, C2) |
| Emergency medicines under a Ministry of Health record exempt from 2x | art. 32 §1 (state agencies) | art. 32 §1 | Same (C7) |
| Adhesion that may be required for voluntary transfers | art. 32 §2: **municipal** adhesion, for state transfers | art. 32 §2: subnational adhesion, federal programmes | The 2x exemption comes from Law art. 86 §6, which covers only federal records: a state programme does not lift the cap (C8) |
| Restriction on adhering to other spheres | art. 33: **state agencies may not adhere to municipal records, except state capitals** | art. 33: federal agencies may not adhere to non-federal records | New rule C18, active under the `Alagoas` profile |
| Tool for balances and adhesion requests | art. 24: the federal *Gestão de Atas* (Compras.gov.br), via Termo de Acesso (arts. 5–6) | art. 24 equivalent | Marjan complements, not replaces, the official tool |
| Adhering to a record managed outside Alagoas | art. 7, XI: the state managing body deliberates on it | — | Not modeled: human gate, recordable as evidence |

**Approval gates from Alagoas Decree 90.391/2023 (not modeled, recorded as evidence when relevant):** AMGESP's Director-President homologates price-record tenders (art. 2, III); processes above R$ 350,000.00 go to SEGOV after the State Attorney's Office (art. 3); secretaries ratify contracts arising from adhesions once SEGOV has checked the demand against government priorities (art. 4).

## Module 2 — Obligations

| # | Rule | Source | Enforced in | Tests |
| --- | --- | --- | --- | --- |
| O1 | A debt is recognized by liquidation (origin, amount, creditor) | Law 4.320/1964, art. 63 | `register_obligation` (signed by the debtor agency) | `full_lifecycle_finances_once_then_settles` |
| O2 | Liquidation does not by itself make a claim assignable; eligibility is a separate, signed confirmation | Design decision (see DECISIONS.md) | `mark_eligible` restricted to `Registry.eligibility_verifier` | `eligibility_is_separate_from_verification`; `verified_is_not_eligible_eligibility_is_a_separate_signed_step` |
| O3 | Eligibility must be backed by fiscal documents; documents cannot exceed the verified amount | Design decision | `rules::attach_document`, `rules::mark_eligible` | same as O2; `the_same_fiscal_document_cannot_back_two_obligations` |
| O4 | A fiscal document backs at most one obligation | Design decision | `FiscalDocument` PDA derived from the document key hash | `the_same_fiscal_document_cannot_back_two_obligations` |
| O5 | The cumulative financed amount never exceeds the eligible amount | Design decision (anti double financing) | `rules::finance` | `the_same_balance_cannot_be_financed_twice`; `full_lifecycle_finances_once_then_settles`; `randomized_ledger_sequences_keep_invariants` |
| O6 | Assignment is effective against the debtor only after notice | Civil Code, art. 290 | `finance` requires creditor signature and a notice evidence hash | `financing_requires_the_creditor_and_notice_evidence` |
| O7 | Withholdings and disallowances are recorded, never refused; they cap eligibility and may impair financed claims | Debtor's administrative acts | `rules::apply_reduction`, status `Impaired` | `reductions_cap_eligibility_and_flag_impairment`; `a_disallowance_after_financing_marks_the_obligation_impaired` |
| O8 | Payments cannot exceed the outstanding amount; full payment settles the obligation | Law 4.320/1964, payment stage | `rules::apply_payment` | `payments_close_financing_and_settle`; `full_lifecycle_finances_once_then_settles` |
| O9 | An obligation linked to an adhesion requires that adhesion to be executed and to match debtor and creditor | Cross-module consistency; Decree art. 34 | `register_obligation` | `an_obligation_can_only_derive_from_an_executed_adhesion` |
| O11 | Obligations citing one adhesion together never exceed its authorized value (quantity × registered unit price) | Cross-module consistency (audit M-2) | `register_obligation` | `m2_obligations_cannot_exceed_the_value_of_their_source_adhesion` |
| O10 | Only the debtor records payments and reductions | Debtor's administrative acts | `DebtorUpdate` constraints | `only_the_debtor_can_record_payments_and_reductions` |

## Open interpretation questions (part of research question QP1)

1. **Rounding of the 50% cap** for odd registered quantities (50% of 75 = 37.5). We apply the floor. This is now supported: TCU Acórdão 547/2026-Plenário, item 9.1.1.6, recommends rounding down after finding the federal system rounded up (para. 67), and the founder observed the same practice at a state central purchasing body.
2. **Exempt quantities and the §5 pool.** We track exempt adhesions separately, so they neither consume nor are blocked by the 2x pool. A stricter reading would count them toward the pool for later non-exempt adhesions.
3. **Participants adhering to other items** (Decree art. 31 §4): an agency that integrates the record may adhere to items for which it has no registered quantity. The program models only the manager; participant quotas, supply authorizations and rectification are specified and tested in the framework (PROC-11, PROC-12) and are the next program feature.
9. **Per item or per object?** Art. 86 §4 is written per item. A body that splits its demand across items with the same description and price stays within every per-item cap; whether the 50% should also apply per object is open. The framework reports the per-object share (PROC-15) without judging it.
4. **Extension of the record** (Decree art. 22): whether an extension renews the quantities available for adhesion. Several model records state it "may" be renewed; not modeled yet.
5. **Who attests the exceptions.** Today the registry attests that an agency is the Ministry of Health, and the adherent states the purpose with an evidence hash. The legal judgment that the purchase is an emergency, or that it executes a federal programme, remains human.
6. **Subnational regulations.** Decree 11.462/2023 governs the federal sphere. States and municipalities issue their own regulations; each one becomes a configuration profile once validated. Alagoas is the first (C18). Art. 33 of Decree 95.019/2023 reads: "Fica vedada aos órgãos e às entidades da Administração Pública estadual a adesão a ata de registro de preços gerenciada por órgão ou entidade municipal, excetuando-se aquelas gerenciadas por municípios que sejam capital de Estado e do Distrito Federal." (wording confirmed by the founder, 2026-10-09).
8. **Which regulation governs a cross-sphere adhesion.** We apply the adherent's regulation to restrictions on *who may adhere* (art. 33 of each decree) and the statute's caps to the record. Whether the manager's regulation can add further limits for outside adherents is open.
7. **Assignment regime** for claims against each level of government: required notice, consent and the role of the bank-domicile link.
