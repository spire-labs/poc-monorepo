use ethers::signers::{LocalWallet, Signer};
use tokio;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let priv_key = "0xdaafc7ff176bcb11eddfb1e6238ffe292e0e7fb9b9809a40b187c840776dd7b1";

    println!("The private key is: {}", priv_key);
    let wallet = priv_key.parse::<LocalWallet>()?;
    println!("The address is: {:#x}", wallet.address());
    // println!("The public key is: {}", wallet.public_key());

    Ok(())
}
