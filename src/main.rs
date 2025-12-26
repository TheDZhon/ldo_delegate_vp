use alloy_network::Ethereum;
use alloy_primitives::{Address, U256};
use alloy_provider::RootProvider;
use alloy_sol_types::sol;
use anyhow::{Context, Result};
use clap::Parser;
use dotenvy::dotenv;
use futures::stream::{self, StreamExt};
use ldo_delegate_vp::{format_units, format_units_human, redact_rpc_url, unique_preserve_order};
use std::{iter, sync::Arc};

sol! {
    #[sol(rpc)]
    interface LidoVoting {
        function getDelegatedVoters(address _delegate, uint256 _offset, uint256 _limit) external view returns (address[] voters);
        function getVotingPowerMultipleAtVote(uint256 _voteId, address[] _voters) external view returns (uint256[] balances);
        function getVotingPowerMultiple(address[] _voters) external view returns (uint256[] balances);
    }
}

#[derive(Parser)]
#[command(version, about = "Fetch delegated voters sorted by voting power")]
struct Args {
    /// Vote ID to query historical voting power at. If omitted, queries current voting power.
    #[arg(short, long)]
    vote_id: Option<u64>,

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

    /// Concurrent requests for voting power fetching.
    #[arg(long, default_value_t = 5)]
    concurrency: usize,

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
    if args.concurrency == 0 {
        anyhow::bail!("--concurrency must be >= 1");
    }

    if !args.quiet {
        println!("🔗 RPC: {}", redact_rpc_url(&args.rpc_url));
        println!("📜 Contract: {}", args.contract_address);
        println!("👤 Delegate: {}", args.delegate_address);
    }

    let provider = Arc::new(RootProvider::<Ethereum>::new_http(
        args.rpc_url.parse().context("invalid RPC URL")?,
    ));
    let contract = LidoVoting::new(args.contract_address, provider);

    let vote_id = args.vote_id.map(U256::from);
    let mut delegated_voters: Vec<Address> = Vec::new();
    let mut offset = U256::ZERO;
    let limit = U256::from(args.page_size as u64);

    if !args.quiet {
        println!("\n📥 Fetching delegated voters...");
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
            println!("   ✓ Fetched {} voters", fetched);
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
        println!("   📊 Unique addresses: {}", addresses.len());
        match vote_id {
            Some(id) => println!("\n⏳ Calculating voting power at vote #{}...", id),
            None => println!("\n⏳ Calculating current voting power..."),
        }
    }

    let mut voting_power_map: Vec<(Address, U256)> = Vec::with_capacity(addresses.len());

    let mut stream = stream::iter(addresses.chunks(args.chunk_size))
        .map(|chunk| {
            let contract = contract.clone();
            let chunk = chunk.to_vec();
            async move {
                let balances: Vec<U256> = match vote_id {
                    Some(id) => contract
                        .getVotingPowerMultipleAtVote(id, chunk.clone())
                        .call()
                        .await
                        .context("getVotingPowerMultipleAtVote RPC call failed")?,
                    None => contract
                        .getVotingPowerMultiple(chunk.clone())
                        .call()
                        .await
                        .context("getVotingPowerMultiple RPC call failed")?,
                };

                anyhow::ensure!(
                    balances.len() == chunk.len(),
                    "voting power response length mismatch (got {}, expected {})",
                    balances.len(),
                    chunk.len()
                );

                Ok::<_, anyhow::Error>(
                    chunk
                        .into_iter()
                        .zip(balances.into_iter())
                        .collect::<Vec<_>>(),
                )
            }
        })
        .buffer_unordered(args.concurrency);

    while let Some(result) = stream.next().await {
        let pairs = result?;
        voting_power_map.extend(pairs);
    }

    // Sort by voting power descending
    voting_power_map.sort_by(|a, b| b.1.cmp(&a.1).then_with(|| a.0.cmp(&b.0)));

    // Separate non-zero and zero voting power addresses
    let (with_power, without_power): (Vec<_>, Vec<_>) = voting_power_map
        .iter()
        .partition(|(_, power)| !power.is_zero());

    let total_voting_power: U256 = with_power.iter().map(|(_, power)| *power).sum();

    // Print header
    println!();
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    match args.vote_id {
        Some(id) => println!("🗳️  VOTING POWER AT VOTE #{}", id),
        None => println!("🗳️  CURRENT VOTING POWER"),
    }
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");

    // Print active voters
    if !with_power.is_empty() {
        println!();
        println!("💎 ACTIVE VOTERS ({} addresses)", with_power.len());
        println!(
            "────────────────────────────────────────────────────────────────────────────────"
        );
        for (i, (address, power)) in with_power.iter().enumerate() {
            let power_str = format_units_human(*power, 18);
            println!("  #{:<3}  {}  {:>22} LDO", i + 1, address, power_str);
        }
    }

    // Print inactive voters summary
    if !without_power.is_empty() {
        println!();
        println!("💤 INACTIVE: {} addresses with 0 LDO", without_power.len());
    }

    // Print totals
    println!();
    println!("════════════════════════════════════════════════════════════════════════════════");
    println!(
        "🏆 TOTAL VOTING POWER:  {} LDO",
        format_units_human(total_voting_power, 18)
    );
    println!(
        "📊 Full precision:      {} LDO",
        format_units(total_voting_power, 18)
    );
    println!("════════════════════════════════════════════════════════════════════════════════");

    Ok(())
}
