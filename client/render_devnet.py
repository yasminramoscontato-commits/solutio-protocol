"""Renders docs/DEVNET.md from the recorded devnet runs.

    python3 client/render_devnet.py
"""

import json
from pathlib import Path

HERE = Path(__file__).resolve().parent
OUT = HERE.parent / "docs" / "DEVNET.md"
PROGRAM_SHA256 = "99ce61559156d59b42b3fae711aaf589f5769766afab9857a12b6386e5837ebd"
UPGRADE_SIG = "fyQ8KkWM1n9UcgWzCE4XSZkdienD4xNTyuz3L5hEXTSZ9c4BhMFU3uVLWCvzAAEee97njXE5wBn4Kv7BriywPcz"


def outcome(s: str) -> str:
    return "✅ accepted" if s == "ok" else f"⛔ refused: `{s}`"


def main() -> None:
    demo = json.loads((HERE / "devnet-run.json").read_text())
    conc = json.loads((HERE / "concurrency-run.json").read_text())
    prog = demo["program"]
    lines = [
        "# Devnet runs",
        "",
        "The Marjan program is deployed on **Solana devnet**. Every row below is a real devnet transaction you can open in the explorer.",
        "",
        f"- Program: [`{prog}`](https://explorer.solana.com/address/{prog}?cluster=devnet)",
        f"- Current binary: SHA-256 `{PROGRAM_SHA256}` (the audited version, commit `38d8baa`; checked against `solana program dump`). It was built under the project's former name, Solutio; the rename changes no instruction, account or seed, so a binary rebuilt under the new name behaves the same but has a different hash. "
        f"Upgrade transaction: [`{UPGRADE_SIG[:16]}…`](https://explorer.solana.com/tx/{UPGRADE_SIG}?cluster=devnet).",
        "- An earlier run of 32 transactions against the pre-audit binary is preserved in the git history (commit `722f683`, this file).",
        "",
        "**What is real:** the program, the transactions, the on-chain refusals, the balances and the rent refunds.",
        "**What is not:** every agency, supplier and financier is a DEMO identity with a fictional name, registered by a demo issuer. "
        "Amounts are illustrative. No real government data, pilot or partnership is involved.",
        "",
        "Refused steps are sent with preflight disabled on purpose, so the program's refusal is itself recorded on-chain.",
        "",
        f"## 1. Scripted scenario ({len(demo['steps'])} transactions, {demo['started_at'][:16]} UTC)",
        "",
        "Agencies and the registry were registered in the earlier run and are reused here.",
        "",
        "| # | Step | Basis | Outcome | Explorer |",
        "| --- | --- | --- | --- | --- |",
    ]
    for i, s in enumerate(demo["steps"], 1):
        lines.append(f"| {i} | {s['label']} | {s['basis']} | {outcome(s['outcome'])} | [tx]({s['explorer']}) |")
    lines += [
        "",
        f"The {len(demo.get('agency_wallet_lamports', []))} signer wallets (managing agency, municipalities, supplier, financier) hold "
        "**0 SOL**: a sponsor paid every fee and rent deposit, and received the rent back when accounts were closed.",
        "",
        "## 2. Concurrency test (transactions submitted in parallel)",
        "",
        "Each race signs all transactions with the same blockhash and submits them at once, without waiting. The validator "
        "serializes transactions that write the same account; each one re-checks the rules against the state the previous one left.",
        "",
    ]
    for r in conc.get("races", []):
        lines += [
            f"### {r['label']}",
            "",
            f"{r['succeeded']} of {r['submitted']} succeeded (expected {r['expected_succeeded']}), landing in slot(s) "
            f"{', '.join(str(x) for x in r['slots'])}.",
            "",
            "| Outcome | Slot | Explorer |",
            "| --- | --- | --- |",
        ]
        lines += [f"| {outcome(x['outcome'])} | {x['slot']} | [tx]({x['explorer']}) |" for x in r["results"]]
        lines.append("")
    lines += [
        f"State read back after the races: item committed **{conc.get('item_committed_after_race')} of 200**; obligation financed "
        f"**{conc.get('obligation_financed_after_race')} of 1,000,000** cents.",
        "",
        "In the financing race the losing transaction fails with `AccountAlreadyInUse`: the winner already created the financing "
        "record with that sequence number, so the second financing of the same balance cannot exist.",
        "",
        "**Scope of this evidence:** one devnet run. It shows serialization of conflicting writes within and across consecutive "
        "slots; it does not measure behaviour under mainnet congestion.",
        "",
        "## Reproduce",
        "",
        "```bash",
        "cd client && npm install",
        "node devnet-demo.mjs      # scripted scenario",
        "node concurrency.mjs      # parallel races",
        "node measure-costs.mjs    # fees, rent and refunds -> devnet-costs.json",
        "python3 render_devnet.py  # this page",
        "```",
        "",
        "Requires a funded devnet keypair at `~/.config/solana/devnet-deployer.json` (the program's upgrade authority, which is also "
        "the registry authority). Set `MARJAN_RPC=http://127.0.0.1:8899` to run against a local validator; local runs write "
        "`*.localnet.json` and never overwrite the devnet records.",
        "",
    ]
    OUT.write_text("\n".join(lines))
    print(f"wrote {OUT.relative_to(HERE.parent)}")


if __name__ == "__main__":
    main()
