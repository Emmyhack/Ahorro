# Ahorro

DeFi-native thrift and savings platform on Solana. MVP includes rotational thrift groups with an insurance pool funded from each contribution. USDC-based operations.

## Stack
- Anchor (Solana programs)
- Rust (on-chain)
- React + Vite + TypeScript (frontend)
- Anchor/TypeScript client

## Prerequisites
- Node 18+ (Node 22 OK)
- Rust toolchain (installed via rustup)
- Anchor CLI (optional for localnet)
- Solana CLI (optional for devnet interactions)

## Programs (Anchor)
- Program name: `ahorro`
- Program id: `Ez7nS3RhjdeYknDMJSrunJE1wbACMg7yN4YTgFmkHkQz` (devnet placeholder)

Build:

```
cargo build
```

Tests (require Anchor toolchain and local validator):

```
yarn test
```

## Frontend

```
cd frontend
cp .env.example .env
npm i
npm run dev
```

Set `VITE_SOLANA_RPC_ENDPOINT` in `.env` to your RPC or use the default devnet.

## Notes
- USDC operations use the SPL Token program; PDAs hold group and insurance vault authorities.
- Future: integrate DeFi yield protocols (Kamino/Solend) for Contribution-Plus-Yield.
