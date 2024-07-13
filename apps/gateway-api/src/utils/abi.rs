use ethers::{
    core::abi::{decode, encode, Abi, ParamType, Token},
    types::{Address, Bytes, U256},
};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use spvm_rs::*;
use std::error::Error;

use super::types::PreconfirmationPayload;

#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct AbiEntry {
    pub inputs: Vec<AbiInput>,
    pub state_mutability: Option<String>,
    pub r#type: String,
    pub name: Option<String>,
    pub outputs: Option<Vec<AbiOutput>>,
}

#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct AbiInput {
    pub internal_type: String,
    pub name: String,
    pub r#type: String,
}

#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct AbiOutput {
    pub internal_type: String,
    pub name: String,
    pub r#type: String,
}

#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct Contract {
    pub address: Address,
    pub abi: Abi,
}

#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct ChainContracts {
    pub spvm: Contract,
    pub election: Contract,
    pub slashing: Contract,
}

#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct ContractData {
    pub chain_a: ChainContracts,
    pub chain_b: ChainContracts,
}

// helper function which pulls contract information (addresses, abis) from Anvil blockchains
pub async fn fetch_contract_data() -> Result<ContractData, reqwest::Error> {
    let client = Client::new();
    let response = client
        .get("http://localhost:5000/contracts")
        .send()
        .await?
        .json::<ContractData>()
        .await?;
    Ok(response)
}

// HELPER FUNCTIONS TO ABI-ENCODE STRUCTS
pub fn encode_tx_params(tx_params: &TransactionParams) -> Bytes {
    match tx_params {
        TransactionParams::Mint(params) => {
            let tokens = Token::Tuple(vec![
                Token::String(params.token_ticker.clone()),
                Token::Address(params.owner),
                Token::Uint(U256::from(params.supply)),
            ]);
            encode(&[tokens]).into()
        }
        TransactionParams::Transfer(params) => {
            let tokens = Token::Tuple(vec![
                Token::String(params.token_ticker.clone()),
                Token::Address(params.to),
                Token::Uint(U256::from(params.amount)),
            ]);
            encode(&[tokens]).into()
        }
    }
}

pub fn encode_tx_content(tx_content: &TransactionContent) -> Bytes {
    let encoded_tx_param = encode_tx_params(&tx_content.tx_param);

    let tokens = Token::Tuple(vec![
        Token::Address(tx_content.from),
        Token::Uint(U256::from(tx_content.tx_type)),
        Token::Bytes(encoded_tx_param.to_vec()),
        Token::Uint(U256::from(tx_content.nonce)),
    ]);

    encode(&[tokens]).into()
}

pub fn encode_preconf_payload(payload: &PreconfirmationPayload) -> Bytes {
    let encoded_tx = encode_transaction(&payload.transaction);
    let encoded_tip_tx = encode_transaction(&payload.tip_tx);

    let tokens = Token::Tuple(vec![
        Token::Bytes(encoded_tx.to_vec()),
        Token::Bytes(encoded_tip_tx.to_vec()),
        Token::Address(payload.preconfer_contract),
    ]);

    encode(&[tokens]).into()
}

pub fn encode_transaction(tx: &Transaction) -> Bytes {
    let encoded_cont = encode_tx_content(&tx.tx_content);
    let tokens = Token::Tuple(vec![
        Token::Bytes(encoded_cont.to_vec()),
        Token::Bytes(tx.tx_hash.as_bytes().to_vec()),
        Token::Bytes(tx.signature.into()),
    ]);

    encode(&[tokens]).into()
}

pub fn strip_0x_prefix(data: &str) -> &str {
    if data.starts_with("0x") {
        &data[2..]
    } else {
        data
    }
}

// HELPER FUNCTIONS TO DECODE ABI STRUCTS
pub fn decode_tx_content(data: &str) -> Result<TransactionContent, Box<dyn Error>> {
    // Attempt to decode data as hex string
    let decoded_data = match hex::decode(strip_0x_prefix(data)) {
        Ok(decoded) => decoded,
        Err(e) => {
            println!("Error decoding hex string: {:?}", e);
            return Err(Box::new(e));
        }
    };

    let tokens = decode(
        &[ParamType::Tuple(vec![
            ParamType::Address,
            ParamType::Uint(8),
            ParamType::Bytes,
            ParamType::Uint(32),
        ])],
        &decoded_data,
    )?;

    let tokens = match &tokens[0] {
        Token::Tuple(t) => t.clone(),
        _ => return Err("Expected Tuple type".into()),
    };

    if tokens.len() != 4 {
        return Err("Incorrect number of tokens".into());
    }

    let from = match &tokens[0] {
        Token::Address(addr) => *addr,
        _ => return Err("Expected Address type for 'from'".into()),
    };

    let tx_type = match &tokens[1] {
        Token::Uint(u) => u.low_u32() as u8,

        _ => return Err("Expected Uint type for 'tx_type'".into()),
    };

    let tx_param_tokens = match &tokens[2] {
        Token::Bytes(b) => decode(
            &[ParamType::Tuple(vec![
                ParamType::String,
                ParamType::Address,
                ParamType::Uint(16),
            ])],
            b,
        )?,
        _ => return Err("Expected Bytes type for 'tx_param'".into()),
    };

    let tx_params = decode_transaction_params(tx_type, tx_param_tokens)?;

    let nonce = match &tokens[3] {
        Token::Uint(u) => u.low_u32(),
        _ => return Err("Expected Uint type for 'nonce'".into()),
    };

    Ok(TransactionContent {
        from,
        tx_type,
        tx_param: tx_params,
        nonce,
    })
}

pub fn decode_transaction_params(
    tx_type: u8,
    params: Vec<Token>,
) -> Result<TransactionParams, Box<dyn Error>> {
    let params = match &params[0] {
        Token::Tuple(t) => t.clone(),
        _ => return Err("Expected Tuple type".into()),
    };

    match tx_type {
        0 => {
            if params.len() != 3 {
                return Err("Invalid number of parameters for Mint".into());
            }
            let token_ticker = match &params[0] {
                Token::String(s) => s.clone(),
                _ => return Err("Expected string for token_ticker".into()),
            };
            let owner = match &params[1] {
                Token::Address(addr) => *addr,
                _ => return Err("Expected address for owner".into()),
            };
            let supply = match &params[2] {
                Token::Uint(u) => u.as_u32() as u16,
                _ => return Err("Expected uint for supply".into()),
            };
            Ok(TransactionParams::Mint(MintTransactionParams {
                token_ticker,
                owner,
                supply,
            }))
        }
        1 => {
            if params.len() != 3 {
                return Err("Invalid number of parameters for Transfer".into());
            }
            let token_ticker = match &params[0] {
                Token::String(s) => s.clone(),
                _ => return Err("Expected string for token_ticker".into()),
            };
            let to = match &params[1] {
                Token::Address(addr) => *addr,
                _ => return Err("Expected address for to".into()),
            };
            let amount = match &params[2] {
                Token::Uint(u) => u.as_u32() as u16,
                _ => return Err("Expected uint for amount".into()),
            };
            Ok(TransactionParams::Transfer(TransferTransactionParams {
                token_ticker,
                to,
                amount,
            }))
        }
        _ => Err("Unsupported transaction type".into()),
    }
}
