// Measures real fees, rent deposits and rent refunds of a recorded run
// (client/devnet-run.json, or its .localnet.json twin when MARJAN_RPC is local).
import fs from "node:fs";
import path from "node:path";
import { IS_LOCAL, fetchTx, here, sleep } from "./lib.mjs";

const name = IS_LOCAL ? "devnet-run.localnet.json" : "devnet-run.json";
const out = IS_LOCAL ? "devnet-costs.localnet.json" : "devnet-costs.json";
const run = JSON.parse(fs.readFileSync(path.join(here, name)));
const steps = [];
for (const s of run.steps) {
  const m = (await fetchTx(s.signature)).meta;
  // The sponsor is account 0 (fee payer): its balance change is the fee plus the
  // rent it deposited (negative when rent comes back from a closed account).
  const spent = m.preBalances[0] - m.postBalances[0];
  steps.push({
    label: s.label, outcome: s.outcome, fee_lamports: m.fee, sponsor_spent_lamports: spent,
    rent_lamports: spent - m.fee, compute_units: m.computeUnitsConsumed ?? null, signature: s.signature,
  });
  if (!IS_LOCAL) await sleep(250);
}
fs.writeFileSync(path.join(here, out),
  JSON.stringify({ measured_at: new Date().toISOString(), program: run.program, rpc: run.rpc, steps }, null, 2));
const sum = (k) => steps.reduce((a, s) => a + s[k], 0);
console.log(`transactions=${steps.length} fees=${sum("fee_lamports")} net_rent=${sum("rent_lamports")} cu=${sum("compute_units")}`);
