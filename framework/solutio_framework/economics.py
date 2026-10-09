"""Unit economics and scalability from measured devnet costs.

On-chain costs come from client/devnet-costs.json: fees and rent deposits read
back from the 32 real devnet transactions of docs/DEVNET.md. Devnet charges the
same base fee per signature as mainnet (5,000 lamports); priority fees, which
vary with congestion, are excluded and stated as such.
"""

from __future__ import annotations

import json
from dataclasses import dataclass
from pathlib import Path

LAMPORTS_PER_SOL = 1_000_000_000
HERE = Path(__file__).resolve().parent
REPO = HERE.parent.parent


def load_costs(path: Path | None = None) -> dict:
    return json.loads((path or REPO / "client" / "devnet-costs.json").read_text())


def load_assumptions(path: Path | None = None) -> dict:
    return json.loads((path or HERE / "assumptions.json").read_text())


def _sum(steps: list[dict], prefixes: tuple[str, ...], key: str, only_ok: bool = True) -> int:
    picked = [s for s in steps if s["label"].startswith(prefixes) and (s["outcome"] == "ok" or not only_ok)]
    seen, total = set(), 0
    for s in picked:  # one representative of each step type
        kind = next(p for p in prefixes if s["label"].startswith(p))
        if kind in seen:
            continue
        seen.add(kind)
        total += s[key]
    return total


@dataclass(frozen=True)
class LifecycleCost:
    name: str
    transactions: int
    fee_lamports: int
    rent_lamports: int
    compute_units: int

    def sol(self, include_rent: bool = True) -> float:
        return (self.fee_lamports + (self.rent_lamports if include_rent else 0)) / LAMPORTS_PER_SOL


def lifecycle_costs(costs: dict) -> dict[str, LifecycleCost]:
    steps = costs["steps"]
    adhesion = ("Prefeitura A requests 50", "Supplier accepts", "Managing agency authorizes", "Prefeitura A executes")
    obligation = ("Prefeitura A records a verified", "Attach fiscal document", "Designated verifier", "Financier A advances",
                  "Prefeitura A records payment")
    refused = [s for s in steps if s["outcome"] != "ok"]
    return {
        "adhesion": LifecycleCost("Adhesion: request → supplier accepts → authorize → execute", len(adhesion),
                                  _sum(steps, adhesion, "fee_lamports"), _sum(steps, adhesion, "rent_lamports"),
                                  _sum(steps, adhesion, "compute_units")),
        "obligation": LifecycleCost("Obligation: verify → document → eligible → finance → pay", len(obligation),
                                    _sum(steps, obligation, "fee_lamports"), _sum(steps, obligation, "rent_lamports"),
                                    _sum(steps, obligation, "compute_units")),
        "refusal": LifecycleCost("Refused transaction (recorded on-chain)", 1,
                                 round(sum(s["fee_lamports"] for s in refused) / len(refused)), 0,
                                 round(sum(s["compute_units"] for s in refused) / len(refused))),
    }


def scenarios(assumptions: dict | None = None, costs: dict | None = None, usd_per_sol: float | None = None,
              close_accounts: bool | None = None) -> list[dict]:
    a = assumptions or load_assumptions()
    lc = lifecycle_costs(costs or load_costs())
    brl_usd = a["fx"]["brl_per_usd"]["value"]
    sol_usd = usd_per_sol if usd_per_sol is not None else a["fx"]["usd_per_sol"]["value"]
    sol_brl = sol_usd * brl_usd
    p, o = a["pricing"], a["operations"]
    closing = o["close_accounts_after_settlement"]["value"] if close_accounts is None else close_accounts
    rent_counts = not closing
    per_obligation_sol = lc["adhesion"].sol(rent_counts) * o["average_adhesion_obligations"]["value"] + lc["obligation"].sol(rent_counts)
    out = []
    for sc in a["scenarios"]:
        if sc["market"] == "BR":
            market_brl = a["anchors"]["br_srp_contracting_brl_per_year"]["value"]
        else:
            market_brl = a["anchors"]["global_procurement_usd_per_year"]["value"] * brl_usd
        volume = market_brl * sc["share_of_volume"]
        obligations = volume / o["average_obligation_brl"]["value"]
        financed_volume = volume * sc["share_financed"]
        financed_count = obligations * sc["share_financed"]
        revenue_take = financed_volume * p["take_rate_on_financed_volume"]["value"]
        revenue_checks = financed_count * p["checks_per_financed_obligation"]["value"] * p["verification_fee_brl_per_check"]["value"]
        revenue = revenue_take + revenue_checks
        chain_cost = obligations * per_obligation_sol * sol_brl
        out.append({
            "scenario": sc["name"],
            "volume_brl": volume,
            "obligations_per_year": obligations,
            "tx_per_second_avg": obligations * (lc["adhesion"].transactions + lc["obligation"].transactions) / (365 * 86_400),
            "financed_volume_brl": financed_volume,
            "revenue_brl": revenue,
            "revenue_take_brl": revenue_take,
            "revenue_checks_brl": revenue_checks,
            "chain_cost_brl": chain_cost,
            "chain_cost_share_of_revenue": chain_cost / revenue if revenue else float("inf"),
            "usd_per_sol": sol_usd,
            "rent_counted_as_cost": rent_counts,
        })
    return out


def cost_per_obligation_brl(usd_per_sol: float, assumptions: dict | None = None, costs: dict | None = None,
                            include_rent: bool = True) -> float:
    a = assumptions or load_assumptions()
    lc = lifecycle_costs(costs or load_costs())
    rent_counts = include_rent
    sol = lc["adhesion"].sol(rent_counts) + lc["obligation"].sol(rent_counts)
    return sol * usd_per_sol * a["fx"]["brl_per_usd"]["value"]
