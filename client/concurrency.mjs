// Concurrency test on devnet: competing transactions sent at the same time
// against the same state, without waiting for each other.
//
// Round 1: six agencies each request 50 units of an item whose adhesion pool is
// 200 (100 registered x 2). All six transactions are signed with the same
// blockhash and submitted in parallel. Expected: exactly four succeed, two fail
// with ExceedsGlobalCap, and the item ends at 200.
//
// Round 2: two financiers try to finance the last R$ 1.000,00 of the same
// obligation at the same time. Expected: exactly one succeeds.
//
// What this shows: the validator serializes transactions that write the same
// account, and each one re-checks the rules against the state left by the
// previous one. What it does not show: behaviour under mainnet congestion, or
// across leaders, beyond what one devnet run can exercise.

import {
  BN, SystemProgram, agencyPda, ataPda, connection, explorer, fetchTx, financingPda, fiscalDocPda, itemPda, key,
  obligationPda, outcomeOf, program, recorder, registryPda, requestPda, sha, signed, sponsor, usagePda,
} from "./lib.mjs";

const { run, save, step } = recorder("concurrency-run.json", { kind: "concurrency" });
const tag = Date.now().toString(36);
const plain = { isHealthMinistry: false, isStateCapital: false, profile: { baseline: {} } };

async function race(label, txs, expectOk) {
  const sigs = await Promise.all(txs.map((tx) => connection.sendRawTransaction(tx.serialize(), { skipPreflight: true })));
  const results = [];
  for (const sig of sigs) {
    const info = await fetchTx(sig);
    results.push({ signature: sig, slot: info.slot, outcome: outcomeOf(info.meta.err), explorer: explorer(sig) });
  }
  const ok = results.filter((r) => r.outcome === "ok").length;
  const slots = [...new Set(results.map((r) => r.slot))];
  run.races = run.races ?? [];
  run.races.push({ label, submitted: txs.length, succeeded: ok, expected_succeeded: expectOk, slots, results });
  save();
  console.log(`${ok === expectOk ? "✔" : "✘"} ${label}: ${ok}/${txs.length} succeeded (expected ${expectOk}); slots ${slots.join(", ")}`);
  for (const r of results) console.log(`    ${r.outcome.padEnd(22)} slot ${r.slot}  ${r.explorer}`);
  if (ok !== expectOk) throw new Error(`race "${label}" did not match expectation`);
  return results;
}

