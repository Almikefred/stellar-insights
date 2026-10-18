# PayRaider

> ⚠️ **DEPRECATED — Development has moved to the [Pay-Raider organisation](https://github.com/Pay-Raider).**
>
> This repository is **archived and no longer maintained**. No new features, bug fixes, or security patches will be merged here. All active development continues in the Pay-Raider org:
>
> | Component | New home |
> | --- | --- |
> | Backend API (Rust) | [Pay-Raider/payraider-backend](https://github.com/Pay-Raider/payraider-backend) |
> | Web app (Next.js) | [Pay-Raider/payraider-app](https://github.com/Pay-Raider/payraider-app) |
> | Plugin & SDKs | [Pay-Raider/payraider-plugin](https://github.com/Pay-Raider/payraider-plugin) |
> | Soroban contracts | [Pay-Raider/payraider-contracts](https://github.com/Pay-Raider/payraider-contracts) |
> | Mobile app | [Pay-Raider/payraider-mobile](https://github.com/Pay-Raider/payraider-mobile) |
>
> **If you have open issues or PRs here**, please re-open them in the relevant Pay-Raider repository. See the [migration checklist](#migration-checklist--archive-plan) below.

## Migration Checklist & Archive Plan

### What has migrated

- [x] Core preflight API (`/api/v1/preflight`) → `payraider-backend`
- [x] TypeScript and Python SDKs → `payraider-plugin`
- [x] MCP server → `payraider-plugin`
- [x] Claude plugin → `payraider-plugin`
- [x] Soroban snapshot contract → `payraider-contracts`
- [x] Next.js dashboard and corridor explorer → `payraider-app`
- [x] React Native mobile app → `payraider-mobile`

### What stays here (history only)

- Earlier-scope features: governance tooling, quest system, GDPR tooling — not carried forward.
- All git history, tags, and releases are preserved in this repo for reference.

### Archive plan

1. All open issues labelled `migrate` will be recreated in the appropriate Pay-Raider repo.
2. This repo will be set to **read-only / archived** on GitHub once the issue backlog is triaged (target: next sprint after this PR merges).
3. A redirect banner (this notice) will remain in the README permanently.
4. The `main` branch will receive no further commits after archiving.

[![Coverage](https://codecov.io/gh/Ndifreke000/stellar-insights/branch/main/graph/badge.svg)](https://codecov.io/gh/Ndifreke000/stellar-insights)
[![Coverage CI](https://github.com/Ndifreke000/stellar-insights/actions/workflows/coverage.yml/badge.svg)](https://github.com/Ndifreke000/stellar-insights/actions/workflows/coverage.yml)

**Check a Stellar payment corridor before you pay out.**

Off-ramps and payout apps lose money when a payment goes out on a corridor
that is degraded or too thin for the amount. PayRaider answers one question
before the money moves: *is this corridor healthy enough to pay on right
now?*

```bash
curl -s "https://<your-payraider-host>/api/v1/preflight?source_asset=USDC&destination_asset=NGN&amount_usd=2500"
```

returns `proceed`, `caution`, `hold` or `unknown`, the checks behind it
(success rate of recent payments including failed ones, liquidity against the
amount, sample size, corridor health) and healthier corridors to the same
currency.

- **Observed, not modelled.** Decisions come from recent payments on the
  Stellar ledger. No data means `unknown`, never a guess.
- **Free to try.** No API key needed; keys raise the rate limit.
- **Paid in USDC on Stellar.** The Pro plan is bought with a USDC payment that
  the API verifies on the ledger.
- **Fits any stack.** REST, TypeScript and Python SDKs, an MCP server for AI
  agents, and a Claude plugin.

Integration guide: [docs/PLUGIN.md](docs/PLUGIN.md).

### What is in this repository

| Path | What it is |
| --- | --- |
| `backend/` | Rust API (Axum, SQLite): corridor data, the pre-payment check, wallet sign-in, API keys and USDC billing |
| `sdk/typescript`, `sdk/python`, `sdk/react` | Client SDKs and React hooks — releases: [sdk/RELEASING.md](sdk/RELEASING.md), versions: [sdk/COMPATIBILITY.md](sdk/COMPATIBILITY.md) |
| `sdk/mcp-server`, `plugins/payraider` | MCP server and Claude plugin |
| `frontend/` | Next.js dashboard, corridor explorer and self-serve API keys |
| `contracts/` | The `payraider` Soroban contract that anchors snapshot hashes on-chain — how to verify a snapshot: [docs/SNAPSHOT_ANCHORING.md](docs/SNAPSHOT_ANCHORING.md) |
| `mobile/` | React Native app ([status](mobile/README.md#current-status)) |

The dashboard also carries features from the project's earlier scope
(governance, quests, GDPR tooling and others). They are kept but are not part
of the off-ramp product.

## 🔧 Quick start

No database server to install — SQLite runs in-process and the file is created
on first start.

1. Run backend

```bash
cd backend
cp .env.example .env
# DATABASE_URL already defaults to sqlite:./payraider.db
# replace the CHANGE_ME placeholders: the backend refuses to start with them
#   JWT_SECRET=$(openssl rand -base64 48)
#   ENCRYPTION_KEY=$(openssl rand -hex 32)
#   SEP10_SERVER_PUBLIC_KEY=<a real Stellar public key>
# set STELLAR_RPC_URL etc. as needed
cargo run
```

2. Run frontend

```bash
cd frontend
cp .env.example .env.local   # NEXT_PUBLIC_API_URL must point at the backend
pnpm install
pnpm dev
```

## Plugin: check a corridor before you pay

Off-ramp and payout apps can ask whether a corridor is healthy enough to pay
on right now. The check is public and read-only, and needs no API key:

```bash
curl -s -X POST http://localhost:8080/api/v1/preflight \
  -H 'content-type: application/json' \
  -d '{"source_asset":"USDC","destination_asset":"NGN","amount_usd":2500}'
```

It returns `proceed`, `caution`, `hold` or `unknown` with the checks behind the
decision and healthier alternative corridors. The same check is available
through the TypeScript and Python SDKs, as an MCP server you can host
(`sdk/mcp-server`), and as a Claude plugin (`plugins/payraider`). See
[docs/PLUGIN.md](docs/PLUGIN.md).

### Source amount or destination amount

The check accepts either the amount you send or the amount the recipient must
receive:

- `amount_usd` — the source amount you intend to send. The liquidity check
  runs against this amount.
- `destination_amount` — the amount the recipient must receive. The source
  amount is derived from it via the strict-receive path, and the liquidity
  check runs against that converted source amount.

Send exactly one of the two. Supplying both, or neither, is rejected at the
boundary with the standard error response.

```bash
curl -s -X POST http://localhost:8080/api/v1/preflight \
  -H 'content-type: application/json' \
  -d '{"source_asset":"USDC","destination_asset":"NGN","destination_amount":2500}'
```

### Batch checks

Payout apps that process bulk files can check many corridors and amounts in a
single request. `POST /api/v1/preflight/batch` accepts up to **100** items and
returns one result per item, in the same order. All items in a batch are
evaluated against a single corridor table snapshot, so the results are
mutually consistent even if the underlying data changes mid-request.

```bash
curl -s -X POST http://localhost:8080/api/v1/preflight/batch \
  -H 'content-type: application/json' \
  -d '{"items":[
    {"source_asset":"USDC","destination_asset":"NGN","amount_usd":2500},
    {"source_asset":"USDC","destination_asset":"KES","amount_usd":800}
  ]}'
```

Each item follows the same rules as the single check (exactly one of
`amount_usd` / `destination_amount`, `CODE:ISSUER` asset pinning). A batch with
more than 100 items, or an empty `items` array, is rejected at the boundary
with the standard error response.

### Corridors are keyed by asset code **and** issuer

A corridor is identified by the full asset, not just its code. `USDC` issued
by Circle and `USDC` issued by another anchor are **different corridors** with
different liquidity and risk, so they are tracked and evaluated separately.

Pass the issuer with `CODE:ISSUER` to pin a specific asset:

```bash
curl -s -X POST http://localhost:8080/api/v1/preflight \
  -H 'content-type: application/json' \
  -d '{"source_asset":"USD

/* … truncated 1003 chars — edit only what you need near the top … */
