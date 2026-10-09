// Measures real fees and rent deposits of the recorded devnet run (client/devnet-run.json).
import fs from "node:fs";
import { Connection } from "@solana/web3.js";
const run = JSON.parse(fs.readFileSync(new URL("./devnet-run.json", import.meta.url)));
const conn = new Connection(run.rpc, "confirmed");
const out = [];
for (const s of run.steps) {
  let tx = null;
  for (let i = 0; i < 5 && !tx; i++) {
    try { tx = await conn.getTransaction(s.signature, { commitment: "confirmed", maxSupportedTransactionVersion: 0 }); }
    catch { await new Promise((r) => setTimeout(r, 1500)); }
  }
  const m = tx.meta;
  // Sponsor is account index 0 (fee payer); its balance change = fee + rent it deposited.
  const sponsorDelta = m.preBalances[0] - m.postBalances[0];
  out.push({ label: s.label, outcome: s.outcome, fee_lamports: m.fee, sponsor_spent_lamports: sponsorDelta,
    rent_lamports: sponsorDelta - m.fee, compute_units: m.computeUnitsConsumed ?? null, signature: s.signature });
  await new Promise((r) => setTimeout(r, 250));
}
fs.writeFileSync(new URL("./devnet-costs.json", import.meta.url), JSON.stringify({ measured_at: new Date().toISOString(), program: run.program, steps: out }, null, 2));
const sum = (k) => out.reduce((a, s) => a + s[k], 0);
console.log(`transactions=${out.length} fees=${sum("fee_lamports")} rent=${sum("rent_lamports")} cu=${sum("compute_units")}`);
