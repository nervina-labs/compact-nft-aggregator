use crate::models::claim::ClaimDb;
use crate::models::common::get_all_cota_by_lock_hash;
use crate::models::define::DefineDb;
use crate::models::hold::HoldDb;
use crate::models::withdrawal::WithdrawDb;
use crate::smt::constants::{
    CLAIM_NFT_SMT_TYPE, DEFINE_NFT_SMT_TYPE, HOLD_NFT_SMT_TYPE, WITHDRAWAL_NFT_SMT_TYPE,
};
use crate::utils::error::Error;
use crate::utils::helper::diff_time;
use chrono::prelude::*;
use cota_smt::common::{Uint16, Uint32, *};
use cota_smt::molecule::prelude::*;
use cota_smt::smt::SMT;
use cota_smt::smt::{blake2b_256, H256};

pub fn generate_define_key(cota_id: [u8; 20]) -> (DefineCotaNFTId, H256) {
    let cota_id = CotaId::from_slice(&cota_id).unwrap();
    let smt_type = Uint16::from_slice(&DEFINE_NFT_SMT_TYPE).unwrap();
    let define_key = DefineCotaNFTIdBuilder::default()
        .cota_id(cota_id)
        .smt_type(smt_type)
        .build();
    let mut define_key_bytes = [0u8; 32];
    define_key_bytes[0..22].copy_from_slice(define_key.as_slice());
    let key = H256::from(define_key_bytes);
    (define_key, key)
}

pub fn generate_define_value(
    total: [u8; 4],
    issued: [u8; 4],
    configure: u8,
) -> (DefineCotaNFTValue, H256) {
    let define_value = DefineCotaNFTValueBuilder::default()
        .total(Uint32::from_slice(&total).unwrap())
        .issued(Uint32::from_slice(&issued).unwrap())
        .configure(Byte::from(configure))
        .build();
    let mut define_value_bytes = [0u8; 32];
    define_value_bytes[0..9].copy_from_slice(define_value.as_slice());
    let value = H256::from(define_value_bytes);
    (define_value, value)
}

pub fn generate_hold_key(cota_id: [u8; 20], token_index: [u8; 4]) -> (CotaNFTId, H256) {
    let hold_key = CotaNFTIdBuilder::default()
        .cota_id(CotaId::from_slice(&cota_id).unwrap())
        .smt_type(Uint16::from_slice(&HOLD_NFT_SMT_TYPE).unwrap())
        .index(Uint32::from_slice(&token_index).unwrap())
        .build();
    let mut hold_key_bytes = [0u8; 32];
    hold_key_bytes[0..26].copy_from_slice(hold_key.as_slice());
    let key = H256::from(hold_key_bytes);
    (hold_key, key)
}

pub fn generate_hold_value(
    configure: u8,
    state: u8,
    characteristic: [u8; 20],
) -> (CotaNFTInfo, H256) {
    let hold_value = CotaNFTInfoBuilder::default()
        .characteristic(Characteristic::from_slice(&characteristic).unwrap())
        .configure(Byte::from(configure))
        .state(Byte::from(state))
        .build();
    let mut hold_value_bytes = [0u8; 32];
    hold_value_bytes[0..22].copy_from_slice(hold_value.as_slice());
    let value = H256::from(hold_value_bytes);
    (hold_value, value)
}

pub fn generate_withdrawal_key(cota_id: [u8; 20], token_index: [u8; 4]) -> (CotaNFTId, H256) {
    let withdrawal_key = CotaNFTIdBuilder::default()
        .cota_id(CotaId::from_slice(&cota_id).unwrap())
        .smt_type(Uint16::from_slice(&WITHDRAWAL_NFT_SMT_TYPE).unwrap())
        .index(Uint32::from_slice(&token_index).unwrap())
        .build();
    let mut withdrawal_key_bytes = [0u8; 32];
    withdrawal_key_bytes[0..26].copy_from_slice(withdrawal_key.as_slice());
    let key = H256::from(withdrawal_key_bytes);

    (withdrawal_key, key)
}