async function main() {
  const central = key("central-de-compras");
  const supplier = key("fornecedor");
  const verifier = key("verifier");
  const cities = [1, 2, 3, 4, 5, 6].map((i) => key(`conc-city-${i}-${tag}`));
  const funds = [key("financiador-a"), key("financiador-b")];

  for (const [i, c] of cities.entries()) {
    await step(`Register agency DEMO Prefeitura C${i + 1} ${tag}`, "Demo issuer", "ok",
      program.methods.registerAgency({ municipal: {} }, `DEMO Prefeitura C${i + 1} ${tag}`, plain).accountsStrict({
        payer: sponsor.publicKey, registryAuthority: sponsor.publicKey, registry: registryPda,
        agencyAuthority: c.publicKey, agency: agencyPda(c.publicKey), systemProgram: SystemProgram.programId,
      }));
  }
  const now = Math.floor(Date.now() / 1000);
  const ataId = sha(`DEMO-ARP-CONC-${tag}`);
  const mgr = agencyPda(central.publicKey);
  const ata = ataPda(mgr, ataId);
  await step("Create price record for the race", "", "ok",
    program.methods.createAta(ataId, supplier.publicKey, sha("conc-doc"), new BN(now - 86_400), new BN(now + 365 * 86_400))
      .accountsStrict({ payer: sponsor.publicKey, managerAuthority: central.publicKey, managerAgency: mgr, ata, systemProgram: SystemProgram.programId }),
    [central]);
  const item = itemPda(ata, 1);
  await step("Add item: 100 registered, adhesion pool 200", "Art. 86 §5", "ok",
    program.methods.addItem(1, new BN(100), new BN(200), new BN(20_000)).accountsStrict({
      payer: sponsor.publicKey, managerAuthority: central.publicKey, managerAgency: mgr, ata, item, systemProgram: SystemProgram.programId,
    }), [central]);

  // ---- Round 1: six simultaneous requests for a pool that fits four.
  const { blockhash } = await connection.getLatestBlockhash("confirmed");
  const txs = [];
  for (const c of cities) {
    const ag = agencyPda(c.publicKey);
    txs.push(await signed(
      program.methods.requestAdhesion(new BN(1), new BN(50), { none: {} }, sha(`oficio-${c.publicKey}`)).accountsStrict({
        payer: sponsor.publicKey, adherentAuthority: c.publicKey, adherentAgency: ag, ata, item,
        usage: usagePda(item, ag), request: requestPda(item, ag, 1), systemProgram: SystemProgram.programId,
      }), [c], blockhash));
  }
  const r1 = await race("Six agencies request 50 units each at once (pool of 200)", txs, 4);
  const it = await program.account.ataItem.fetch(item);
  run.item_committed_after_race = it.committedCapped.toString();
  save();
  console.log(`    item committed after the race: ${it.committedCapped.toString()} of 200`);
  if (it.committedCapped.toNumber() !== 200) throw new Error("cap broken");

  // ---- Round 2: two financiers race for the last R$ 1.000,00 of one obligation.
  const winnerIdx = r1.findIndex((r) => r.outcome === "ok");
  const city = cities[winnerIdx];
  const ag = agencyPda(city.publicKey);
  const req = requestPda(item, ag, 1);
  await step("Supplier accepts", "", "ok",
    program.methods.supplierRespond(true).accountsStrict({ supplier: supplier.publicKey, ata, item, usage: usagePda(item, ag), request: req }), [supplier]);
  await step("Manager authorizes 50", "", "ok",
    program.methods.authorizeAdhesion(new BN(50), Array(32).fill(0)).accountsStrict({
      managerAuthority: central.publicKey, managerAgency: mgr, ata, item, usage: usagePda(item, ag), request: req,
    }), [central]);
  await step("Agency executes", "", "ok",
    program.methods.formalizeAdhesion(sha("empenho-conc")).accountsStrict({ adherentAuthority: city.publicKey, adherentAgency: ag, ata, request: req }), [city]);
  const obId = sha(`DEMO-NE-CONC-${tag}`);
  const ob = obligationPda(ag, obId);
  await step("Verified obligation of R$ 10.000,00", "", "ok",
    program.methods.registerObligation(obId, supplier.publicKey, new BN(1_000_000), sha("liq-conc")).accountsStrict({
      payer: sponsor.publicKey, debtorAuthority: city.publicKey, debtorAgency: ag, obligation: ob, sourceRequest: req, systemProgram: SystemProgram.programId,
    }), [city]);
  const nfe = sha(`DEMO-NFE-CONC-${tag}`);
  await step("Fiscal document", "", "ok",
    program.methods.attachFiscalDocument(nfe, new BN(1_000_000)).accountsStrict({
      payer: sponsor.publicKey, debtorAuthority: city.publicKey, debtorAgency: ag, obligation: ob, fiscalDocument: fiscalDocPda(nfe), systemProgram: SystemProgram.programId,
    }), [city]);
  await step("Eligibility confirmed", "", "ok",
    program.methods.markEligible(new BN(1_000_000), sha("parecer-conc")).accountsStrict({ verifier: verifier.publicKey, registry: registryPda, obligation: ob }), [verifier]);
  await step("Financier A finances R$ 9.000,00", "", "ok",
    program.methods.finance(0, new BN(900_000), sha("notif-0")).accountsStrict({
      payer: sponsor.publicKey, financier: funds[0].publicKey, creditor: supplier.publicKey, obligation: ob,
      financing: financingPda(ob, 0), systemProgram: SystemProgram.programId,
    }), [funds[0], supplier]);

  const bh2 = (await connection.getLatestBlockhash("confirmed")).blockhash;
  const fin = await Promise.all(funds.map((f, i) => signed(
    program.methods.finance(1, new BN(100_000), sha(`notif-race-${i}`)).accountsStrict({
      payer: sponsor.publicKey, financier: f.publicKey, creditor: supplier.publicKey, obligation: ob,
      financing: financingPda(ob, 1), systemProgram: SystemProgram.programId,
    }), [f, supplier], bh2)));
  await race("Two financiers race for the last R$ 1.000,00 at once", fin, 1);
  const o = await program.account.obligation.fetch(ob);
  run.obligation_financed_after_race = o.financedAmount.toString();
  run.finished_at = new Date().toISOString();
  save();
  console.log(`    obligation financed after the race: ${o.financedAmount.toString()} of 1000000`);
  if (o.financedAmount.toNumber() !== 1_000_000) throw new Error("single financing broken");
}

main().catch((e) => { console.error(e); process.exit(1); });
