use alloy_network::Ethereum;
use alloy_primitives::{address, Address, U256};
use alloy_provider::RootProvider;
use alloy_sol_types::sol;
use anyhow::Result;
use clap::Parser;
use dotenvy::dotenv;
use std::{env, sync::Arc};

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

    #[arg(short, long, default_value = "0x6D8D914205bB14104c0f95BfaDb4B1680EF60CCC")]
    delegate_address: String,
}

#[tokio::main]
async fn main() -> Result<()> {
    dotenv().ok();
    let Args { vote_id, delegate_address } = Args::parse();

    let rpc_url = env::var("RPC_URL").unwrap_or("https://eth.drpc.org".into());
    println!("RPC URL: {}", rpc_url);

    let provider = Arc::new(RootProvider::<Ethereum>::new_http(rpc_url.parse()?));

    let contract_address = address!("2e59A20f205bB85a89C53f1936454680651E618e");
    let delegate_address: Address = delegate_address.parse()?;
    let contract = LidoVoting::new(contract_address, provider);

    let vote_id = U256::from(vote_id);
    let mut delegated_voters = vec![];
    let mut offset = U256::ZERO;
    let limit = U256::from(100);

    println!("Fetching delegated voters for delegate {}...", delegate_address);
    loop {
        let voters = contract
            .getDelegatedVoters(delegate_address, offset, limit)
            .call()
            .await?
            .voters;

        if voters.is_empty() {
            break;
        }

        println!("Fetched {} voters", voters.len());
        delegated_voters.extend(voters.clone());

        if voters.len() < limit.to::<usize>() {
            break;
        }

        offset += limit;
    }

    println!("Total delegated voters fetched: {}", delegated_voters.len());

    let mut addresses = vec![delegate_address];
    addresses.extend(delegated_voters);

    let mut voting_power_map = vec![];

    println!("\n✅ Calculating voting power at vote ID {}...", vote_id);
    for chunk in addresses.chunks(100) {
        let balances = contract
            .getVotingPowerMultipleAtVote(vote_id, chunk.to_vec())
            .call()
            .await?
            .balances;

        voting_power_map.extend(chunk.iter().zip(balances.iter()).map(|(addr, power)| (*addr, *power)));
    }

    // Sort by voting power descending
    voting_power_map.sort_by(|a, b| b.1.cmp(&a.1));

    println!("\n📌 Addresses sorted by Voting Power (Descending):");
    for (address, power) in &voting_power_map {
        println!(
            "Address: {:?}, Voting power: {} LDO",
            address,
            format_units(*power, 18)
        );
    }

    let total_voting_power: U256 = voting_power_map.iter().map(|(_, power)| power).sum();

    println!(
        "\n🚀 Total voting power at vote ID {}: {} LDO",
        vote_id,
        format_units(total_voting_power, 18)
    );

    Ok(())
}

// Utility function clearly formatting U256 into readable decimals
fn format_units(value: U256, decimals: u32) -> String {
    let factor = U256::from(10).pow(U256::from(decimals));
    let whole = value / factor;
    let fractional = value % factor;

    if fractional.is_zero() {
        whole.to_string()
    } else {
        let fractional_str = format!("{:0>width$}", fractional, width = decimals as usize)
            .trim_end_matches('0')
            .to_string();
        format!("{}.{}", whole, fractional_str)
    }
}
