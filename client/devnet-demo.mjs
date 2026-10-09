// Solutio — scripted demo against the program deployed on Solana devnet.
//
// Every step is a real devnet transaction. Steps that the law forbids are
// submitted on purpose with preflight disabled, so the rejection itself is
// recorded on-chain and can be inspected in the explorer.
//
// All agencies, suppliers and financiers are DEMO identities with fictional
// names. No real government data is used.
//
// Usage: node devnet-demo.mjs   (needs ~/.config/solana/devnet-deployer.json funded on devnet)

import fs from "node:fs";
import path from "node:path";
import os from "node:os";
import crypto from "node:crypto";
import { fileURLToPath } from "node:url";
import anchor from "@anchor-lang/core";
import { Connection, Keypair, PublicKey, SystemProgram, Transaction } from "@solana/web3.js";

const { AnchorProvider, Program, Wallet, BN } = anchor;
const here = path.dirname(fileURLToPath(import.meta.url));
const RPC = process.env.SOLUTIO_RPC ?? "https://api.devnet.solana.com";
const idl = JSON.parse(fs.readFileSync(path.join(here, "idl/solutio.json"), "utf8"));
const PROGRAM_ID = new PublicKey(idl.address);

// ---------- keys ----------
const keyDir = path.join(here, ".keys");
fs.mkdirSync(keyDir, { recursive: true });
function key(name) {
  const f = path.join(keyDir, `${name}.json`);
  if (!fs.existsSync(f)) fs.writeFileSync(f, JSON.stringify(Array.from(Keypair.generate().secretKey)));
  return Keypair.fromSecretKey(Uint8Array.from(JSON.parse(fs.readFileSync(f, "utf8"))));
}
const sponsor = Keypair.fromSecretKey(
  Uint8Array.from(JSON.parse(fs.readFileSync(path.join(os.homedir(), ".config/solana/devnet-deployer.json"), "utf8"))),
);

const connection = new Connection(RPC, "confirmed");
const provider = new AnchorProvider(connection, new Wallet(sponsor), { commitment: "confirmed" });
const program = new Program(idl, provider);

// ---------- helpers ----------
const sha = (s) => Array.from(crypto.createHash("sha256").update(s).digest());
const pda = (...seeds) => PublicKey.findProgramAddressSync(seeds, PROGRAM_ID)[0];
const u16 = (n) => { const b = Buffer.alloc(2); b.writeUInt16LE(n); return b; };
const u32 = (n) => { const b = Buffer.alloc(4); b.writeUInt32LE(n); return b; };
const u64 = (n) => { const b = Buffer.alloc(8); b.writeBigUInt64LE(BigInt(n)); return b; };
const enc = (s) => Buffer.from(s);

const registryPda = pda(enc("registry"));
const agencyPda = (auth) => pda(enc("agency"), auth.toBuffer());
const ataPda = (mgr, id) => pda(enc("ata"), mgr.toBuffer(), Buffer.from(id));
const itemPda = (ata, n) => pda(enc("item"), ata.toBuffer(), u16(n));
const usagePda = (item, ag) => pda(enc("usage"), item.toBuffer(), ag.toBuffer());
const requestPda = (item, ag, id) => pda(enc("request"), item.toBuffer(), ag.toBuffer(), u64(id));
const obligationPda = (debtor, id) => pda(enc("obligation"), debtor.toBuffer(), Buffer.from(id));
const fiscalDocPda = (h) => pda(enc("fiscal_doc"), Buffer.from(h));
const financingPda = (ob, n) => pda(enc("financing"), ob.toBuffer(), u32(n));

const errorsByCode = Object.fromEntries((idl.errors ?? []).map((e) => [e.code, e]));
const explorer = (sig) => `https://explorer.solana.com/tx/${sig}?cluster=devnet`;

const runFile = path.join(here, "devnet-run.json");
const run = { rpc: RPC, program: PROGRAM_ID.toBase58(), started_at: new Date().toISOString(), steps: [] };

