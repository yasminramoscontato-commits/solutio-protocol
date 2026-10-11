# Operational flows at a state central purchasing body (anonymized)

This page records the founder's operational knowledge of how a Brazilian state central purchasing body handles price records. It is anonymized: no names of people, suppliers or bodies, no process numbers and no internal documents are published. The standard dispatches at the end are templates rebuilt with placeholders.

The flows follow the Alagoas regulation: Decree 95.019/2023, arts. 30–34, and Decree 90.391/2023, arts. 2–4.

## Flow A: adhesion by a non-participating body

| # | Who | Step | Where it is recorded |
| --- | --- | --- | --- |
| 1 | Requesting body | Need memo; head of the body authorizes; market research identifies the record as the cheapest valid reference | Electronic process (SEI) |
| 2 | Requesting body | Budget availability confirmed | SEI |
| 3 | Requesting body | **Supplier consulted by e-mail**; acceptance attached as PDF | E-mail, SEI |
| 4 | Central purchasing body, record-use unit | **Limit dispatch**: 50% of the registered quantity, rounded down ("observada a impossibilidade de fracionamento") | SEI, internal control |
| 5 | Requesting body | Demand readjusted to the limit, sometimes split across items with the same description and price; adhesion requested in Compras.gov.br with the acceptance PDF | Compras.gov.br, SEI |
| 6 | Central purchasing body | **Adhesion authorization**: three signatures, conditions of art. 31, 90-day deadline in free text | SEI, internal control |
| 7 | Requesting body | Technical study, terms of reference, legal opinion; government and budget committees (SEGOV, CPOF); contract or commitment note | SEI, accounting system |

Observed duration in one case: **51 days** from step 1 to step 6.

## Flow B: use by a participating body (supply authorization)

| # | Who | Step | Where it is recorded |
| --- | --- | --- | --- |
| 1 | Participating body | Memo and request form: record, item, quantity, budget line, contract manager, delivery place and schedule | SEI |
| 2 | Central purchasing body | **One supply authorization per record**, "tendo em vista que existe saldo de demanda"; items of records where the body is not a participant are refused | SEI, internal control |
| 3 | Central purchasing body | Consolidated dispatch with a table per item (initial quantity, authorized, remaining). Above the value threshold, the process goes to SEGOV (Decree 90.391, art. 3). After use, **"baixa definitiva do saldo"** (final write-off); if unused, **"restituição do saldo"** (restoration) | SEI, internal control |
| 4 | Government and budget committees | Deliberation; possible referral back for review; quantities revised | SEI |
| 5 | Central purchasing body | Authorizations re-issued as **"RETIFICAÇÃO"** | SEI, internal control |
| 6 | Participating body | Supply order and commitment note sent directly to the supplier; invoice issued in the body's name | Accounting system |

Observed duration in one case: about **five months** from step 1 to step 5.

## Flow C: checking whether the state already has a record before an external adhesion

1. The process arrives at the record-use unit.
2. The unit manager checks whether the external record the body wants to join is in the file.
3. The process is assigned to an analyst.
4. The analyst checks **control spreadsheets** for valid state records with the same object. A record-control system on the intranet is replacing them; it is fed by each dispatch issued in SEI.
5. A verification dispatch is issued. The planning unit then checks whether an intention-to-register (IRP) procedure is open.

## What each step becomes in Marjan

| Today | In Marjan | Rule |
| --- | --- | --- |
| Supplier acceptance by e-mail, attached as PDF | The supplier signs the acceptance; no attachment to check | PROC-04 |
| 50% computed by hand in a dispatch | Computed and enforced when quantity is reserved, rounded down | PROC-01 |
| Balance in spreadsheets or an intranet system, fed after the dispatch | Public balance per item, updated by the act itself | PROC-03, PROC-14 |
| The same adhesion in SEI, Compras.gov.br and internal control | One state that every system reads; the SEI dispatch carries the transaction link | PROC-13 |
| 90-day deadline written in the dispatch | Deadline stored; lapse releases quantity automatically | PROC-06 |
| Participant quota checked by hand; refusal for non-participants | Quota per participant; non-participants refused by the record | PROC-11 |
| Rectified authorizations re-issued | Rectification releases the difference to the balance | PROC-12 |
| "Is there a valid state record for this object?" checked in spreadsheets | Query of valid records by object code (CATMAT/CATSER) | roadmap |
| Split across identical items | Per-object share reported for review | PROC-15 |

