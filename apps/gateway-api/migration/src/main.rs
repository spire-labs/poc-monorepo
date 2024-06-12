use sea_orm_migration::prelude::*;

#[async_std::main]
pub async fn main() {
    cli::run_cli(migration::Migrator).await;
}
