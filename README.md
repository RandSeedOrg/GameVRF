# GameVRF

Randseed.org's Public repository for **game algorithms**, **RandSeed query logic**, and **verification documentation**.

## Purpose

This repository provides a public reference for:

- game algorithm documentation
- RandSeed query documentation
- verification instructions for supported games
- supporting assets related to provable randomness and result verification

The goal is to make the game result generation and verification process transparent and publicly accessible.

## Repository structure

The repository is organized by documentation and related implementation files.

```text
GameVRF/
├── doc/
│   ├── daily4/
│   │   ├── daily4vrf.md
│   │   ├── ProvableRandomness.png
│   │   ├── verify.png
│   │   ├── input1seed.png
│   │   └── input4seeds.png
│   └── ...
├── src/
├── canisters/
├── scripts/
└── README.md
```

### Structure overview

- **doc/** — documentation for games verification instructions
- **src/** — source code for game logic and supporting functions
- **canisters/** — canister-related code for query and verification
- **scripts/** — utility scripts for development and maintenance

## Verification instructions

Detailed verification instructions for each game are listed below.

- **Daily4** — [doc/daily4/daily4vrf.md](https://github.com/RandSeedOrg/GameVRF/blob/main/doc/daily4/daily4vrf.md)


## Future games

When additional games support verification, their detailed instruction links should be added here in the same format.

Example:

- **Game Name** — `doc/game-name/game-namevrf.md`
