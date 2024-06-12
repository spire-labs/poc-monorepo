//! `SeaORM` Entity. Generated by sea-orm-codegen 0.12.15

use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq)]
#[sea_orm(table_name = "initialized_tickers")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false, column_type = "Text")]
    pub ticker: String,
    pub is_initialized: bool,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(has_many = "super::state::Entity")]
    State,
}

impl Related<super::state::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::State.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}