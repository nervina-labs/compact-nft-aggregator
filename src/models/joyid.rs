use crate::schema::joy_id_infos::dsl::joy_id_infos;
use crate::schema::joy_id_infos::{
    alg, avatar, cota_cell_id, credential_id, description, extension, lock_hash, name, nickname,
    pub_key,
};
use crate::schema::sub_key_infos::dsl::sub_key_infos;
use crate::schema::sub_key_infos::{
    alg as sub_alg, credential_id as sub_credential_id, lock_hash as sub_lock_hash,
    pub_key as sub_pub_key,
};
use crate::utils::error::Error;
use crate::utils::helper::parse_bytes_n;
use crate::POOL;
use diesel::*;
use log::error;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone, Eq, PartialEq, Default)]
pub struct JoyIDInfo {
    pub name:          String,
    pub avatar:        String,
    pub description:   String,
    pub extension:     String,
    pub nickname:      String,
    pub pub_key:       String,
    pub credential_id: String,
    pub alg:           String,
    pub cota_cell_id:  String,
    pub sub_keys:      Vec<SubKeyDb>,
}

#[derive(Queryable, Debug, Clone, Eq, PartialEq)]
pub struct JoyIDInfoDb {
    pub name:          String,
    pub avatar:        String,
    pub description:   String,
    pub extension:     String,
    pub nickname:      String,
    pub pub_key:       String,
    pub credential_id: String,
    pub alg:           String,
    pub cota_cell_id:  String,
}

#[derive(Serialize, Deserialize, Queryable, Debug, Clone, Eq, PartialEq)]
pub struct SubKeyDb {
    pub pub_key:       String,
    pub credential_id: String,
    pub alg:           String,
}

pub fn get_joyid_info_by_lock_hash(lock_hash_: [u8; 32]) -> Result<Option<JoyIDInfo>, Error> {
    let conn = &POOL.clone().get().expect("Mysql pool connection error");
    let lock_hash_hex = hex::encode(lock_hash_);
    let joyid_infos: Vec<JoyIDInfoDb> = joy_id_infos
        .select((
            name,
            avatar,
            description,
            extension,
            nickname,
            pub_key,
            credential_id,
            alg,
            cota_cell_id,
        ))
        .filter(lock_hash.eq(lock_hash_hex.clone()))
        .limit(1)
        .load::<JoyIDInfoDb>(conn)
        .map_or_else(
            |e| {
                error!("Query joyid info error: {}", e.to_string());
                Err(Error::DatabaseQueryError(e.to_string()))
            },
            Ok,
        )?;
    let sub_keys: Vec<SubKeyDb> = sub_key_infos
        .select((sub_pub_key, sub_credential_id, sub_alg))
        .filter(sub_lock_hash.eq(lock_hash_hex))
        .load::<SubKeyDb>(conn)
        .map_or_else(
            |e| {
                error!("Query sub keys info error: {}", e.to_string());
                Err(Error::DatabaseQueryError(e.to_string()))
            },
            Ok,
        )?;
    let joyid_info = joyid_infos.get(0).cloned().map(|info| JoyIDInfo {
        name: info.name,
        avatar: info.avatar,
        description: info.description,
        extension: info.extension,
        nickname: info.nickname,
        pub_key: info.pub_key,
        credential_id: info.credential_id,
        alg: info.alg,
        cota_cell_id: info.cota_cell_id,
        sub_keys,
    });
    Ok(joyid_info)
}

pub fn get_lock_hash_by_nickname(nickname_: &str) -> Result<Option<[u8; 32]>, Error> {
    let conn = &POOL.clone().get().expect("Mysql pool connection error");
    let lock_hashes = joy_id_infos
        .select(lock_hash)
        .filter(nickname.eq(nickname_.to_string()))
        .limit(1)
        .load::<String>(conn)
        .map_err(|e| {
            error!("Query lock hash by nickname error: {}", e.to_string());
            Error::DatabaseQueryError(e.to_string())
        })?;
    let lock_hash_ = lock_hashes
        .get(0)
        .cloned()
        .map(|hash| parse_bytes_n::<32>(hash).unwrap());
    Ok(lock_hash_)
}
