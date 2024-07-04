use abi::Abi;
use ethers::{
    contract::Contract,
    prelude::*,
    providers::{Http, Provider},
    signers::{LocalWallet, Signer},
    types::{Address, TransactionRequest, U256},
    utils::Anvil,
};
use eyre::Result;
use serde_json::Value;
use std::{fs, sync::Arc};

#[tokio::main]
async fn main() -> Result<()> {
    let rpc_url = "http://127.0.0.1:8545";
    let provider = Provider::<Http>::try_from(rpc_url)?;
    let block_number = provider.get_block_number().await?;
    println!("Simulate against block number: {:?}", block_number);

    let anvil = Anvil::new()
        .fork("http://127.0.0.1:8545")
        .fork_block_number(block_number.as_u64())
        .spawn();
    let wallet: LocalWallet = anvil.keys()[0].clone().into();
    let client = Arc::new(SignerMiddleware::new(
        provider.clone(),
        wallet.with_chain_id(anvil.chain_id()),
    ));

    let abi_str = fs::read_to_string("./src/abi/univ2.json")?;
    let uniswap_abi: Abi = serde_json::from_str(&abi_str)?;
    let uniswap = Contract::new(
        "0x7a250d5630B4cF539739dF2C5dAcb4c659F2488D".parse::<Address>()?,
        uniswap_abi,
        client.clone(),
    );

    let dai_address = "0x6B175474E89094C44Da98b954EedeAC495271d0F".parse::<Address>()?;
    let erc20_abi_str = fs::read_to_string("./src/abi/erc20.json")?;
    let erc20_abi: Abi = serde_json::from_str(&erc20_abi_str)?;

    let dai_contract = Contract::new(dai_address, erc20_abi.clone(), client.clone());

    // Get balances before transaction
    let eth_balance_before = provider.get_balance(client.address(), None).await?;
    let dai_balance_before: U256 = dai_contract
        .method("balanceOf", client.address())?
        .call()
        .await?;

    println!(
        "ETH Balance Before: {:.18} ETH",
        eth_balance_before.as_u128() as f64 / 1e18
    );
    println!(
        "DAI Balance Before: {:.18} DAI",
        dai_balance_before.as_u128() as f64 / 1e18
    );

    // Amount of ETH to Swap for Tokens
    let amount_to_swap = U256::from(100_000_000_000_000_000u64); // 0.1 ETH in wei

    // Prepare and execute the swap from ETH to DAI
    let args = (
        U256::from(1_00u64), // Minimum amount of tokens to receive (e.g., 100 DAI in wei)
        vec![
            "0xC02aaA39b223FE8D0A0e5C4F27eAD9083C756Cc2".parse::<Address>()?,
            dai_address,
        ], // Path: ETH -> DAI
        client.address(),    // Recipient address
        U256::max_value(),   // Deadline: use max value for no specific deadline constraint
    );

    let tx = TransactionRequest::new()
        .to("0x7a250d5630B4cF539739dF2C5dAcb4c659F2488D".parse::<Address>()?)
        .value(amount_to_swap) // Value in ETH to be sent
        .data(uniswap.encode("swapExactETHForTokens", args)?)
        .gas(U256::from(200_000u64)); // Set a gas limit

    let tx_response = client.send_transaction(tx, None).await?.await?;

    // Get balances after transaction
    let eth_balance_after = provider.get_balance(client.address(), None).await?;
    let dai_balance_after: U256 = dai_contract
        .method("balanceOf", client.address())?
        .call()
        .await?;

    let eth_diff = eth_balance_before - eth_balance_after;
    let dai_diff = dai_balance_after - dai_balance_before;
    let tx_hash = tx_response.unwrap().transaction_hash;
    println!("Transaction Hash: {:?}", tx_hash);
    println!(
        "ETH Balance After: {:.18} ETH",
        eth_balance_after.as_u128() as f64 / 1e18
    );
    println!(
        "DAI Balance After: {:.18} DAI",
        dai_balance_after.as_u128() as f64 / 1e18
    );
    println!(
        "ETH Balance Change: {:.18} ETH",
        eth_diff.as_u128() as f64 / 1e18
    );
    println!(
        "DAI Balance Change: {:.18} DAI",
        dai_diff.as_u128() as f64 / 1e18
    );

    // let transaction_hash: H256 = tx_response.unwrap().transaction_hash;
    // Convert H256 to JSON and prepare it as a parameter array
    let params = serde_json::json!([tx_hash]); // Using serde_json::json! to ensure correct formatting
    let trace: Value = provider.request("trace_transaction", params).await?;
    println!("Transaction Trace: {:?}", trace);

    Ok(())
}
