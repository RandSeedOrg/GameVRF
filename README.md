# GameVRF

RandSeed.org is building an open, verifiable randomness layer for on-chain and online games. GameVRF is part of that mission: it gives players, developers, partners, and auditors a public place to inspect how RandSeed game outcomes are produced from randomness inputs. Instead of asking users to trust a black box, this repository publishes the core game algorithms, verification canister code, and verification app structure so results can be independently reproduced.

This repository is both an engineering resource and a transparency resource. If someone wants to check game integrity, they can review the Rust implementation, run the deterministic tests, reproduce a game result from the public inputs, and even deploy or call the verification logic locally for their own experiments.

## What this repository provides

- **Open game algorithms** for supported RandSeed games.
- **Deterministic result generation** from public randomness inputs such as seeds and round numbers.
- **Shared Rust library code** that can be reviewed, tested, reused, or embedded in a verifier.
- **Internet Computer canister code** for exposing verification functions.
- **A web verification app** for user-facing verification workflows.
- **Game documentation** that explains how a player or auditor can reproduce supported game results.

## Repository structure

```text
GameVRF/
├── apps/
│   └── game-verify/                    # React + TypeScript + Vite verification app
├── canisters/
│   ├── game-verify/                    # IC canister exposing game verification APIs
│   │   └── src/
│   │       ├── daily4.rs               # Daily4 verification canister functions
│   │       ├── keno.rs                 # Keno verification canister functions
│   │       └── lib.rs                  # Canister entry module
│   └── random-oracle/                  # Randomness/oracle canister code
├── doc/
│   ├── daily4/                         # Daily4 verification guide and screenshots
│   └── Mines/                          # Mines verification documentation placeholder
├── libraries/
│   └── game-algorithms/                # Shared Rust game algorithm library
│       └── src/
│           ├── common/                 # Shared randomness, seed, and helper logic
│           │   ├── chacha.rs           # ChaCha-based deterministic number generation
│           │   ├── seed_format.rs      # Seed formatting helpers
│           │   └── seed_mixer.rs       # Seed mixing by round/game input
│           ├── daily4/                 # Daily4 algorithm implementation
│           ├── keno/                   # Keno algorithm implementation
│           │   └── mod.rs              # Keno deterministic number generation
│           ├── mines/                  # Mines algorithm implementation
│           └── lib.rs                  # Public algorithm modules
├── Cargo.toml                          # Rust workspace configuration
├── package.json                        # npm workspace configuration
└── README.md
```

The most important folder for checking game integrity is:

```text
libraries/game-algorithms/src/
```

That folder contains the deterministic game logic. For example, Keno lives here:

```text
libraries/game-algorithms/src/keno/mod.rs
```

The Keno algorithm takes a 32-byte seed, mixes it with a round value, and uses the shared ChaCha-based generator to produce a fixed count of unique numbers inside the configured range. Because the same inputs always produce the same output, anyone can reproduce and compare the result.

## How to verify game integrity

A reader, player, or auditor can verify a supported game by following this process:

1. **Find the game algorithm**
   - Keno: `libraries/game-algorithms/src/keno/mod.rs`
   - Daily4: `libraries/game-algorithms/src/daily4/mod.rs`
   - Mines: `libraries/game-algorithms/src/mines/mod.rs`

2. **Review the shared randomness helpers**
   - Seed mixing: `libraries/game-algorithms/src/common/seed_mixer.rs`
   - Deterministic number generation: `libraries/game-algorithms/src/common/chacha.rs`
   - Seed formatting: `libraries/game-algorithms/src/common/seed_format.rs`

3. **Collect the public verification inputs**
   - Game name
   - Round or draw identifier
   - Public seed/randomness value
   - Any game-specific parameters, such as number range or draw count
   - Published game result to compare against

4. **Run the same algorithm locally**
   - The Rust code is deterministic.
   - The same seed, round, and parameters should reproduce the same game result.