## Standard dispatches (templates)

Rebuilt with placeholders from the structure of standard dispatches. Text in Portuguese, as used in practice.

### 1. Limit dispatch for an adhesion request

> **DESPACHO – [SIGLA]-[Nº]-[MÊS]-[ANO]**
> À [unidade solicitante],
> Em atenção à solicitação de consulta quanto à existência de saldo disponível na Ata de Registro de Preços nº [ATA], informamos que, para o item [ITEM], cujo quantitativo total registrado corresponde a [Q] unidades, o quantitativo passível de solicitação por adesão corresponde a 50% do total registrado, nos termos do art. 86, § 4º, da Lei nº 14.133/2021 e do art. 32, I, do Decreto Estadual nº 95.019/2023.
> Considerando o quantitativo total de [Q] unidades, o limite de 50% corresponde a [⌊Q/2⌋] unidades, observada a impossibilidade de fracionamento da unidade.
> Encaminhem-se os autos para ciência e providências, observando-se o limite acima indicado e a existência de saldo disponível na Ata.

### 2. Adhesion authorization

> **DESPACHO – AUTORIZAÇÃO PARA ADESÃO Nº [Nº]-[MÊS]-[ANO]**
> O presente processo trata da solicitação do(a) [ÓRGÃO NÃO PARTICIPANTE], com o objetivo de adesão à Ata de Registro de Preços nº [ATA].
>
> | Nº item | Descritivo | Quant. solicitada | Percentual de adesão |
> | --- | --- | --- | --- |
> | [ITEM] | [DESCRIÇÃO] | [QTD] | [QTD/Q %] |
>
> Fornecedor: [RAZÃO SOCIAL], [CNPJ].
> Condicionantes (art. 31 do Decreto Estadual nº 95.019/2023): requisição formal com justificativa de vantajosidade; análise da entidade gerenciadora quanto ao saldo e aos limites do art. 86, §§ 3º a 5º, da Lei nº 14.133/2021; autorização expressa da gerenciadora; instrução completa (ETP, TR, estimativa de preços, parecer jurídico, análises SEGOV e CPOF); formalização no prazo de 90 dias contados desta autorização, respeitada a vigência da ata; concordância do fornecedor beneficiário.

### 3. Supply authorization to a participating body

> **AUTORIZAÇÃO PARA FORNECIMENTO Nº [SIGLA]-[Nº]-[MÊS]-[ANO]**
> Órgão de origem: [ÓRGÃO PARTICIPANTE] · Ata nº [ATA] · Objeto: [OBJETO] · Pregão eletrônico nº [PREGÃO]
> Autorizamos o fornecimento dos produtos abaixo relacionados ao órgão participante, tendo em vista que existe saldo de demanda para a aquisição pretendida.
>
> | Item | Descrição | Quant. | Valor unit. | Valor total |
> | --- | --- | --- | --- | --- |
>
> Dotação orçamentária: [PT / fonte / natureza / elemento]. Gestor contratual: [cargo]. Local e forma de entrega: [..]. Prazo de entrega: [N] dias após a ordem de fornecimento acompanhada da nota de empenho. Pagamento: [N] dias.
> A gerenciadora emite apenas a autorização, para fins de controle das atas; a responsabilidade financeira é do órgão solicitante, que requisitará o fornecimento por ordem de fornecimento e nota de empenho. A nota fiscal deve ser emitida em nome do órgão solicitante.

### 4. Consolidated dispatch and balance control

> **DESPACHO – [SIGLA]-[Nº]-[MÊS]-[ANO]**
> Trata o presente de solicitação do(a) [ÓRGÃO] com vistas à utilização das Atas de Registro de Preços nº [ATAS]. Verificamos constar instrução com: solicitação; indicação da dotação; autorização do ordenador de despesa; local de execução; gestor contratual; justificativa.
>
> | Ata | Nº do item | Quant. inicial | Quant. autorizada | Saldo remanescente |
> | --- | --- | --- | --- | --- |
>
> Conforme o art. 3º do Decreto Estadual nº 90.391/2023, os autos seguem à SEGOV para análise de compatibilidade e deliberação. Finalizados os procedimentos de utilização, os autos devem retornar a esta Agência para a **baixa definitiva do saldo** da ata ou, em caso de não utilização, a **restituição do saldo** correspondente.
