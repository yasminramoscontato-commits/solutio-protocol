# Devnet runs

The Solutio program is deployed on **Solana devnet**. Every row below is a real devnet transaction you can open in the explorer.

- Program: [`5cmBDMRdqAfrhmkMmLVxTBCNBHxh5sneyvWXJViyjz9E`](https://explorer.solana.com/address/5cmBDMRdqAfrhmkMmLVxTBCNBHxh5sneyvWXJViyjz9E?cluster=devnet)
- Current binary: SHA-256 `99ce61559156d59b42b3fae711aaf589f5769766afab9857a12b6386e5837ebd` (the audited version, commit `38d8baa`; checked against `solana program dump`). Upgrade transaction: [`fyQ8KkWM1n9UcgWz…`](https://explorer.solana.com/tx/fyQ8KkWM1n9UcgWzCE4XSZkdienD4xNTyuz3L5hEXTSZ9c4BhMFU3uVLWCvzAAEee97njXE5wBn4Kv7BriywPcz?cluster=devnet).
- An earlier run of 32 transactions against the pre-audit binary is preserved in the git history (commit `722f683`, this file).

**What is real:** the program, the transactions, the on-chain refusals, the balances and the rent refunds.
**What is not:** every agency, supplier and financier is a DEMO identity with a fictional name, registered by a demo issuer. Amounts are illustrative. No real government data, pilot or partnership is involved.

Refused steps are sent with preflight disabled on purpose, so the program's refusal is itself recorded on-chain.

## 1. Scripted scenario (28 transactions, 2026-10-10T02:47 UTC)

Agencies and the registry were registered in the earlier run and are reused here.

| # | Step | Basis | Outcome | Explorer |
| --- | --- | --- | --- | --- |
| 1 | Create price record DEMO-ARP-mv1snte3 | Law 14.133 art. 82; Decree 11.462 art. 22 (validity) | ✅ accepted | [tx](https://explorer.solana.com/tx/3FAmf5PmgbywyYY5xH3gc3LQeoZtDT6PcJ5gN4DbZZHrVeZH6BnQZMrHWdXQHSXWdA2XtqNn38j4ZJMx9b7oQTf8?cluster=devnet) |
| 2 | Add item 1: 100 registered, adhesion maximum 200 | Decree 11.462 art. 15 XI; art. 86 §5 | ✅ accepted | [tx](https://explorer.solana.com/tx/4FcbNTSsTmBnHYjUcwY7ttsjZ15fQeRjg7fUvfu7ikdW7XjrX5saWnGAma8oaS7kaPa1pLntubhp27WsoSe7MRSZ?cluster=devnet) |
| 3 | Prefeitura A requests 50 units | Art. 86 §4: up to 50% per agency | ✅ accepted | [tx](https://explorer.solana.com/tx/35rcNxdLc8dAwLBN2ttNn9Z46t6yHXTnpfL7tLoPvy1Eu4sXvi7EwKtiuh5fZpNFKhoiK7du3a2LpuMEr2iTQSbu?cluster=devnet) |
| 4 | Prefeitura A requests 1 units | Art. 86 §4: 51% would exceed the individual cap | ⛔ refused: `ExceedsIndividualCap` | [tx](https://explorer.solana.com/tx/3LchMMKbMqMyyN5AvJ6yzP6QwvuVpwqU9WFaSnvFeBjER6eoZWfTcbeAgjbUBPRoX1wsESU7SxGTW1GhP6zctZCA?cluster=devnet) |
| 5 | Prefeitura B requests 50 units | Art. 86 §5: running total 100 of 200 | ✅ accepted | [tx](https://explorer.solana.com/tx/5FFhaZ7UJ94QNpscTuze4W15P4aqQuedvgHVdh6xDu39uUZNjh49vcoL3cK5HBToRZDsNHXsySiuGvLEnUp42ntp?cluster=devnet) |
| 6 | Prefeitura C requests 50 units | Art. 86 §5: running total 150 of 200 | ✅ accepted | [tx](https://explorer.solana.com/tx/2dQvUbScrdsvjtZcUPyJ5Kgz5BUXr7Qh8YgeriBPH6FaeMVqpGZeZfm64gbZbnijwAzoxrMd2AzbjvqRGxB3QBBe?cluster=devnet) |
| 7 | Prefeitura D requests 50 units | Art. 86 §5: running total 200 of 200 | ✅ accepted | [tx](https://explorer.solana.com/tx/2CPjxB6CD1CBcTfxErtdNhUWTCpM9gwgz7UYyS6tGqb59HDJ91LMBT6DYhbczVgqB3wcWKvbXRiKaqMTeP8tot1a?cluster=devnet) |
| 8 | Prefeitura E requests 1 units | Art. 86 §5: the item's adhesion pool is exhausted | ⛔ refused: `ExceedsGlobalCap` | [tx](https://explorer.solana.com/tx/4kTADgHvxQ8MUQ3AL5pVusYvwTvuLzti3QRhEUkQP4X4rDb5YUPvthtehNiEfve1YV3MUKgNtd6J9xeU7bgDQVSt?cluster=devnet) |
| 9 | Ministerio Federal requests 10 units | Art. 86 §8: federal agency to a state record | ⛔ refused: `FederalAdhesionForbidden` | [tx](https://explorer.solana.com/tx/2ctcfHGy8YxsN7UwQ3J8hg14MWwt3xsuAKUSedabTd7JRTurnqfJtbVEqarpC9D91KbJWKum54bq5riFAMPZaZKC?cluster=devnet) |
| 10 | Create price record DEMO-ARP-MUN-mv1snte3 | Law 14.133 art. 82; Decree 11.462 art. 22 (validity) | ✅ accepted | [tx](https://explorer.solana.com/tx/5KP8K5nQE1KEGZbW8EKy5wr6sbJsF8z1M5BRvPJatCGnzQurVzT4FQzWdpFGpEMBARFuFBxRFS29tSG6gRx96hQ4?cluster=devnet) |
| 11 | Add item 1: 100 registered, adhesion maximum 200 | Decree 11.462 art. 15 XI; art. 86 §5 | ✅ accepted | [tx](https://explorer.solana.com/tx/sAzsksqbNgGgbQD6J4y6bTdLcwaZhYeAa537xvZ61fymAvj84vdJA2QmBcdrNTSkcnnrBHifNipeYE6f9LuUTTe?cluster=devnet) |
| 12 | Secretaria Estadual (AL) requests 10 units | Alagoas Decree 95.019/2023 art. 33: state agency to a non-capital municipal record | ⛔ refused: `MunicipalAdhesionForbidden` | [tx](https://explorer.solana.com/tx/3XhS3CyXqbbW8CmFimm5oyrZq3XNugWhAKD6MdxSjchYzfULowJQPbmxxJAWHUd6s2JxXN9VuRJpEp8JMq4kWgoF?cluster=devnet) |
| 13 | Supplier declines Prefeitura D's adhesion: its 50 units return to the pool | Decree 11.462 art. 31 III (supplier acceptance) | ✅ accepted | [tx](https://explorer.solana.com/tx/3iUD4vH74huoQrSZ44auWzKxxG663iVZhKDsjXEr6NFuXWmSYAcsow9numdrJZa5ot8kgxZj3WYWAPf1LMXX1V5C?cluster=devnet) |
| 14 | Anyone closes the rejected request; rent returns to the sponsor | Rent recovery (no state of value is lost) | ✅ accepted | [tx](https://explorer.solana.com/tx/2YwoANJsSSw6dto51dxzJ7TJTTdDJFqJCgCky6rLCFRU7xa2A8pc2ayPCd7kdgRiar2tccQ8NonERQycarAPNX4Y?cluster=devnet) |
| 15 | Supplier accepts Prefeitura A's adhesion by signature | Decree 11.462 art. 31 §1 (acceptance before authorization) | ✅ accepted | [tx](https://explorer.solana.com/tx/2p8vW333WNAZfA9Zu8tzmiqNUpCXLNRRygLkAKyTyEwchGRrtLcZrbpDKWX7xbx7yS8jGLrrDzmMKa4btT1vGif2?cluster=devnet) |
| 16 | Managing agency authorizes 50 units | Decree 11.462 art. 31 §1-2 (90-day execution window starts) | ✅ accepted | [tx](https://explorer.solana.com/tx/5G2FmmtVLYcRxpeygP8jpposv8tSEDmZKdmyXakbYYkLiRmi1DahkYK883EqhWJXNprfGpE2djBu1dNphi2mRstG?cluster=devnet) |
| 17 | Prefeitura A executes the purchase (nota de empenho) | Decree 11.462 art. 34 | ✅ accepted | [tx](https://explorer.solana.com/tx/4oxa9EeLaxJHYKocfX4P8SrdFN8jKktUM9posJ1vEzHyjunnGmW9PbEsTGuXgCfbiVf9gzFs4hSk2LqkgCNquXiC?cluster=devnet) |
| 18 | Prefeitura A records a verified obligation of R$ 10.000,00 | Lei 4.320 art. 63 (liquidação); source = executed adhesion | ✅ accepted | [tx](https://explorer.solana.com/tx/2HfKjGDYvGL98KDv3GMYvvMDrvLQNZUBuSfMawYcGDLYME563pnm4huFXVRLhinUEbUkNYhWLEB85o3hs1hU3a2d?cluster=devnet) |
| 19 | A second obligation citing the same adhesion, R$ 0,01 beyond its value | Obligations ≤ authorized quantity × unit price (50 × R$ 200,00) | ⛔ refused: `ExceedsAdhesionValue` | [tx](https://explorer.solana.com/tx/4oEz9vhSoobTVF5N8KCywvtH1Qke7HWr5jV1pCHKmJEAbxzCiHtcv1LtGJKrdvLEqNkaVp5typAhnDppotsaon2u?cluster=devnet) |
| 20 | Attach fiscal document (hash of the NF-e key) | One document backs one obligation (PDA by document hash) | ✅ accepted | [tx](https://explorer.solana.com/tx/5MuyqX1cYyPQFQUPgUq3e8vY4vKfN7GZhn582NYnTiMC7pZ9jxbdy12LV59KVGfGHZaNBLehEWfUarJ17jpXzixZ?cluster=devnet) |
| 21 | Designated verifier confirms eligibility | Verified ≠ eligible: separate signed step | ✅ accepted | [tx](https://explorer.solana.com/tx/5mpZV8mXpyZkw2vbXFFrv7aGaWEDEsfoMM1cvMw6F3NjQ32brtXf73X8ipcsCWUzQtdUUZ32BeMBG5nMh2LxD2sR?cluster=devnet) |
| 22 | Financier A advances R$ 6.000,00 (creditor co-signs) | CC art. 290 (notice of assignment); cumulative financing ≤ eligible | ✅ accepted | [tx](https://explorer.solana.com/tx/4M9x298atUGwD5cQKuHdfUBqPr2sUrekGjK8ju9dffajgdn7vTtm5ZqBW43LBLBK18yAkWT6jMvMs5wF5dW3E4ds?cluster=devnet) |
| 23 | Financier B advances R$ 4.000,00 | CC art. 290 (notice of assignment); cumulative financing ≤ eligible | ✅ accepted | [tx](https://explorer.solana.com/tx/45Goom6TZM89c5BtAGWeLmvcmoHd19435nbSaYTJaL4fB6pBhJ7PiQkjmVu9oTAVY7EdypWx5BrTbLonkvPcVTUj?cluster=devnet) |
| 24 | Financier A tries to finance R$ 0,01 more | CC art. 290 (notice of assignment); cumulative financing ≤ eligible | ⛔ refused: `ExceedsFinanceableBalance` | [tx](https://explorer.solana.com/tx/PAp46RjYV7gWPzD8VjxkMVyFQ8qZXfVuk2UN6CLBDkPYoKFME5EbJCCwgzzcYTheCXnNEUgVD3HYbzxdiBnaSPQ?cluster=devnet) |
| 25 | Prefeitura A records payment of R$ 10.000,00 | Lei 4.320 arts. 64-65 (ordem bancária) | ✅ accepted | [tx](https://explorer.solana.com/tx/7eWVipEioqjmcpM8wnKREH3rUHVtzGfHsqo8WBEQztdCpVNXQ24wed94j3MQhfkNiktA8NieYsN59hDpPzP2UTo?cluster=devnet) |
| 26 | Anyone closes financing #0 of the settled obligation; rent returns to the sponsor | Rent recovery after settlement | ✅ accepted | [tx](https://explorer.solana.com/tx/4Yztmx112dRU5qP1SM1vzhqKodmnh3PaYxfCXSohAYNi4by3aEQrwJ4FT275eqmU12syMBdSYC3suYeaqeBxaQ4A?cluster=devnet) |
| 27 | Anyone closes financing #1 of the settled obligation; rent returns to the sponsor | Rent recovery after settlement | ✅ accepted | [tx](https://explorer.solana.com/tx/4zZNj2rESmsvQCKSWPcUEDGkyCj1cE4TYLCJ2MfV7tHNRmgARTexGe82ignEws1ZJrZNLNQHLvgav9r8GRpdNuFk?cluster=devnet) |
| 28 | Anyone closes the settled obligation; its fiscal document stays as the anti-reuse marker | Rent recovery after settlement | ✅ accepted | [tx](https://explorer.solana.com/tx/55cG1J5e2bRempPXsTByn2P7YvHvbQxwC8YJLuYUGeu5KLFok9SQDbMAADTcgWBiwqou6dNm8jA8YacZ5s6nF6fL?cluster=devnet) |

The 8 signer wallets (managing agency, municipalities, supplier, financier) hold **0 SOL**: a sponsor paid every fee and rent deposit, and received the rent back when accounts were closed.

## 2. Concurrency test (transactions submitted in parallel)

Each race signs all transactions with the same blockhash and submits them at once, without waiting. The validator serializes transactions that write the same account; each one re-checks the rules against the state the previous one left.

### Six agencies request 50 units each at once (pool of 200)

4 of 6 succeeded (expected 4), landing in slot(s) 509395094, 509395095.

| Outcome | Slot | Explorer |
| --- | --- | --- |
| ✅ accepted | 509395094 | [tx](https://explorer.solana.com/tx/2h5VQNWitaTTKnPQ9XW72aLijQ91gDShpPH9E2fyGNX6yjpvMAK2fSojALPh5g1LHojFQfadSc1thrsst53tEhBo?cluster=devnet) |
| ✅ accepted | 509395094 | [tx](https://explorer.solana.com/tx/J4sCQpqp9CopUJf1uvTJEQ4EfBSHcb8D6rU3ZrUNLvSApAovHHAXzJggcREjTBxCpy5pqB8DyTgMMwn51DCvn1y?cluster=devnet) |
| ⛔ refused: `ExceedsGlobalCap` | 509395095 | [tx](https://explorer.solana.com/tx/4zqR31xLt2NBCa3jwRGKQr1YZAAZh8dTjcg4JyRD1Hj4NwtSdrfhR4pz9MNT7Sd5NFGMM8MpjyZmuLzy49L9oq21?cluster=devnet) |
| ✅ accepted | 509395095 | [tx](https://explorer.solana.com/tx/2fMeTa2K8zbSmf3kpnsnHVaeSuenfNhNvzHru3thy2oDuopodWgbgjqv2d5XTPfJ23yXpcTwUN65V9tHNZgCVh36?cluster=devnet) |
| ✅ accepted | 509395094 | [tx](https://explorer.solana.com/tx/2L3RiGy4FAPcvjVhmttX9MCmE3J9p9VoNWM6GDTNBN519bMgdsVZqe3CfuPKyG36QHuTBJQM9TnVbTEx4PbkVb5E?cluster=devnet) |
| ⛔ refused: `ExceedsGlobalCap` | 509395095 | [tx](https://explorer.solana.com/tx/5m6hKiGbLcK54YMftm62kR38gbtE2JS9h6rArSybvqjYiKMcwo39JggP41VwXbAxfLi4D8FKAApRyi3oTTtiKhMe?cluster=devnet) |

### Two financiers race for the last R$ 1.000,00 at once

1 of 2 succeeded (expected 1), landing in slot(s) 509395186.

| Outcome | Slot | Explorer |
| --- | --- | --- |
| ✅ accepted | 509395186 | [tx](https://explorer.solana.com/tx/gFqtQ5kLzF7rnoZocZ19Vhj5trRowK4P58jipQWPcj6pkHt4rKekBvT5EKWHWbPwcywmWMwuKd9s9muUycQpXxz?cluster=devnet) |
| ⛔ refused: `AccountAlreadyInUse` | 509395186 | [tx](https://explorer.solana.com/tx/2n7RYM8eLPq4zfFScsMfU3Anin4SXEzfPMbgxtsQP66iJgBup33MgCeKMZBtw2unjTMPhw5og2c3BnrK5TDfBxZc?cluster=devnet) |

State read back after the races: item committed **200 of 200**; obligation financed **1000000 of 1,000,000** cents.

In the financing race the losing transaction fails with `AccountAlreadyInUse`: the winner already created the financing record with that sequence number, so the second financing of the same balance cannot exist.

**Scope of this evidence:** one devnet run. It shows serialization of conflicting writes within and across consecutive slots; it does not measure behaviour under mainnet congestion.

## Reproduce

```bash
cd client && npm install
node devnet-demo.mjs      # scripted scenario
node concurrency.mjs      # parallel races
node measure-costs.mjs    # fees, rent and refunds -> devnet-costs.json
python3 render_devnet.py  # this page
```

Requires a funded devnet keypair at `~/.config/solana/devnet-deployer.json` (the program's upgrade authority, which is also the registry authority). Set `SOLUTIO_RPC=http://127.0.0.1:8899` to run against a local validator; local runs write `*.localnet.json` and never overwrite the devnet records.
