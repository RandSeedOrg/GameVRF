# GameVRF

Public verification resources for RandSeed game randomness, game algorithms, and Internet Computer (IC) canister integrations.

GameVRF is intended to make game result generation transparent and independently verifiable. It includes Rust algorithm libraries, IC canisters, a React/Vite verification app, and game-specific verification documentation.

## What is in this repository

- **Game algorithm implementations** used to reproduce supported game outcomes from randomness inputs.
- **Verification canisters** for exposing or validating game verification logic on the Internet Computer.
- **Random oracle canister** code used by the verification flow.
- **Game verification web app** for user-facing verification workflows.
- **Documentation and screenshots** explaining how to verify supported games.

## Repository structure

```text
GameVRF/
├── apps/
│   └── game-verify/          # React + TypeScript + Vite verification app
├── canisters/
│   ├── game-verify/          # IC canister for game verification APIs
│   └── random-oracle/        # IC canister for randomness/oracle functionality
├── doc/
│   ├── daily4/               # Daily4 verification guide and screenshots
│   └── Mines/                # Mines verification documentation placeholder
├── libraries/
│   └── game-algorithms/      # Shared Rust game algorithm library
├── Cargo.toml                # Rust workspace configuration
├── package.json              # npm workspace configuration
└── README.md
```

## Supported verification guides

| Game | Guide | Status |
| --- | --- | --- |
| Daily4 | [doc/daily4/daily4vrf.md](https://github.com/RandSeedOrg/GameVRF/blob/main/doc/daily4/daily4vrf.md) | Available |
| Mines | [doc/Mines/minesvrf.md](https://github.com/RandSeedOrg/GameVRF/blob/main/doc/Mines/minesvrf.md) | Placeholder |

When additional games support public verification, add their guide links to this table.

## Technology stack

### Rust / IC canisters

- Rust workspace managed by `Cargo.toml`
- IC canister crates under `canisters/`
- Shared game algorithms under `libraries/game-algorithms/`
- Key workspace dependencies include `candid`, `ic-cdk`, `ic-stable-structures`, `serde`, `sha2`, `rand`, and `rand_chacha`

### Frontend

- React
- TypeScript
- Vite
- npm workspaces
- DFINITY JavaScript packages for IC integration

## Getting started

### Prerequisites

- Rust toolchain compatible with `rust-toolchain.toml`
- Node.js and npm
  - Team baseline: Node.js `>=20.19.0`
  - The current root package manifest is permissive, but contributors should prefer the team baseline for consistency.

### Install frontend dependencies

```bash
npm install
```

### Run the verification app locally

```bash
npm --workspace apps/game-verify run start
```

Environment-specific modes are also available:

```bash
npm --workspace apps/game-verify run start:dev
npm --workspace apps/game-verify run start:test
```

### Build the verification app

```bash
npm --workspace apps/game-verify run build:dev
npm --workspace apps/game-verify run build:test
npm --workspace apps/game-verify run build:prod
```

### Lint the verification app

```bash
npm --workspace apps/game-verify run lint
```

### Rust checks

```bash
cargo fmt --all --check
cargo clippy --workspace --all-targets
cargo test --workspace
```

## Verification workflow

A typical verification flow is:

1. Open the game-specific verification guide in `doc/`.
2. Collect the required public inputs described by that guide, such as round identifiers, randomness seeds, or result data.
3. Reproduce the result using the documented algorithm and/or verification app.
4. Compare the reproduced result against the published game result.

For a concrete example, see the Daily4 guide: [doc/daily4/daily4vrf.md](https://github.com/RandSeedOrg/GameVRF/blob/main/doc/daily4/daily4vrf.md).

## Development notes

- Keep game algorithms deterministic and reproducible.
- Prefer integer math for game outcome, odds, payout, and verification logic.
- Do not mutate canister state from query methods.
- Avoid holding mutable state across inter-canister `await` calls.
- Keep Candid types and generated bindings consistent with the canister interfaces.
- Use the existing workspace structure before introducing new frameworks or packages.

## Adding a new game verification guide

1. Add the game algorithm implementation under `libraries/game-algorithms/src/` when applicable.
2. Add or update canister APIs under `canisters/` when verification requires canister support.
3. Add the documentation under `doc/<game-name>/`.
4. Include all required public inputs, reproduction steps, and expected outputs in the guide.
5. Add screenshots or diagrams only when they clarify the verification process.
6. Update the **Supported verification guides** table in this README.

Suggested guide naming format:

```text
doc/<game-name>/<game-name>vrf.md
```

## License

See [LICENSE](https://github.com/RandSeedOrg/GameVRF/blob/main/LICENSE).