5. **Compare outputs**
   - If the locally reproduced output matches the published game result, the result is consistent with the open algorithm and public inputs.
   - If it does not match, the input data, game parameters, or published result should be investigated.

## Keno verification example

The Keno source is in `libraries/game-algorithms/src/keno/mod.rs`.

At a high level, Keno verification checks that:

1. A 32-byte root seed is provided.
2. The game round is mixed into that seed.
3. The mixed seed is passed into the shared ChaCha-based generator.
4. The generator returns the configured number of unique balls within the allowed range.
5. The reproduced numbers match the published Keno result.

The public Keno function is:

```rust
pub fn generate_numbers<T: SeedMixableNumber>(
  seed: [u8; 32],
  round: T,
  count: usize,
  min: u8,
  max: u8,
) -> Vec<u8>
```

This makes the integrity check straightforward: given the same `seed`, `round`, `count`, `min`, and `max`, the function must produce the same Keno numbers.

## Run verification code locally

You can inspect and run the Rust verification logic locally with Cargo.

### Run all Rust tests

```bash
cargo test --workspace
```

### Run only the game algorithm library tests

```bash
cargo test -p game-algorithms
```

### Run checks used by contributors

```bash
cargo fmt --all --check
cargo clippy --workspace --all-targets
cargo test --workspace
```

These commands let a reviewer confirm that the deterministic algorithms and included test cases behave as expected.

## Try the verification function locally

The game algorithms are written as a reusable Rust library under `libraries/game-algorithms`. A developer can create a small local Rust program or test that imports the library and calls a function such as `keno::generate_numbers` with known inputs.

For canister-based verification, the repository includes IC canister crates under `canisters/`:

- `canisters/game-verify` depends on `libraries/game-algorithms` and exposes game verification functions.
- `canisters/random-oracle` contains randomness/oracle canister code.

This repository does not currently include a root `dfx.json`, so local IC deployment may require adding a local DFX configuration or importing these canister crates into a local DFX project. Once configured, a developer can deploy the verification canister locally, call the Keno or Daily4 verification methods, and compare the returned result with the published game output.

## Verification app

The frontend verification app is in:

```text
apps/game-verify/
```

Install dependencies:

```bash
npm install
```

Run the app locally:

```bash
npm --workspace apps/game-verify run start
```

Other available app commands:

```bash
npm --workspace apps/game-verify run start:dev
npm --workspace apps/game-verify run start:test
npm --workspace apps/game-verify run build:dev
npm --workspace apps/game-verify run build:test
npm --workspace apps/game-verify run build:prod
npm --workspace apps/game-verify run lint
```

## Supported verification guides

| Game | Guide | Status |
| --- | --- | --- |
| Daily4 | [doc/daily4/daily4vrf.md](https://github.com/RandSeedOrg/GameVRF/blob/main/doc/daily4/daily4vrf.md) | Available |
| Mines | [doc/Mines/minesvrf.md](https://github.com/RandSeedOrg/GameVRF/blob/main/doc/Mines/minesvrf.md) | Placeholder |
| Keno | Source: [libraries/game-algorithms/src/keno/mod.rs](https://github.com/RandSeedOrg/GameVRF/blob/main/libraries/game-algorithms/src/keno/mod.rs) | Algorithm available |

When additional games support public verification, add their guide links to this table.

## Why this matters

Game integrity depends on reproducibility. RandSeed.org uses this repository to make the verification path visible:

- The randomness transformation is public.
- The game algorithm is public.
- The canister integration code is public.
- The verification app is public.
- The result can be reproduced independently.

That transparency helps players trust the games they play and helps developers or auditors confirm that RandSeed game results are generated by deterministic, reviewable logic rather than hidden server-side behavior.

## Development notes

- Keep game algorithms deterministic and reproducible.
- Prefer integer math for game outcome, odds, payout, and verification logic.
- Keep shared game logic in `libraries/game-algorithms` when possible.
- Keep canister APIs in `canisters/game-verify` focused on exposing verification behavior.
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
