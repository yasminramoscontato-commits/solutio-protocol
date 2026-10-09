# Legal rules implemented — traceability matrix

Each row links a legal source to the code that enforces it and to the tests that prove the behavior. Profile: **Brazil, Law 14.133/2021**. Unit tests live in `programs/solutio/src/rules.rs`; integration tests in `programs/solutio/tests/`.

## Module 1 — Carona (art. 86)

| # | Rule | Source | Enforced in | Tests |
| --- | --- | --- | --- | --- |
| C1 | Each non-participating agency may adhere to at most 50% of the quantity registered for the managing and participating agencies | Art. 86, §4 | `rules::individual_cap`, `rules::check_adhesion` | `art86_par4_individual_cap_is_half_rounded_down`; `art86_par4_each_agency_is_capped_at_half_the_registered_quantity` |
| C2 | The sum of adhesions to an item may not exceed twice its registered quantity, regardless of the number of adherents | Art. 86, §5 | `rules::global_cap`, `rules::check_adhesion` | `art86_par5_global_cap_is_twice_registered`; `art86_par5_total_adhesions_are_capped_at_twice_the_registered_quantity`; `race_for_the_last_units_only_one_can_win` |
| C3 | Statutory exceptions lift the §5 cap only; §4 still applies | Art. 86, §§6–7 | `AtaItem.global_cap_exempt`, `rules::check_adhesion` | `art86_exception_lifts_par5_but_not_par4`; `statutory_exception_lifts_par5_but_par4_still_applies` |
| C4 | Federal bodies may not adhere to state, district or municipal price records | Art. 86, §8 | `rules::check_adhesion` | `art86_par8_federal_cannot_adhere_to_non_federal`; `art86_par8_federal_agency_cannot_adhere_to_a_state_record`; `federal_agency_can_adhere_to_a_federal_record` |
| C5 | Adhesion requires prior consultation and acceptance by the managing agency and the supplier | Art. 86, §2 | three-step flow: `request_adhesion` → `approve_adhesion` → `accept_adhesion` | `adhesion_needs_request_approval_and_supplier_acceptance`; `only_the_managing_agency_can_approve`; `rejected_requests_never_consume_balance` |
| C6 | No adhesion outside the record's validity | Art. 84 and record clauses | `rules::check_adhesion` (re-checked at acceptance) | `validity_window_and_basic_guards`; `expired_record_rejects_requests_and_late_acceptances` |
| C7 | Registered quantities cannot be increased | Usual record clause ("vedado efetuar acréscimos") | no instruction modifies `registered_qty` | by construction |
| C8 | The managing agency is not a non-participant of its own record | Definition of non-participant, art. 6, XLIX | `rules::check_adhesion` | `the_manager_cannot_adhere_to_its_own_record` |

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
| O9 | An obligation linked to an adhesion requires that adhesion to be effective and to match debtor and creditor | Cross-module consistency | `register_obligation` | `an_obligation_can_only_derive_from_an_effective_adhesion` |
| O10 | Only the debtor records payments and reductions | Debtor's administrative acts | `DebtorUpdate` constraints | `only_the_debtor_can_record_payments_and_reductions` |

## Open interpretation questions (part of research question QP1)

1. **Rounding of the 50% cap** for odd registered quantities (50% of 75 = 37.5). We apply the floor, the conservative reading that never authorizes more than the statute allows.
2. **Reserved balance:** should an approved but not yet executed adhesion count toward the caps? Today only effective (supplier-accepted) adhesions count; caps are re-checked at acceptance.
3. **Extension of the record:** does an extension renew the quantities available for adhesion?
4. **Who attests a statutory exception** (§§6–7)? Today the managing agency declares it when registering the item.
5. **Assignment regime** for claims against each level of government: required notice, consent and the role of the bank-domicile link.
6. **90-day execution window** after approval (federal regulation and usual record clauses) is not enforced yet.
