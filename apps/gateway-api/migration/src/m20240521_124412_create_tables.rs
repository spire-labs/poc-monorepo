use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(EnforcerMetadata::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(EnforcerMetadata::Address)
                            .string()
                            .not_null()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(EnforcerMetadata::Name).string().not_null())
                    .col(
                        ColumnDef::new(EnforcerMetadata::PreConfContracts)
                            .array(sea_query::ColumnType::String(None))
                            .not_null(),
                    )
                    .col(ColumnDef::new(EnforcerMetadata::Url).string().not_null())
                    .to_owned(),
            )
            .await?;

        manager
            .create_table(
                Table::create()
                    .table(EnforcerChallenge::Challenge)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(EnforcerChallenge::Challenge)
                            .string()
                            .not_null()
                            .primary_key(),
                    )
                    .to_owned(),
            )
            .await?;

        manager
            .create_table(
                Table::create()
                    .table(PreconfCommitment::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(PreconfCommitment::TxHash)
                            .string() //TODO: change type to match handler schema
                            .not_null()
                            .primary_key(),
                    )
                    .col(
                        ColumnDef::new(PreconfCommitment::Commitment)
                            .string()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(PreconfCommitment::Status)
                            .string()
                            .not_null(),
                    )
                    .to_owned(),
            )
            .await?;

        manager
            .create_table(
                Table::create()
                    .table(PreconfStatus::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(PreconfStatus::TxHash)
                            .string()
                            .not_null()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(PreconfStatus::Status).string().not_null())
                    .to_owned(),
            )
            .await?;
        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(EnforcerMetadata::Table).to_owned())
            .await?;
        manager
            .drop_table(Table::drop().table(EnforcerChallenge::Table).to_owned())
            .await?;
        manager
            .drop_table(Table::drop().table(PreconfCommitment::Table).to_owned())
            .await?;
        manager
            .drop_table(Table::drop().table(PreconfStatus::Table).to_owned())
            .await?;
        Ok(())
    }
}

#[derive(DeriveIden)]
enum EnforcerMetadata {
    Table,
    Address,
    Name,
    PreConfContracts,
    Url,
}

#[derive(DeriveIden)]
enum EnforcerChallenge {
    Table,
    Challenge,
}

#[derive(DeriveIden)]
enum PreconfCommitment {
    Table,
    TxHash,
    Commitment,
    Status,
}

#[derive(DeriveIden)]
enum PreconfStatus {
    Table,
    TxHash,
    Status,
}