pub fn generate_withdrawal_value(
    configure: u8,
    state: u8,
    characteristic: [u8; 20],
    to_lock_script: Vec<u8>,
    out_point: [u8; 24],
) -> (WithdrawalCotaNFTValue, H256) {
    let cota_info = CotaNFTInfoBuilder::default()
        .configure(Byte::from(configure))
        .state(Byte::from(state))
        .characteristic(Characteristic::from_slice(&characteristic).unwrap())
        .build();
    let to_lock_bytes: Vec<Byte> = to_lock_script.iter().map(|v| Byte::from(*v)).collect();
    let withdrawal_value = WithdrawalCotaNFTValueBuilder::default()
        .nft_info(cota_info)
        .out_point(OutPointSlice::from_slice(&out_point).unwrap())
        .to_lock(BytesBuilder::default().set(to_lock_bytes).build())
        .build();
    let value = H256::from(blake2b_256(withdrawal_value.as_slice()));
    (withdrawal_value, value)
}

pub fn generate_claim_key(
    cota_id: [u8; 20],
    token_index: [u8; 4],
    out_point: [u8; 24],
) -> (ClaimCotaNFTKey, H256) {
    let nft_id = CotaNFTIdBuilder::default()
        .smt_type(Uint16::from_slice(&CLAIM_NFT_SMT_TYPE).unwrap())
        .cota_id(CotaId::from_slice(&cota_id).unwrap())
        .index(Uint32::from_slice(&token_index).unwrap())
        .build();
    let claimed_key = ClaimCotaNFTKeyBuilder::default()
        .nft_id(nft_id)
        .out_point(OutPointSlice::from_slice(&out_point).unwrap())
        .build();
    let key = H256::from(blake2b_256(claimed_key.as_slice()));
    (claimed_key, key)
}

pub fn generate_claim_value() -> (Byte32, H256) {
    let claim_value = Byte32Builder::default()
        .set([Byte::from(255u8); 32])
        .build();
    let value = H256::from([255u8; 32]);
    (claim_value, value)
}

pub fn generate_empty_value() -> (Byte32, H256) {
    let empty_value = Byte32Builder::default().set([Byte::from(0u8); 32]).build();
    let value = H256::from([0u8; 32]);
    (empty_value, value)
}

pub fn generate_history_smt(lock_hash: [u8; 32]) -> Result<SMT, Error> {
    let start_time = Local::now().timestamp_millis();
    let mut smt: SMT = SMT::default();
    let (defines, holds, withdrawals, claims) = get_all_cota_by_lock_hash(lock_hash)?;
    diff_time(start_time, "Load history smt leaves from database");

    let start_time = Local::now().timestamp_millis();
    for define_db in defines {
        let DefineDb {
            cota_id,
            total,
            issued,
            configure,
        } = define_db;
        let (_, key) = generate_define_key(cota_id);
        let (_, value) =
            generate_define_value(total.to_be_bytes(), issued.to_be_bytes(), configure);
        smt.update(key, value).expect("SMT update leave error");
    }
    for hold_db in holds {
        let HoldDb {
            cota_id,
            token_index,
            configure,
            state,
            characteristic,
        } = hold_db;
        let (_, key) = generate_hold_key(cota_id, token_index);
        let (_, value) = generate_hold_value(configure, state, characteristic);
        smt.update(key, value).expect("SMT update leave error");
    }
    for withdrawal_db in withdrawals {
        let WithdrawDb {
            cota_id,
            token_index,
            configure,
            state,
            characteristic,
            receiver_lock_script,
            out_point,
        } = withdrawal_db;
        let (_, key) = generate_withdrawal_key(cota_id, token_index);
        let (_, value) = generate_withdrawal_value(
            configure,
            state,
            characteristic,
            receiver_lock_script,
            out_point,
        );
        smt.update(key, value).expect("SMT update leave error");
    }
    for claim_db in claims {
        let ClaimDb {
            cota_id,
            token_index,
            out_point,
        } = claim_db;
        let (_, key) = generate_claim_key(cota_id, token_index, out_point);
        let (_, value) = generate_claim_value();
        smt.update(key, value).expect("SMT update leave error");
    }
    diff_time(start_time, "Push history leaves to smt");
    Ok(smt)
}
