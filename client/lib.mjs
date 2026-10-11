// Shared helpers for the Marjan devnet scripts.
import fs from "node:fs";
import path from "node:path";
import os from "node:os";
import crypto from "node:crypto";
import { fileURLToPath } from "node:url";
import anchor from "@anchor-lang/core";
import { Connection, Keypair, PublicKey, SystemProgram, Transaction } from "@solana/web3.js";

export const { AnchorProvider, Program, Wallet, BN } = anchor;
export { Keypair, PublicKey, SystemProgram, Transaction };

export const here = path.dirname(fileURLToPath(import.meta.url));
export const RPC = process.env.MARJAN_RPC ?? "https://api.devnet.solana.com";
export const idl = JSON.parse(fs.readFileSync(path.join(here, "idl/marjan.json"), "utf8"));
export const PROGRAM_ID = new PublicKey(idl.address);
export const BPF_UPGRADEABLE = new PublicKey("BPFLoaderUpgradeab1e11111111111111111111111");

const keyDir = path.join(here, ".keys");
fs.mkdirSync(keyDir, { recursive: true });
/** Loads (or creates) a named demo keypair kept out of git in client/.keys. */
export function key(name) {
  const f = path.join(keyDir, `${name}.json`);
  if (!fs.existsSync(f)) fs.writeFileSync(f, JSON.stringify(Array.from(Keypair.generate().secretKey)));
  return Keypair.fromSecretKey(Uint8Array.from(JSON.parse(fs.readFileSync(f, "utf8"))));
}

export const sponsor = Keypair.fromSecretKey(
  Uint8Array.from(JSON.parse(fs.readFileSync(path.join(os.homedir(), ".config/solana/devnet-deployer.json"), "utf8"))),
);
export const connection = new Connection(RPC, "confirmed");
export const program = new Program(idl, new AnchorProvider(connection, new Wallet(sponsor), { commitment: "confirmed" }));

export const sha = (s) => Array.from(crypto.createHash("sha256").update(s).digest());
const pda = (...seeds) => PublicKey.findProgramAddressSync(seeds, PROGRAM_ID)[0];
const u16 = (n) => { const b = Buffer.alloc(2); b.writeUInt16LE(n); return b; };
const u32 = (n) => { const b = Buffer.alloc(4); b.writeUInt32LE(n); return b; };
const u64 = (n) => { const b = Buffer.alloc(8); b.writeBigUInt64LE(BigInt(n)); return b; };
const enc = (s) => Buffer.from(s);

export const registryPda = pda(enc("registry"));
export const programDataPda = PublicKey.findProgramAddressSync([PROGRAM_ID.toBuffer()], BPF_UPGRADEABLE)[0];
export const agencyPda = (auth) => pda(enc("agency"), auth.toBuffer());
export const ataPda = (mgr, id) => pda(enc("ata"), mgr.toBuffer(), Buffer.from(id));
export const itemPda = (ata, n) => pda(enc("item"), ata.toBuffer(), u16(n));
export const usagePda = (item, ag) => pda(enc("usage"), item.toBuffer(), ag.toBuffer());
export const requestPda = (item, ag, id) => pda(enc("request"), item.toBuffer(), ag.toBuffer(), u64(id));
export const obligationPda = (debtor, id) => pda(enc("obligation"), debtor.toBuffer(), Buffer.from(id));
export const fiscalDocPda = (h) => pda(enc("fiscal_doc"), Buffer.from(h));
export const financingPda = (ob, n) => pda(enc("financing"), ob.toBuffer(), u32(n));

const errorsByCode = Object.fromEntries((idl.errors ?? []).map((e) => [e.code, e]));
export const explorer = (sig) => `https://explorer.solana.com/tx/${sig}?cluster=devnet`;
export const sleep = (ms) => new Promise((r) => setTimeout(r, ms));

/** Decodes a transaction error into the program's error name. */
export function outcomeOf(err) {
  if (!err) return "ok";
  const custom = err?.InstructionError?.[1]?.Custom;
  if (custom === 0) return "AccountAlreadyInUse"; // system program: the PDA was created by the winner
  return custom !== undefined ? errorsByCode[custom]?.name ?? `Custom(${custom})` : JSON.stringify(err);
}

/** Builds and signs a transaction (sponsor pays) without sending it. */
export async function signed(builder, signers, blockhash) {
  const tx = new Transaction().add(await builder.instruction());
  tx.feePayer = sponsor.publicKey;
  tx.recentBlockhash = blockhash;
  tx.sign(sponsor, ...signers);
  return tx;
}

export async function fetchTx(sig) {
  for (let i = 0; i < 20; i++) {
    const info = await connection.getTransaction(sig, { commitment: "confirmed", maxSupportedTransactionVersion: 0 }).catch(() => null);
    if (info) return info;
    await sleep(1000);
  }
  throw new Error(`transaction ${sig} not found`);
}

/** A run log: every step is a real transaction with its expected and observed outcome. */
export const IS_LOCAL = /127\.0\.0\.1|localhost/.test(RPC);
export function recorder(file, meta = {}) {
  // Local-validator runs never overwrite the devnet evidence files.
  if (IS_LOCAL) file = file.replace(/\.json$/, ".localnet.json");
  const run = { rpc: RPC, program: PROGRAM_ID.toBase58(), started_at: new Date().toISOString(), ...meta, steps: [] };
  const save = () => fs.writeFileSync(path.join(here, file), JSON.stringify(run, null, 2));
  async function step(label, basis, expect, builder, signers = []) {
    const { blockhash, lastValidBlockHeight } = await connection.getLatestBlockhash("confirmed");
    const tx = await signed(builder, signers, blockhash);
    const sig = await connection.sendRawTransaction(tx.serialize(), { skipPreflight: expect !== "ok" });
    await connection.confirmTransaction({ signature: sig, blockhash, lastValidBlockHeight }, "confirmed");
    const info = await fetchTx(sig);
    const outcome = outcomeOf(info?.meta?.err ?? null);
    const pass = expect === outcome;
    run.steps.push({ label, basis, expected: expect, outcome, pass, signature: sig, slot: info.slot, explorer: explorer(sig) });
    save();
    console.log(`${pass ? "✔" : "✘"} ${label} — ${outcome}\n    ${explorer(sig)}`);
    if (!pass) throw new Error(`Unexpected outcome for "${label}": expected ${expect}, got ${outcome}`);
    return sig;
  }
  return { run, save, step };
}