/** Sends one instruction (sponsor pays) and records the on-chain outcome. */
async function step(label, basis, expect, builder, signers = []) {
  const ix = await builder.instruction();
  const tx = new Transaction().add(ix);
  tx.feePayer = sponsor.publicKey;
  const { blockhash, lastValidBlockHeight } = await connection.getLatestBlockhash("confirmed");
  tx.recentBlockhash = blockhash;
  tx.sign(sponsor, ...signers);
  const sig = await connection.sendRawTransaction(tx.serialize(), { skipPreflight: expect !== "ok" });
  await connection.confirmTransaction({ signature: sig, blockhash, lastValidBlockHeight }, "confirmed");
  const info = await connection.getTransaction(sig, { commitment: "confirmed", maxSupportedTransactionVersion: 0 });
  const err = info?.meta?.err ?? null;
  let outcome = "ok";
  if (err) {
    const custom = err?.InstructionError?.[1]?.Custom;
    outcome = custom !== undefined ? errorsByCode[custom]?.name ?? `Custom(${custom})` : JSON.stringify(err);
  }
  const pass = expect === outcome;
  run.steps.push({ label, basis, expected: expect, outcome, pass, signature: sig, explorer: explorer(sig) });
  fs.writeFileSync(runFile, JSON.stringify(run, null, 2));
  console.log(`${pass ? "✔" : "✘"} ${label} — ${outcome}\n    ${explorer(sig)}`);
  if (!pass) throw new Error(`Unexpected outcome for "${label}": expected ${expect}, got ${outcome}`);
  return sig;
}

const plain = { isHealthMinistry: false, isStateCapital: false, profile: { baseline: {} } };
async function registerAgency(kp, sphere, name, attributes = plain) {
  return step(`Register agency: ${name}`, "Demo issuer (registry authority)", "ok",
    program.methods.registerAgency({ [sphere]: {} }, name, attributes).accountsStrict({
      payer: sponsor.publicKey, registryAuthority: sponsor.publicKey, registry: registryPda,
      agencyAuthority: kp.publicKey, agency: agencyPda(kp.publicKey), systemProgram: SystemProgram.programId,
    }));
}

async function createAta(manager, label, supplier) {
  const now = Math.floor(Date.now() / 1000);
  const id = sha(label);
  const mgrAgency = agencyPda(manager.publicKey);
  const ata = ataPda(mgrAgency, id);
  await step(`Create price record ${label}`, "Law 14.133 art. 82; Decree 11.462 art. 22 (validity)", "ok",
    program.methods.createAta(id, supplier.publicKey, sha(`${label}-document`), new BN(now - 86_400), new BN(now + 365 * 86_400))
      .accountsStrict({ payer: sponsor.publicKey, managerAuthority: manager.publicKey, managerAgency: mgrAgency, ata, systemProgram: SystemProgram.programId }),
    [manager]);
  return ata;
}

async function addItem(manager, ata, itemNo, qty, maxAdhesion) {
  const item = itemPda(ata, itemNo);
  await step(`Add item ${itemNo}: ${qty} registered, adhesion maximum ${maxAdhesion}`, "Decree 11.462 art. 15 XI; art. 86 §5", "ok",
    program.methods.addItem(itemNo, new BN(qty), new BN(maxAdhesion), new BN(1_250)).accountsStrict({
      payer: sponsor.publicKey, managerAuthority: manager.publicKey, managerAgency: agencyPda(manager.publicKey),
      ata, item, systemProgram: SystemProgram.programId,
    }), [manager]);
  return item;
}

async function requestAdhesion(adherent, name, ata, item, reqId, qty, basis, expect = "ok") {
  const agency = agencyPda(adherent.publicKey);
  const request = requestPda(item, agency, reqId);
  await step(`${name} requests ${qty} units`, basis, expect,
    program.methods.requestAdhesion(new BN(reqId), new BN(qty), { none: {} }, sha(`${name}-oficio-${reqId}`)).accountsStrict({
      payer: sponsor.publicKey, adherentAuthority: adherent.publicKey, adherentAgency: agency, ata, item,
      usage: usagePda(item, agency), request, systemProgram: SystemProgram.programId,
    }), [adherent]);
  return request;
}

