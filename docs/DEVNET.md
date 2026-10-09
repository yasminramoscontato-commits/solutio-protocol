# Devnet run

The Solutio program is deployed on **Solana devnet** and was exercised end to end by [`client/devnet-demo.mjs`](../client/devnet-demo.mjs). Every row below is a real devnet transaction you can open in the explorer.

- Program: [`5cmBDMRdqAfrhmkMmLVxTBCNBHxh5sneyvWXJViyjz9E`](https://explorer.solana.com/address/5cmBDMRdqAfrhmkMmLVxTBCNBHxh5sneyvWXJViyjz9E?cluster=devnet)
- Deployed binary SHA-256: `4a9ee4815508b38d5e9bdd157edaad017e814abf230db644c2873e64e133581d` (same bytes as `target/deploy/solutio.so`; checked with `solana program dump`)
- Run: 2026-10-09T18:48:49.075Z → 2026-10-09T18:49:33.145Z, 32 transactions, all outcomes matched expectations.
- Registry: `2gsNHY73voZ1A2mHsrJ7VSRdfZo2uiuXKFrukEB6DMno` · Price record: `A1X8YBDXzXzjqmBCJU8PN59hYs4Xra6tfvkTPJ6ceywV` · Item: `2hSoBAXTypXeJGY7EKrVCGbCFXeahHfxixbkk3ZAHihd` · Obligation: `9cPTvUnoas4MjJFZMjEPc2bhGBhhwkkwBbmEkwTYG7f5`

**What is real:** the program, the transactions, the on-chain rejections and the balances.
**What is not:** every agency, supplier and financier is a DEMO identity with a fictional name, registered by a demo issuer. Amounts are illustrative. No real government data, pilot or partnership is involved.

Refused steps were sent with preflight disabled on purpose, so the program's rejection is itself recorded on-chain (open the transaction to see the error).

| # | Step | Legal / design basis | Outcome | Explorer |
| --- | --- | --- | --- | --- |
| 1 | Initialize registry | Demo issuer; designates the eligibility verifier | ✅ accepted | [tx](https://explorer.solana.com/tx/53yatCzPTywiw9KtcwNkZLgWR1xLVRB1y3kfXudTpoAUKWoPMwvZdkvEiNHT7vZEhsaxz7N5uptNnE2cCv9pHzSd?cluster=devnet) |
| 2 | Register agency: DEMO Central de Compras mv1bjudp | Demo issuer (registry authority) | ✅ accepted | [tx](https://explorer.solana.com/tx/3Lspcsapsopnqsbw8Aau3vkfN7ahn3FKgcFBq9W9YTeUdQTRoGhPmeE9ishhwAx2BUbQbDWv2xm3RQEYELzmYJUK?cluster=devnet) |
| 3 | Register agency: DEMO Prefeitura A mv1bjudp | Demo issuer (registry authority) | ✅ accepted | [tx](https://explorer.solana.com/tx/gCWJupEC5bxHnUCLgyLLZdCR1F3n7tUZVSCcJ3RvaQrM5BVmqi8wf3nzLLfgDq1rsPeTwcges55PcJaHuQED7Tx?cluster=devnet) |
| 4 | Register agency: DEMO Prefeitura B mv1bjudp | Demo issuer (registry authority) | ✅ accepted | [tx](https://explorer.solana.com/tx/4hmqGSL7aZXTLsWhszbVdVTZ3UyPz7L9dWSULRVhAnn8H1odik3xyYcUpHtNKTygcLWoHwzDmB4Gy9bGcJXb3LiV?cluster=devnet) |
| 5 | Register agency: DEMO Prefeitura C mv1bjudp | Demo issuer (registry authority) | ✅ accepted | [tx](https://explorer.solana.com/tx/FeHZij1gAsKjfPMwUPd8mfpsyqRAYNdh2xztzXfGcuhhJdVSSBizrCpcBz44ZK3T6aV9em2ykXGuLBdVgy1uGmS?cluster=devnet) |
| 6 | Register agency: DEMO Prefeitura D mv1bjudp | Demo issuer (registry authority) | ✅ accepted | [tx](https://explorer.solana.com/tx/4mQe18sGZFipbQU26wkFEEHZggTUmDr3aTUijQ2wmvh3SL1B4Egro9a7ptnmg7LCB3S93WtYsSp22YXmmsPcUjVV?cluster=devnet) |
| 7 | Register agency: DEMO Prefeitura E mv1bjudp | Demo issuer (registry authority) | ✅ accepted | [tx](https://explorer.solana.com/tx/5c1WYy8qwwd3B6Q4nJGXKJMzt2D4QaZ8eerVZ4CiVb1wCdqW8YEV81tjAjoWWLKwM9FoDE1Pyf7XNdMbUtJSvzBG?cluster=devnet) |
| 8 | Register agency: DEMO Ministerio Federal mv1bjudp | Demo issuer (registry authority) | ✅ accepted | [tx](https://explorer.solana.com/tx/gy1soro8izJcBrZHToF7wWtzoogt3qGxVWtEQm97KM5hZJFwY7UdcGtjbqB1njRbYCK6ikxGQQNHZzq59DMSHK8?cluster=devnet) |
| 9 | Register agency: DEMO Prefeitura do Interior mv1bjudp | Demo issuer (registry authority) | ✅ accepted | [tx](https://explorer.solana.com/tx/2wJf9NbgZJF7VFZUYzNUGTDrjpHNNZ3wxx7Vj2mH5dHgFenW3uHD3ptTLCWFSW4xLcBNzB62TdBRrKXGi2eNZ6vh?cluster=devnet) |
| 10 | Register agency: DEMO Secretaria Estadual AL mv1bjudp | Demo issuer (registry authority) | ✅ accepted | [tx](https://explorer.solana.com/tx/2KDfUon8vwAus4jjhtS2emi2BRz5rDYxGHGUNfnbsvnmKwX7kR6EfVvhfj7yZXCEEQTcRcXBNpiEDmfjSovpkDGn?cluster=devnet) |
| 11 | Create price record DEMO-ARP-mv1bjudp | Law 14.133 art. 82; Decree 11.462 art. 22 (validity) | ✅ accepted | [tx](https://explorer.solana.com/tx/48MMTpdcBQCyTvDbvXUHMvk4aQnYnKx1B5kyG3nySHmwjZY1TDs7m1TpBwMrf7qfyZVPYhxJ2JHyW3Fbw26aUKSv?cluster=devnet) |
| 12 | Add item 1: 100 registered, adhesion maximum 200 | Decree 11.462 art. 15 XI; art. 86 §5 | ✅ accepted | [tx](https://explorer.solana.com/tx/BHhmLgxFhVDKwVwBg2jb9xGuWjy3QeUXu26kb3PrGm7KoHGdHV6ZMw2TFZDFc2nNFNkBdTxubhDstFmrxxy8ku3?cluster=devnet) |
| 13 | Prefeitura A requests 50 units | Art. 86 §4: up to 50% per agency | ✅ accepted | [tx](https://explorer.solana.com/tx/63AusWmrcqVpeQm1PXpiqiNr9zmTMhii7WU2utiB1ShhTjjZK9aifvDmML3igCWDEJZiT16ddrgiivVcbWvaEazA?cluster=devnet) |
| 14 | Prefeitura A requests 1 units | Art. 86 §4: 51% would exceed the individual cap | ⛔ refused: `ExceedsIndividualCap` | [tx](https://explorer.solana.com/tx/3R3dziRTMvwdSnU5NcV9AUcVGG5X9NvoPPzQKAQe3PFJsz5fpJi3M5GN41phbRLX4tPyphrRPmtgYZRDAvnZ5PeH?cluster=devnet) |
| 15 | Prefeitura B requests 50 units | Art. 86 §5: running total 100 of 200 | ✅ accepted | [tx](https://explorer.solana.com/tx/3xX6oaTxyYQFkhnwCmnmjrbY6ke4YYKFpRpq6o4TFRax9Wx7MwmAjVMP9PvoCmvwBwcU1Xwizwcs9RnX9ArGRb44?cluster=devnet) |
| 16 | Prefeitura C requests 50 units | Art. 86 §5: running total 150 of 200 | ✅ accepted | [tx](https://explorer.solana.com/tx/U2nEjj9hST8DpDg4TRLrEoJjzqhdutk3BqBysZa8smwEm44X66DXRUKQEnXeHsHR5WbmTvBmmeVAqq1PiACEvqU?cluster=devnet) |
| 17 | Prefeitura D requests 50 units | Art. 86 §5: running total 200 of 200 | ✅ accepted | [tx](https://explorer.solana.com/tx/4tKuXBrrA7ZsjeihbmKinehES8JYiXSWnXUPcsYvkxXAHczgP9ycQSaLmGsyy5s17cVNgJ58Zbdjg45BQvAKtnYw?cluster=devnet) |
| 18 | Prefeitura E requests 1 units | Art. 86 §5: the item's adhesion pool is exhausted | ⛔ refused: `ExceedsGlobalCap` | [tx](https://explorer.solana.com/tx/iYRWRCAHm1s8hNRkSvQc2J7YTqthAWk55SmhyL3inkZRCjJXB5uiPGrneiAryLSJCEmSJc42zDBdEJ2aoYRn3bd?cluster=devnet) |
| 19 | Ministerio Federal requests 10 units | Art. 86 §8: federal agency to a state record | ⛔ refused: `FederalAdhesionForbidden` | [tx](https://explorer.solana.com/tx/2goH6bpJxFjQTzoSJxMUs4f2wugAspLQLNh9nyfEhCYUXC6nMYX6XqayZ4JUh448DbYP9DSryEAddmeDjjEcsncZ?cluster=devnet) |
| 20 | Create price record DEMO-ARP-MUN-mv1bjudp | Law 14.133 art. 82; Decree 11.462 art. 22 (validity) | ✅ accepted | [tx](https://explorer.solana.com/tx/5335hhJwKZ9D2gD7EuEVmPEX7qJiFZoFfqnKULtgNn1hbT2s2xGib13JMPFyqot2FQKmPL76YDgtryjZoUgVCW6w?cluster=devnet) |
| 21 | Add item 1: 100 registered, adhesion maximum 200 | Decree 11.462 art. 15 XI; art. 86 §5 | ✅ accepted | [tx](https://explorer.solana.com/tx/5JgnbGbsuMN2V71sb3qdaT9tHiDHidnanuSybQgYvMY1ULSfLWUzDxPXth1ZwnkWRuPfrnK47Mgtyj5qJsKtTDHs?cluster=devnet) |
| 22 | Secretaria Estadual (AL) requests 10 units | Alagoas Decree 95.019/2023 art. 33: state agency to a non-capital municipal record | ⛔ refused: `MunicipalAdhesionForbidden` | [tx](https://explorer.solana.com/tx/374qWD2DnjqSeZLiEoE1B3m2p3mVegcXA6DfFaRR8bnwcGS7HZ1eaXJb5hSaoddEwvWjt5eYoXde6V4vEeBGfoy1?cluster=devnet) |
| 23 | Supplier accepts Prefeitura A's adhesion by signature | Decree 11.462 art. 31 §1 (acceptance before authorization) | ✅ accepted | [tx](https://explorer.solana.com/tx/4T9vgucdtQe3bjXe9rvnfDQqvCP3HiSA3aAiJFuYAQr8WEDfUchM5H8SL2EU6G8EN7e2n6BhjExZxZV2YPic21PL?cluster=devnet) |
| 24 | Managing agency authorizes 50 units | Decree 11.462 art. 31 §1-2 (90-day execution window starts) | ✅ accepted | [tx](https://explorer.solana.com/tx/5qcJKT3BBGz8BmtBa1s8Bwy7DGe3rXtmYquEnEhDW5gDgTEXF823wjcM6HFVNUXn4nb1LEutXExWTbfZj5xTodfr?cluster=devnet) |
| 25 | Prefeitura A executes the purchase (nota de empenho) | Decree 11.462 art. 34 | ✅ accepted | [tx](https://explorer.solana.com/tx/55sspY9shTNnuUcG85y2G3RgdrC8o1NoXwrdAexBXPJUzzsnGHGuhNhRrig4Ayn5StXWBEnbWuUJspEApFSzTSUb?cluster=devnet) |
| 26 | Prefeitura A records a verified obligation of R$ 10.000,00 | Lei 4.320 art. 63 (liquidação); source = executed adhesion | ✅ accepted | [tx](https://explorer.solana.com/tx/x9KigLNK2C1uhSYzq7zuVNjKSGmKGMyRoGDsxYJ99mgKHr8nPCaruVkEgbxp5PyV6FkmJgyttSMu9xnLzxk559u?cluster=devnet) |
| 27 | Attach fiscal document (hash of the NF-e key) | One document backs one obligation (PDA by document hash) | ✅ accepted | [tx](https://explorer.solana.com/tx/5nZG5zC37UJdbAXGNyT5DHE15WhK4afhXYMd8kdxdmWjpYwfVaBL9D5unwHgZ7Kk6eZML9Ydv7D3AeHKpLLB3BGi?cluster=devnet) |
| 28 | Designated verifier confirms eligibility | Verified ≠ eligible: separate signed step | ✅ accepted | [tx](https://explorer.solana.com/tx/4DBejkBZXyitjDpCw4ZFM3HohquwzXuznWwGvRBGoXVUeAzpC8QbdyWYkMWF8Nv7WpCfq7Tcpi6JXHGuwY95gRRd?cluster=devnet) |
| 29 | Financier A advances R$ 6.000,00 (creditor co-signs) | CC art. 290 (notice of assignment); cumulative financing ≤ eligible | ✅ accepted | [tx](https://explorer.solana.com/tx/3gVKDcQ5wVTt9URN3S6xJe7NTHjbijgFL2e6csbNpF9om6K1Lc3jkyPGC73qG5kUzg4TgboMSKsvDzrrXkk6Vbga?cluster=devnet) |
| 30 | Financier B advances R$ 4.000,00 | CC art. 290 (notice of assignment); cumulative financing ≤ eligible | ✅ accepted | [tx](https://explorer.solana.com/tx/3oGwnzn1Udu4duxeHjGVPyo9uW5FTM3t9UE4d49gqxEUFCXNMn6QiQKUqhvn9csP53ruM5kMrqf5pB6JBz4fJ6mq?cluster=devnet) |
| 31 | Financier A tries to finance R$ 0,01 more | CC art. 290 (notice of assignment); cumulative financing ≤ eligible | ⛔ refused: `ExceedsFinanceableBalance` | [tx](https://explorer.solana.com/tx/2KoQGRQWVqLhTaVS16k4QHxxXkLrZ2UDGcKaPCPsQqrVHvg1DbmKV1HnycQ6RJu6FjU3b6v53Qc2YbmqKwmt4bxY?cluster=devnet) |
| 32 | Prefeitura A records payment of R$ 10.000,00 | Lei 4.320 arts. 64-65 (ordem bancária) | ✅ accepted | [tx](https://explorer.solana.com/tx/WsdmBccz5QVzMY2zgsH4hGSDq5aKeaaBnG1oocXseDJncv4189D4BoYkSx7799o99241YLLuGzgGDyx5PAvb1rm?cluster=devnet) |

The eight signer wallets (managing agency, five municipalities, supplier, financier) ended the run holding **0 SOL**: a sponsor paid every fee and rent deposit.

Reproduce: `cd client && npm install && node devnet-demo.mjs` (requires a funded devnet keypair at `~/.config/solana/devnet-deployer.json`; it becomes the demo issuer).
