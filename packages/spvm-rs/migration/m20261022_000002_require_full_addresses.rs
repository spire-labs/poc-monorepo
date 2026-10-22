use sea_orm_migration::{
    prelude::*,
    sea_orm::{ConnectionTrait, Statement},
};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let db = manager.get_connection();
        // Display formatting truncated addresses and discarded their middle
        // bytes. They cannot be safely expanded or assigned to a wallet.
        for table in ["state", "nonces"] {
            let invalid = db.query_one(Statement::from_string(
                db.get_database_backend(),
                format!("SELECT owner_address FROM {table} WHERE LENGTH(owner_address) <> 42 OR owner_address <> LOWER(owner_address) OR SUBSTR(owner_address, 1, 2) <> '0x' LIMIT 1"),
            )).await?;
            if invalid.is_some() {
                return Err(DbErr::Custom(format!(
                    "Legacy account identifiers in {table}: stop and reconcile from verified full addresses; do not clear balances or nonces"
                )));
            }
        }
        Ok(())
    }

    async fn down(&self, _manager: &SchemaManager) -> Result<(), DbErr> {
        // The guard changes no account data; never shorten identifiers again.
        Ok(())
    }
}
