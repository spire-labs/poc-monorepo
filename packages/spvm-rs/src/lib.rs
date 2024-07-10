use ::entity::{initialized_tickers, nonces, state};
use ethers::{
    core::abi::{decode, encode, ParamType, Token},
    core::utils::keccak256,
    types::{Address, Bytes, Signature, TxHash, U256},
};
use sea_orm::{
    entity::*, ActiveModelTrait, ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter, Set,
};
use serde::{Deserialize, Serialize};
use std::error::Error;
use std::{default::Default, fmt::format};

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct TransactionContent {
    pub from: Address,
    pub tx_type: u8, // Only the first 2 bits used
    pub tx_param: TransactionParams,
    pub nonce: u32,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum TransactionParams {
    Mint(MintTransactionParams),
    Transfer(TransferTransactionParams),
}

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct MintTransactionParams {
    pub token_ticker: String,
    pub owner: Address,
    pub supply: u16,
}

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct TransferTransactionParams {
    pub token_ticker: String,
    pub to: Address,
    pub amount: u16,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Transaction {
    pub tx_content: TransactionContent,
    pub tx_hash: TxHash,
    pub signature: Signature,
}

impl Default for TransactionParams {
    fn default() -> Self {
        Self::Mint(MintTransactionParams::default())
    }
}

impl Default for Transaction {
    fn default() -> Self {
        Self {
            tx_content: TransactionContent::default(),
            tx_hash: TxHash::default(),
            signature: Signature {
                r: ethers::types::U256::zero(),
                s: ethers::types::U256::zero(),
                v: 0,
            },
        }
    }
}

impl TransactionContent {
    pub async fn execute_raw_transaction(
        &self,
        db: &DatabaseConnection,
    ) -> Result<(), Box<dyn std::error::Error>> {
        self.check_validity(db).await?;
        match &self.tx_param {
            TransactionParams::Mint(params) => {
                Self::set_balance(&params.token_ticker, params.owner, params.supply, db).await?;
            }
            TransactionParams::Transfer(params) => {
                let sender_balance = Self::get_balance(&params.token_ticker, self.from, db).await?;
                println!("Sender balance: {:?}", sender_balance);
                if sender_balance < params.amount {
                    return Err("Insufficient balance".into());
                }
                let receiver_balance =
                    Self::get_balance(&params.token_ticker, params.to, db).await?;
                Self::set_balance(
                    &params.token_ticker,
                    self.from,
                    sender_balance - params.amount,
                    db,
                )
                .await?;
                Self::set_balance(
                    &params.token_ticker,
                    params.to,
                    receiver_balance + params.amount,
                    db,
                )
                .await?;
            }
        }

        let nonce = nonces::Entity::find()
            .filter(nonces::Column::OwnerAddress.eq(format!("{:#x}", self.from)))
            .one(db)
            .await?;

        match nonce {
            Some(record) => {
                let mut active_record = record.clone().into_active_model();
                active_record.nonce = Set(record.nonce + 1);
                active_record.update(db).await?;
            }
            None => {
                let record = nonces::ActiveModel {
                    owner_address: Set(format!("{:#x}", self.from)),
                    nonce: Set(1),
                };
                record.insert(db).await?;
            }
        }

        Ok(())
    }

    pub async fn check_validity(
        &self,
        db: &DatabaseConnection,
    ) -> Result<bool, Box<dyn std::error::Error>> {
        let nonce = nonces::Entity::find()
            .filter(nonces::Column::OwnerAddress.eq(format!("{:#x}", self.from)))
            .one(db)
            .await?;

        match nonce {
            Some(record) => {
                if record.nonce != self.nonce as i32 {
                    return Err("Invalid nonce".into());
                }
            }
            None => {
                if self.nonce != 0 {
                    return Err("Invalid nonce".into());
                }
            }
        }

        match &self.tx_param {
            TransactionParams::Mint(params) => {
                let initialized = initialized_tickers::Entity::find()
                    .filter(initialized_tickers::Column::Ticker.eq(&params.token_ticker))
                    .one(db)
                    .await?;

                match initialized {
                    Some(record) => {
                        if record.is_initialized {
                            Err("Token already initialized".into())
                        } else {
                            Ok(true)
                        }
                    }
                    None => Ok(true),
                }
            }
            TransactionParams::Transfer(params) => {
                let initialized = initialized_tickers::Entity::find()
                    .filter(initialized_tickers::Column::Ticker.eq(&params.token_ticker))
                    .one(db)
                    .await?;

                println!("Token ticker {:?}", params.token_ticker);
                match initialized {
                    Some(record) => {
                        if !record.is_initialized {
                            return Err("Token not initialized (Some)".into());
                        }
                    }
                    None => return Err("Token not initialized (None)".into()),
                }

                let balance = state::Entity::find()
                    .filter(state::Column::Ticker.eq(&params.token_ticker))
                    .filter(state::Column::OwnerAddress.eq(format!("{:#x}", self.from)))
                    .one(db)
                    .await?;

                println!("Params amount {:?}", params.amount);
                match balance {
                    Some(record) => {
                        println!("Record amount {:?}", record.amount);
                        if record.amount < params.amount as i32 {
                            return Err("Insufficient balance".into());
                        }
                    }
                    None => return Err("Insufficient balance".into()),
                }

                Ok(true)
            }
        }
    }

    pub async fn set_balance(
        ticker: &str,
        holder_address: Address,
        balance: u16,
        db: &DatabaseConnection,
    ) -> Result<(), Box<dyn std::error::Error>> {
        // Initialise ticker if not present or if set to False
        let initialized = initialized_tickers::Entity::find()
            .filter(initialized_tickers::Column::Ticker.eq(ticker))
            .one(db)
            .await?;

        match initialized {
            Some(record) => {
                let mut active_record = record.into_active_model();
                active_record.is_initialized = Set(true);
                active_record.update(db).await?;
            }
            None => {
                let record = initialized_tickers::ActiveModel {
                    ticker: Set(ticker.to_string()),
                    is_initialized: Set(true),
                };
                record.insert(db).await?;
            }
        }

        // Update balance
        let state = state::Entity::find()
            .filter(state::Column::Ticker.eq(ticker))
            .filter(state::Column::OwnerAddress.eq(format!("{:#x}", holder_address)))
            .one(db)
            .await?;

        match state {
            Some(record) => {
                let mut active_record = record.into_active_model();
                active_record.amount = Set(balance as i32);
                active_record.update(db).await?;
            }
            None => {
                let record = state::ActiveModel {
                    ticker: Set(ticker.to_string()),
                    owner_address: Set(format!("{:#x}", holder_address)),
                    amount: Set(balance as i32),
                };
                record.insert(db).await?;
            }
        }

        Ok(())
    }

    pub async fn get_balance(
        ticker: &str,
        holder_address: Address,
        db: &DatabaseConnection,
    ) -> Result<u16, Box<dyn std::error::Error>> {
        let balance = state::Entity::find()
            .filter(state::Column::Ticker.eq(ticker))
            .filter(state::Column::OwnerAddress.eq(format!("{:#x}", holder_address)))
            .one(db)
            .await?;

        Ok(balance.map_or(0, |record| record.amount as u16))
    }
}

impl Transaction {
    pub fn validate_signature(&self) -> Result<(), ethers::types::SignatureError> {
        self.signature.verify(self.tx_hash, self.tx_content.from)
    }

    pub async fn execute_transaction(
        &self,
        db: &DatabaseConnection,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let message = encode_tx_content(&self.tx_content);
        let hash = keccak256(message);

        if self.tx_hash == TxHash::from_slice(&hash) {
            self.validate_signature()?;
            // let connection = Connection::open(db)?;
            self.tx_content.execute_raw_transaction(db).await?;
            Ok(())
        } else {
            Err("Transaction hash mismatch".into())
        }
    }
}

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

pub fn decode_tx_content(data: &Bytes) -> Result<TransactionContent, Box<dyn Error>> {
    let tokens = decode(
        &[ParamType::Tuple(vec![
            ParamType::Address,
            ParamType::Uint(8),
            ParamType::Bytes,
            ParamType::Uint(32),
        ])],
        // &hex::decode(data).unwrap(),
        data,
    )?;
    println!("Tokens: {:?}", tokens);

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

#[cfg(test)]
mod tests {
    use std::env;

    use super::Transaction;
    use super::*;
    use sea_orm::{entity::prelude::*, Database, DbBackend, Schema};

    use ethers::{
        core::rand::thread_rng,
        signers::{LocalWallet, Signer},
    };

    async fn setup() -> (LocalWallet, DatabaseConnection) {
        let wallet0 = LocalWallet::new(&mut thread_rng());

        let db_path = env::var("DB").unwrap();
        let db = Database::connect(db_path).await.unwrap();
        setup_schema(&db).await;

        (wallet0, db)
    }

    async fn setup_schema(db: &DbConn) {
        let schema = Schema::new(DbBackend::Sqlite);

        let stmt0 = schema.create_table_from_entity(state::Entity);
        let stmt1 = schema.create_table_from_entity(nonces::Entity);
        let stmt2 = schema.create_table_from_entity(initialized_tickers::Entity);

        let _result = db.execute(db.get_database_backend().build(&stmt0)).await;

        let _result = db.execute(db.get_database_backend().build(&stmt1)).await;

        let _result = db.execute(db.get_database_backend().build(&stmt2)).await;
    }

    fn create_transaction(
        signer: &LocalWallet,
        tx_type: u8,
        tx_param: TransactionParams,
        nonce: u32,
    ) -> Transaction {
        let tx_content = TransactionContent {
            from: signer.address(),
            tx_type,
            tx_param,
            nonce,
        };

        let message = encode_tx_content(&tx_content);
        let tx_hash = TxHash::from_slice(&keccak256(message));
        let signature = signer.sign_hash(tx_hash).unwrap();

        Transaction {
            tx_content,
            tx_hash,
            signature,
        }
    }

    fn create_mint_transaction(
        ticker: &str,
        from: &LocalWallet,
        owner: Address,
        supply: u16,
        nonce: u32,
    ) -> Transaction {
        create_transaction(
            from,
            0,
            TransactionParams::Mint(MintTransactionParams {
                token_ticker: ticker.to_string(),
                owner,
                supply,
            }),
            nonce,
        )
    }

    fn create_transfer_transaction(
        ticker: &str,
        from: &LocalWallet,
        to: Address,
        amount: u16,
        nonce: u32,
    ) -> Transaction {
        create_transaction(
            from,
            1,
            TransactionParams::Transfer(TransferTransactionParams {
                token_ticker: ticker.to_string(),
                to,
                amount,
            }),
            nonce,
        )
    }

    #[async_std::test]
    async fn get_balance() {
        let (wallet, connection) = setup().await;
        let address = wallet.address();
        let ticker = "ABC";

        let result = TransactionContent::get_balance(ticker, address, &connection)
            .await
            .unwrap();
        assert_eq!(result, 0);
    }

    #[async_std::test]
    async fn set_balance() {
        let (wallet, connection) = setup().await;
        let address = wallet.address();
        let ticker = "ABC";

        assert!(
            TransactionContent::set_balance(ticker, address, 100, &connection)
                .await
                .is_ok()
        );

        let result = TransactionContent::get_balance(ticker, address, &connection)
            .await
            .unwrap();
        assert_eq!(result, 100);

        assert!(
            TransactionContent::set_balance(ticker, address, 0, &connection)
                .await
                .is_ok()
        );

        let result = TransactionContent::get_balance(ticker, address, &connection)
            .await
            .unwrap();
        assert_eq!(result, 0);
    }

    #[async_std::test]
    async fn execute_raw_mint() {
        let (wallet, connection) = setup().await;
        let wallet2 = LocalWallet::new(&mut thread_rng());

        let t1 = create_mint_transaction("ABC", &wallet, wallet2.address(), 100, 0);

        assert!(t1
            .tx_content
            .execute_raw_transaction(&connection)
            .await
            .is_ok());

        let result = TransactionContent::get_balance("ABC", wallet2.address(), &connection)
            .await
            .unwrap();
        assert_eq!(result, 100);

        let result = TransactionContent::get_balance("ABC", wallet.address(), &connection)
            .await
            .unwrap();
        assert_eq!(result, 0);

        let t2 = create_mint_transaction("DEF", &wallet, wallet.address(), 100, 1);

        assert!(t2
            .tx_content
            .execute_raw_transaction(&connection)
            .await
            .is_ok());

        let result = TransactionContent::get_balance("DEF", wallet.address(), &connection)
            .await
            .unwrap();
        assert_eq!(result, 100);

        let result = TransactionContent::get_balance("DEF", wallet2.address(), &connection)
            .await
            .unwrap();
        assert_eq!(result, 0);
    }

    #[async_std::test]
    async fn execute_raw_transfer() {
        let (wallet, connection) = setup().await;
        let wallet2 = LocalWallet::new(&mut thread_rng());

        let t1 = create_mint_transaction("ABC", &wallet, wallet2.address(), 100, 0);

        let t2 = create_transfer_transaction("ABC", &wallet2, wallet.address(), 50, 0);

        assert!(t1
            .tx_content
            .execute_raw_transaction(&connection)
            .await
            .is_ok());
        assert!(t2
            .tx_content
            .execute_raw_transaction(&connection)
            .await
            .is_ok());

        let result = TransactionContent::get_balance("ABC", wallet.address(), &connection)
            .await
            .unwrap();
        assert_eq!(result, 50);

        let result = TransactionContent::get_balance("ABC", wallet2.address(), &connection)
            .await
            .unwrap();
        assert_eq!(result, 50);
    }

    #[async_std::test]
    async fn execute_fail_mint_already_initialized() {
        let (wallet, connection) = setup().await;
        let wallet2 = LocalWallet::new(&mut thread_rng());

        let t1 = create_mint_transaction("ABC", &wallet, wallet2.address(), 100, 0);

        let t2 = create_mint_transaction("ABC", &wallet, wallet.address(), 100, 1);

        assert!(t1
            .tx_content
            .execute_raw_transaction(&connection)
            .await
            .is_ok());
        assert!(t2
            .tx_content
            .execute_raw_transaction(&connection)
            .await
            .is_err());
    }

    #[async_std::test]
    async fn execute_fail_mint_invalid_nonce() {
        let (wallet, connection) = setup().await;
        let wallet2 = LocalWallet::new(&mut thread_rng());

        let t1 = create_mint_transaction("ABC", &wallet, wallet2.address(), 100, 1);

        assert!(t1
            .tx_content
            .execute_raw_transaction(&connection)
            .await
            .is_err());
    }

    #[async_std::test]
    async fn execute_fail_transfer_insifficent_balance() {
        let (wallet, connection) = setup().await;
        let wallet2 = LocalWallet::new(&mut thread_rng());

        let t1 = create_mint_transaction("ABC", &wallet, wallet2.address(), 100, 0);

        let t2 = create_transfer_transaction("ABC", &wallet2, wallet.address(), 200, 0);

        assert!(t1
            .tx_content
            .execute_raw_transaction(&connection)
            .await
            .is_ok());
        assert!(t2
            .tx_content
            .execute_raw_transaction(&connection)
            .await
            .is_err());
    }

    #[async_std::test]
    async fn execute_fail_transfer_token_not_initialized() {
        let (wallet, connection) = setup().await;
        let wallet2 = LocalWallet::new(&mut thread_rng());

        let t1 = create_transfer_transaction("ABC", &wallet2, wallet.address(), 50, 0);

        assert!(t1
            .tx_content
            .execute_raw_transaction(&connection)
            .await
            .is_err());
    }

    #[async_std::test]
    async fn execute_fail_transfer_invalid_nonce() {
        let (wallet, connection) = setup().await;
        let wallet2 = LocalWallet::new(&mut thread_rng());

        let t1 = create_mint_transaction("ABC", &wallet, wallet2.address(), 100, 0);

        let t2 = create_transfer_transaction("ABC", &wallet2, wallet.address(), 50, 1);

        assert!(t1
            .tx_content
            .execute_raw_transaction(&connection)
            .await
            .is_ok());
        assert!(t2
            .tx_content
            .execute_raw_transaction(&connection)
            .await
            .is_err());
    }

    #[async_std::test]
    async fn execute_fail_hash_mismatch() {
        let (wallet, connection) = setup().await;
        let wallet2 = LocalWallet::new(&mut thread_rng());

        let mut t1 = create_mint_transaction("ABC", &wallet, wallet2.address(), 100, 0);

        t1.tx_hash = TxHash::from_slice(&[0; 32]);

        assert!(t1.execute_transaction(&connection).await.is_err());
    }

    #[async_std::test]
    async fn execute_fail_invalid_signature_wrong_addr() {
        let (wallet, connection) = setup().await;
        let wallet2 = LocalWallet::new(&mut thread_rng());

        let mut t1 = create_mint_transaction("ABC", &wallet, wallet2.address(), 100, 0);

        t1.signature = wallet2.sign_hash(t1.tx_hash).unwrap();

        assert!(t1.execute_transaction(&connection).await.is_err());
    }

    #[async_std::test]
    async fn execute_fail_invalid_signature_wrong_hash() {
        let (wallet, connection) = setup().await;
        let wallet2 = LocalWallet::new(&mut thread_rng());

        let mut t1 = create_mint_transaction("ABC", &wallet, wallet2.address(), 100, 0);

        t1.signature = wallet.sign_hash(TxHash::from_slice(&[0; 32])).unwrap();

        assert!(t1.execute_transaction(&connection).await.is_err());
    }

    #[async_std::test]
    async fn execute_transaction() {
        let (wallet, connection) = setup().await;
        let wallet2 = LocalWallet::new(&mut thread_rng());

        let t1 = create_mint_transaction("ABC", &wallet, wallet2.address(), 100, 0);

        t1.execute_transaction(&connection).await.unwrap();

        let result = TransactionContent::get_balance("ABC", wallet2.address(), &connection)
            .await
            .unwrap();
        assert_eq!(result, 100);
    }
}
