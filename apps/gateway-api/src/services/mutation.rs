use rand::distributions::Alphanumeric;
use rand::{thread_rng, Rng};
use sea_orm::*;

use super::QueryDB;
use crate::utils::types::*;
use ::entity::{challenge, enforcer_metadata, preconf_commitment, preconf_status};

pub struct MutationDB;

impl MutationDB {
    pub async fn register_enforcer_metadata(
        db: &DbConn,
        data: RegisterEnforcerMetadata,
    ) -> Result<Option<enforcer_metadata::Model>, DbErr> {
        enforcer_metadata::ActiveModel {
            address: Set(data.address.to_owned()),
            name: Set(data.name.to_owned()),
            pre_conf_contracts: Set(data.preconf_contracts.to_owned()),
            url: Set(data.url.to_owned()),
            ..Default::default()
        }
        .insert(db)
        .await?;

        let enforcer = QueryDB::get_enforcer_by_address(db, data.address).await?;

        return Ok(enforcer);
    }

    pub async fn register_preconf_commitment(
        db: &DbConn,
        data: String, //TODO: change to PreconfirmationCommitment type after writing migration
    ) -> Result<Option<preconf_commitment::Model>, DbErr> {
        preconf_commitment::ActiveModel {
            commitment: Set(data.to_owned()),
            status: Set("accepted".to_owned()),
            ..Default::default()
        }
        .insert(db)
        .await?;

        let commitment = QueryDB::get_preconf_commitment_by_tx_hash(db, data).await?;

        return Ok(commitment);
    }

    pub async fn create_challenge_string(db: &DbConn) -> Result<Option<challenge::Model>, DbErr> {
        let r = thread_rng()
            .sample_iter(&Alphanumeric)
            .take(30)
            .collect::<Vec<_>>();
        let rand_string = String::from_utf8_lossy(&r).to_string();

        challenge::ActiveModel {
            challenge: Set(rand_string.to_owned()),
            ..Default::default()
        }
        .insert(db)
        .await?;

        let challenge_string = QueryDB::get_challenge_string(db, rand_string).await?;

        return Ok(challenge_string);
    }

    pub async fn update_preconf_status(
        db: &DbConn,
        data: UpdatePreconfirmationStatus,
    ) -> Result<Option<preconf_status::Model>, DbErr> {
        //UGLY HACK: can't clone data object because rust doesn't allow copy on strings
        let new_tx_hash = data.tx_hash.clone();
        let new_status = data.status.clone();

        preconf_status::ActiveModel {
            tx_hash: Set(new_tx_hash.clone()).to_owned(),
            status: Set(new_status).to_owned(),
            ..Default::default()
        }
        .insert(db)
        .await?;

        let status = QueryDB::get_preconf_status_by_tx_hash(db, new_tx_hash).await?;

        return Ok(status);
    }
}
