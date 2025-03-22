# 🔷 LDO Delegate Voting Power

![Rust](https://img.shields.io/badge/rust-1.77.2-brightgreen.svg)
![Alloy](https://img.shields.io/badge/alloy-0.12.6-blue.svg)
![License](https://img.shields.io/badge/license-MIT-green.svg)

Simple CLI application written in Rust, using Alloy, to fetch and display Lido (LDO) on-chain voting power for a selected delegate and specific vote ID.

## 🚀 Key Features

- Fetch **all delegated voters** for a specified delegate address.
- Calculate and display **voting power** clearly, in a human-readable format (LDO).
- Sort addresses by voting power in descending order.
- Easy-to-use command-line interface.

## ⚙️ Prerequisites

- [Rust toolchain](https://rustup.rs/) (stable, v1.77.2 or later recommended)

## 📥 Installation

Clone this repository and navigate to the directory:

```bash
git clone <your-repo-url>
cd delegate_voting_power
```

Build the project:

```bash
cargo build --release
```

## ▶️ Usage

Run with default delegate address:

```bash
cargo run --release -- --vote-id 180
```

Run with a custom delegate address:

```bash
cargo run --release -- --vote-id 180 --delegate-address 0xYourAddressHere
```

### Environment Variables

You can optionally set the Ethereum RPC URL via `.env`:

```bash
RPC_URL=https://your.rpc.provider
```

(Default: `https://eth.drpc.org`)

## 📋 License

Licensed under the [MIT License](LICENSE).

---

Made with 🦀 using Rust and Alloy.

