use sea_orm::*;

use ::entity::prelude::{Challenge, EnforcerMetadata, PreconfCommitment, PreconfStatus};
use ::entity::{challenge, enforcer_metadata, preconf_commitment, preconf_status};

pub struct QueryDB;

impl QueryDB {
    pub async fn get_enforcer_by_address(
        db: &DbConn,
        address: String,
    ) -> Result<Option<enforcer_metadata::Model>, DbErr> {
        EnforcerMetadata::find_by_id(address).one(db).await
    }

    pub async fn get_preconf_commitment_by_tx_hash(
        db: &DbConn,
        tx_hash: String,
    ) -> Result<Option<preconf_commitment::Model>, DbErr> {
        PreconfCommitment::find_by_id(tx_hash).one(db).await
    }

    pub async fn get_preconf_status_by_tx_hash(
        db: &DbConn,
        tx_hash: String,
    ) -> Result<Option<preconf_status::Model>, DbErr> {
        PreconfStatus::find_by_id(tx_hash).one(db).await
    }

    pub async fn get_challenge_string(
        db: &DbConn,
        challenge: String,
    ) -> Result<Option<challenge::Model>, DbErr> {
        Challenge::find_by_id(challenge).one(db).await
    }
}
