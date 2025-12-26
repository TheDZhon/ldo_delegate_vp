# LDO Delegate Voting Power

![Rust](https://img.shields.io/badge/rust-1.88%2B-brightgreen.svg)
![Alloy](https://img.shields.io/badge/alloy-1.x-blue.svg)
![License](https://img.shields.io/badge/license-MIT-green.svg)

A small Rust CLI (built on [Alloy](https://github.com/alloy-rs/alloy)) that fetches delegated voters
for a Lido delegate and prints addresses sorted by LDO voting power at a specific vote ID.

## Key features

- Fetch **all delegated voters** for a delegate address.
- Query **voting power** at a specific vote ID (delegate + delegated voters).
- Sort output by voting power (descending).
- Avoid leaking secrets by logging a **redacted RPC host** (no query params / credentials).

## Prerequisites

- [Rust toolchain](https://rustup.rs/) (**Rust 1.88+**; required by Alloy 1.x)

## Installation

```bash
git clone https://github.com/TheDZhon/ldo_delegate_vp.git
cd ldo_delegate_vp
cargo build --release
```

## Usage

Run with the default delegate address ([voteron.eth](https://etherscan.io/address/0x6D8D914205bB14104c0f95BfaDb4B1680EF60CCC)):

```bash
cargo run --release -- --vote-id 180
```

Run with a custom delegate address:

```bash
cargo run --release -- --vote-id 180 --delegate-address 0xYourAddressHere
```

### RPC configuration

You can pass an RPC URL directly or set it via environment variable / `.env`:

```bash
# flag
cargo run --release -- --vote-id 180 --rpc-url https://your.rpc.provider

# env var (also works via `.env`)
export RPC_URL=https://your.rpc.provider
cargo run --release -- --vote-id 180
```

### Other options

- `--contract-address <ADDRESS>`: override the Lido Voting contract address
- `--page-size <N>`: page size for delegated voter fetch (default: 100)
- `--chunk-size <N>`: chunk size for voting power queries (default: 100)
- `--quiet`: suppress progress logs

## Tests

```bash
cargo test
```

## License

Licensed under the [MIT License](LICENSE).
