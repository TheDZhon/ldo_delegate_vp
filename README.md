# LDO Delegate Voting Power

![Rust](https://img.shields.io/badge/rust-1.88%2B-brightgreen.svg)
![Alloy](https://img.shields.io/badge/alloy-1.x-blue.svg)
![License](https://img.shields.io/badge/license-MIT-green.svg)

A robust Rust CLI tool built on [Alloy](https://github.com/alloy-rs/alloy) that fetches delegated voters for a Lido delegate and analyzes their voting power—either at the current block or at a specific historical snapshot.

## Key Features

- **Current & Historical Analysis**: Query current voting power or calculate power at any specific vote ID (snapshot).
- **Efficient Data Fetching**: Retrieves all delegated voters using paginated calls to the Lido Voting contract.
- **Privacy Focused**: Automatically redacts sensitive information from RPC URLs in logs.
- **Sorted Output**: Displays addresses sorted by LDO voting power in descending order.

## Prerequisites

- [Rust toolchain](https://rustup.rs/) (**Rust 1.88+**; required by Alloy 1.x)
- An Ethereum RPC URL (e.g., from [Infura](https://infura.io), [Alchemy](https://alchemy.com), or public endpoints like `https://eth.drpc.org`).

## Installation

1. Clone the repository:
   ```bash
   git clone https://github.com/TheDZhon/ldo_delegate_vp.git
   cd ldo_delegate_vp
   ```

2. Build the project:
   ```bash
   cargo build --release
   ```

## Usage

### Current Voting Power

Query the **current** voting power for the default delegate ([voteron.eth](https://etherscan.io/address/0x6D8D914205bB14104c0f95BfaDb4B1680EF60CCC)):

```bash
cargo run --release
```

### Historical Voting Power (at a Vote ID)

Query voting power at a specific vote snapshot:

```bash
cargo run --release -- --vote-id 180
```

### Custom Delegate

Analyze a different delegate's voters:

```bash
cargo run --release -- --delegate-address 0xYourAddressHere
```

Or combine with a vote ID:

```bash
cargo run --release -- --vote-id 180 --delegate-address 0xYourAddressHere
```

### RPC Configuration

You can provide the RPC URL via a command-line flag or an environment variable.

**Via Flag:**
```bash
cargo run --release -- --vote-id 180 --rpc-url https://your.rpc.provider
```

**Via Environment Variable:**
Create a `.env` file or export the variable:
```bash
export RPC_URL=https://your.rpc.provider
cargo run --release -- --vote-id 180
```

### Options

| Flag | Description | Default |
|------|-------------|---------|
| `--vote-id <ID>` | Vote ID to query historical voting power at. If omitted, queries current power. | (current) |
| `--delegate-address <ADDR>` | The delegate address to query. | `0x6D8D...` |
| `--contract-address <ADDR>` | Lido Voting contract address. | `0x2e59...` |
| `--rpc-url <URL>` | Ethereum RPC URL. | `https://eth.drpc.org` |
| `--page-size <N>` | Number of voters to fetch per request. | 100 |
| `--chunk-size <N>` | Number of addresses to query voting power for per batch. | 100 |
| `--concurrency <N>` | Number of concurrent requests for voting power fetching. | 5 |
| `--quiet` | Suppress progress logs (only output results). | `false` |

## Development

### Setup

Install git hooks to run CI checks before each commit:

```bash
./scripts/install-hooks.sh
```

This will run the following checks on every `git commit`:
- `cargo fmt -- --check` (formatting)
- `cargo clippy --all-targets --all-features -- -D warnings` (lints)
- `cargo test --all-targets --all-features` (tests)

To skip hooks temporarily: `git commit --no-verify`

### Running Tests

Run the unit tests to ensure everything is working correctly:

```bash
cargo test
```

### Project Structure

- `src/main.rs`: CLI entry point, argument parsing, and main logic flow.
- `src/lib.rs`: Helper functions and their unit tests (`format_units`, `redact_rpc_url`, etc.).

## License

This project is licensed under the [MIT License](LICENSE).
