use alloy_network::Ethereum;
use alloy_primitives::{Address, U256};
use alloy_provider::RootProvider;
use alloy_sol_types::sol;
use anyhow::{Context, Result};
use clap::Parser;
use dotenvy::dotenv;
use ldo_delegate_vp::{format_units, redact_rpc_url, unique_preserve_order};
use std::{iter, sync::Arc};

sol! {
    #[sol(rpc)]
    interface LidoVoting {
        function getDelegatedVoters(address _delegate, uint256 _offset, uint256 _limit) external view returns (address[] voters);
        function getVotingPowerMultipleAtVote(uint256 _voteId, address[] _voters) external view returns (uint256[] balances);
    }
}

#[derive(Parser)]
#[command(version, about = "Fetch delegated voters sorted by voting power")]
struct Args {
    #[arg(short, long)]
    vote_id: u64,

    #[arg(
        short,
        long,
        default_value = "0x6D8D914205bB14104c0f95BfaDb4B1680EF60CCC"
    )]
    delegate_address: Address,

    /// Ethereum RPC URL (can also be provided via `RPC_URL` / `.env`).
    #[arg(long, env = "RPC_URL", default_value = "https://eth.drpc.org")]
    rpc_url: String,

    /// Lido Voting contract address (Ethereum mainnet).
    #[arg(long, default_value = "0x2e59A20f205bB85a89C53f1936454680651E618e")]
    contract_address: Address,

    /// Page size for `getDelegatedVoters` calls.
    #[arg(long, default_value_t = 100)]
    page_size: usize,

    /// Chunk size for `getVotingPowerMultipleAtVote` calls.
    #[arg(long, default_value_t = 100)]
    chunk_size: usize,

    /// Suppress progress logging (results still printed).
    #[arg(long)]
    quiet: bool,
}

#[tokio::main]
async fn main() -> Result<()> {
    dotenv().ok();
    let args = Args::parse();

    if args.page_size == 0 {
        anyhow::bail!("--page-size must be >= 1");
    }
    if args.chunk_size == 0 {
        anyhow::bail!("--chunk-size must be >= 1");
    }

    if !args.quiet {
        println!("RPC: {}", redact_rpc_url(&args.rpc_url));
        println!("Contract: {}", args.contract_address);
        println!("Delegate: {}", args.delegate_address);
    }

    let provider = Arc::new(RootProvider::<Ethereum>::new_http(
        args.rpc_url.parse().context("invalid RPC URL")?,
    ));
    let contract = LidoVoting::new(args.contract_address, provider);

    let vote_id = U256::from(args.vote_id);
    let mut delegated_voters: Vec<Address> = Vec::new();
    let mut offset = U256::ZERO;
    let limit = U256::from(args.page_size as u64);

    if !args.quiet {
        println!("Fetching delegated voters...");
    }
    loop {
        let voters: Vec<Address> = contract
            .getDelegatedVoters(args.delegate_address, offset, limit)
            .call()
            .await
            .context("getDelegatedVoters RPC call failed")?;

        if voters.is_empty() {
            break;
        }

        let fetched = voters.len();
        if !args.quiet {
            println!("Fetched {fetched} voters");
        }
        delegated_voters.extend(voters);

        if fetched < args.page_size {
            break;
        }

        offset += limit;
    }

    let addresses = unique_preserve_order(
        iter::once(args.delegate_address).chain(delegated_voters.into_iter()),
    );
    if !args.quiet {
        println!("Unique addresses to query: {}", addresses.len());
        println!("Calculating voting power at vote ID {}...", args.vote_id);
    }

    let mut voting_power_map: Vec<(Address, U256)> = Vec::with_capacity(addresses.len());
    for chunk in addresses.chunks(args.chunk_size) {
        let balances: Vec<U256> = contract
            .getVotingPowerMultipleAtVote(vote_id, chunk.to_vec())
            .call()
            .await
            .context("getVotingPowerMultipleAtVote RPC call failed")?;

        anyhow::ensure!(
            balances.len() == chunk.len(),
            "voting power response length mismatch (got {}, expected {})",
            balances.len(),
            chunk.len()
        );

        voting_power_map.extend(chunk.iter().copied().zip(balances.into_iter()));
    }

    // Sort by voting power descending
    voting_power_map.sort_by(|a, b| b.1.cmp(&a.1).then_with(|| a.0.cmp(&b.0)));

    println!("\nAddresses sorted by Voting Power (Descending):");
    for (address, power) in &voting_power_map {
        println!(
            "Address: {address}, Voting power: {} LDO",
            format_units(*power, 18)
        );
    }

    let total_voting_power: U256 = voting_power_map.iter().map(|(_, power)| *power).sum();

    println!(
        "\nTotal voting power at vote ID {}: {} LDO",
        args.vote_id,
        format_units(total_voting_power, 18)
    );

    Ok(())
}
