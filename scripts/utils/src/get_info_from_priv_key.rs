use ethers::{
    providers::{Http, Middleware, Provider},
    signers::{LocalWallet, Signer},
};
use tokio;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenvy::dotenv().ok();
    let priv_key = "0xdaafc7ff176bcb11eddfb1e6238ffe292e0e7fb9b9809a40b187c840776dd7b1";

    println!("The private key is: {}", priv_key);
    let wallet = priv_key.parse::<LocalWallet>()?;
    println!("The address is: {:#x}", wallet.address());
    let provider_url = std::env::var("ANVIL_RPC_URL").expect("ANVIL_RPC_URL is not set in .env file");

    let provider = Provider::<Http>::try_from(provider_url)?;

    let balance = provider.get_balance(wallet.address(), None).await?;
    println!("It has balance: {}", balance);

    Ok(())
}
