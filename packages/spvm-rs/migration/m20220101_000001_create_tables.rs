use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Replace the sample below with your own migration scripts
        manager
            .create_table(
                Table::create()
                    .table(Nonces::Table)
                    .if_not_exists()
                    .col(ColumnDef::new(Nonces::OwnerAddress).string().primary_key())
                    .col(ColumnDef::new(Nonces::Nonce).integer().not_null())
                    .to_owned(),
            )
            .await?;

        manager
            .create_table(
                Table::create()
                    .table(InitializedTickers::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(InitializedTickers::Ticker)
                            .string()
                            .primary_key(),
                    )
                    .col(
                        ColumnDef::new(InitializedTickers::IsInitialized)
                            .boolean()
                            .not_null(),
                    )
                    .to_owned(),
            )
            .await?;

        manager
            .create_table(
                Table::create()
                    .table(State::Table)
                    .if_not_exists()
                    .col(ColumnDef::new(State::Ticker).string().not_null())
                    .col(ColumnDef::new(State::OwnerAddress).string().not_null())
                    .col(
                        ColumnDef::new(State::Amount).integer().not_null().check(
                            Expr::col(State::Amount)
                                .gte(0)
                                .and(Expr::col(State::Amount).lte(65535)),
                        ),
                    )
                    .primary_key(
                        Index::create()
                            .name("pk_state_ticker_owneraddress")
                            .col(State::Ticker)
                            .col(State::OwnerAddress),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_state_ticker")
                            .from(State::Table, State::Ticker)
                            .to(InitializedTickers::Table, InitializedTickers::Ticker)
                            .on_delete(ForeignKeyAction::NoAction)
                            .on_update(ForeignKeyAction::NoAction),
                    )
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Replace the sample below with your own migration scripts
        manager
            .drop_table(Table::drop().table(Nonces::Table).to_owned())
            .await?;

        manager
            .drop_table(Table::drop().table(State::Table).to_owned())
            .await?;

        manager
            .drop_table(Table::drop().table(InitializedTickers::Table).to_owned())
            .await?;

        Ok(())
    }
}

#[derive(DeriveIden)]
enum Nonces {
    Table,
    OwnerAddress,
    Nonce,
}

#[derive(DeriveIden)]
enum InitializedTickers {
    Table,
    Ticker,
    IsInitialized,
}

#[derive(DeriveIden)]
enum State {
    Table,
    Ticker,
    OwnerAddress,
    Amount,
}