// ---------- scenario ----------
async function main() {
  console.log(`Program ${PROGRAM_ID.toBase58()} on ${RPC}\nSponsor ${sponsor.publicKey.toBase58()}\n`);

  const verifier = key("verifier");
  const central = key("central-de-compras");
  const supplier = key("fornecedor");
  const cities = ["prefeitura-a", "prefeitura-b", "prefeitura-c", "prefeitura-d", "prefeitura-e"].map(key);
  const ministry = key("ministerio-federal");
  const interior = key("prefeitura-interior");
  const stateSecretariat = key("secretaria-estadual-al");
  const fundA = key("financiador-a");
  const fundB = key("financiador-b");

  if (!(await connection.getAccountInfo(registryPda))) {
    await step("Initialize registry", "Demo issuer; designates the eligibility verifier", "ok",
      program.methods.initRegistry(verifier.publicKey).accountsStrict({
        payer: sponsor.publicKey, authority: sponsor.publicKey, registry: registryPda, systemProgram: SystemProgram.programId,
      }));
  }

  // Fresh identities each run (agency PDAs are unique per authority key).
  const tag = Date.now().toString(36);
  const alProfile = { ...plain, profile: { alagoas: {} } };
  await registerAgency(central, "state", `DEMO Central de Compras ${tag}`, alProfile).catch(skipIfExists);
  for (const [i, c] of cities.entries()) await registerAgency(c, "municipal", `DEMO Prefeitura ${"ABCDE"[i]} ${tag}`).catch(skipIfExists);
  await registerAgency(ministry, "federal", `DEMO Ministerio Federal ${tag}`).catch(skipIfExists);
  await registerAgency(interior, "municipal", `DEMO Prefeitura do Interior ${tag}`).catch(skipIfExists);
  await registerAgency(stateSecretariat, "state", `DEMO Secretaria Estadual AL ${tag}`, alProfile).catch(skipIfExists);

  // Module 1 — Carona
  const ata = await createAta(central, `DEMO-ARP-${tag}`, supplier);
  const item = await addItem(central, ata, 1, 100, 200);

  const reqA = await requestAdhesion(cities[0], "Prefeitura A", ata, item, 1, 50, "Art. 86 §4: up to 50% per agency");
  await requestAdhesion(cities[0], "Prefeitura A", ata, item, 2, 1, "Art. 86 §4: 51% would exceed the individual cap", "ExceedsIndividualCap");
  await requestAdhesion(cities[1], "Prefeitura B", ata, item, 1, 50, "Art. 86 §5: running total 100 of 200");
  await requestAdhesion(cities[2], "Prefeitura C", ata, item, 1, 50, "Art. 86 §5: running total 150 of 200");
  await requestAdhesion(cities[3], "Prefeitura D", ata, item, 1, 50, "Art. 86 §5: running total 200 of 200");
  await requestAdhesion(cities[4], "Prefeitura E", ata, item, 1, 1, "Art. 86 §5: the item's adhesion pool is exhausted", "ExceedsGlobalCap");
  await requestAdhesion(ministry, "Ministerio Federal", ata, item, 1, 10, "Art. 86 §8: federal agency to a state record", "FederalAdhesionForbidden");

  const interiorAta = await createAta(interior, `DEMO-ARP-MUN-${tag}`, supplier);
  const interiorItem = await addItem(interior, interiorAta, 1, 100, 200);
  await requestAdhesion(stateSecretariat, "Secretaria Estadual (AL)", interiorAta, interiorItem, 1, 10,
    "Alagoas Decree 95.019/2023 art. 33: state agency to a non-capital municipal record", "MunicipalAdhesionForbidden");

  const cityA = agencyPda(cities[0].publicKey);
  await step("Supplier accepts Prefeitura A's adhesion by signature", "Decree 11.462 art. 31 §1 (acceptance before authorization)", "ok",
    program.methods.supplierRespond(true).accountsStrict({ supplier: supplier.publicKey, ata, item, usage: usagePda(item, cityA), request: reqA }),
    [supplier]);
  await step("Managing agency authorizes 50 units", "Decree 11.462 art. 31 §1-2 (90-day execution window starts)", "ok",
    program.methods.authorizeAdhesion(new BN(50), Array(32).fill(0)).accountsStrict({
      managerAuthority: central.publicKey, managerAgency: agencyPda(central.publicKey), ata, item, usage: usagePda(item, cityA), request: reqA,
    }), [central]);
  await step("Prefeitura A executes the purchase (nota de empenho)", "Decree 11.462 art. 34", "ok",
    program.methods.formalizeAdhesion(sha("nota-de-empenho-A")).accountsStrict({
      adherentAuthority: cities[0].publicKey, adherentAgency: cityA, request: reqA,
    }), [cities[0]]);

  // Module 2 — Obligations
  const obId = sha(`DEMO-NE-${tag}`);
  const obligation = obligationPda(cityA, obId);
  await step("Prefeitura A records a verified obligation of R$ 10.000,00", "Lei 4.320 art. 63 (liquidação); source = executed adhesion", "ok",
    program.methods.registerObligation(obId, supplier.publicKey, new BN(1_000_000), sha("liquidacao-A")).accountsStrict({
      payer: sponsor.publicKey, debtorAuthority: cities[0].publicKey, debtorAgency: cityA, obligation, sourceRequest: reqA,
      systemProgram: SystemProgram.programId,
    }), [cities[0]]);
  const nfe = sha(`DEMO-NFE-${tag}`);
  await step("Attach fiscal document (hash of the NF-e key)", "One document backs one obligation (PDA by document hash)", "ok",
    program.methods.attachFiscalDocument(nfe, new BN(1_000_000)).accountsStrict({
      payer: sponsor.publicKey, debtorAuthority: cities[0].publicKey, debtorAgency: cityA, obligation,
      fiscalDocument: fiscalDocPda(nfe), systemProgram: SystemProgram.programId,
    }), [cities[0]]);
  await step("Designated verifier confirms eligibility", "Verified ≠ eligible: separate signed step", "ok",
    program.methods.markEligible(new BN(1_000_000), sha("parecer-elegibilidade")).accountsStrict({
      verifier: verifier.publicKey, registry: registryPda, obligation,
    }), [verifier]);

  const finance = (fund, n, amount, label, expect = "ok") =>
    step(label, "CC art. 290 (notice of assignment); cumulative financing ≤ eligible", expect,
      program.methods.finance(n, new BN(amount), sha(`notificacao-${n}`)).accountsStrict({
        payer: sponsor.publicKey, financier: fund.publicKey, creditor: supplier.publicKey, obligation,
        financing: financingPda(obligation, n), systemProgram: SystemProgram.programId,
      }), [fund, supplier]);
  await finance(fundA, 0, 600_000, "Financier A advances R$ 6.000,00 (creditor co-signs)");
  await finance(fundB, 1, 400_000, "Financier B advances R$ 4.000,00");
  await finance(fundA, 2, 1, "Financier A tries to finance R$ 0,01 more", "ExceedsFinanceableBalance");

  await step("Prefeitura A records payment of R$ 10.000,00", "Lei 4.320 arts. 64-65 (ordem bancária)", "ok",
    program.methods.recordPayment(new BN(1_000_000), sha("ordem-bancaria-A")).accountsStrict({
      debtorAuthority: cities[0].publicKey, debtorAgency: cityA, obligation,
    }), [cities[0]]);

  // Agencies never held SOL.
  const balances = await Promise.all([central, ...cities, supplier, fundA].map((k) => connection.getBalance(k.publicKey)));
  run.agency_wallet_lamports = balances;
  run.accounts = { registry: registryPda.toBase58(), ata: ata.toBase58(), item: item.toBase58(), obligation: obligation.toBase58() };
  run.finished_at = new Date().toISOString();
  fs.writeFileSync(runFile, JSON.stringify(run, null, 2));
  console.log(`\nSigner wallets' SOL balances (lamports): ${balances.join(", ")}`);
  console.log(`All ${run.steps.length} steps matched expectations. Log: ${runFile}`);
}

function skipIfExists(e) {
  // Agency keys persist across runs; re-registering the same key is refused by design.
  if (/already in use|custom program error: 0x0/i.test(String(e))) return;
  throw e;
}

main().catch((e) => { console.error(e); process.exit(1); });
