//! Implement SMTStore trait

use crate::smt::db::cota_db::CotaRocksDB;
use crate::smt::db::schema::Col;
use crate::smt::store::serde::leaf_key_to_vec;
use crate::utils::helper::parse_vec_n;
use cota_smt::smt::H256;
use sparse_merkle_tree::{
    error::Error as SMTError,
    traits::Store,
    tree::{BranchKey, BranchNode},
};

use super::serde::{branch_key_to_vec, branch_node_to_vec, slice_to_branch_node};

pub struct SMTStore<'a> {
    lock_hash:  [u8; 32],
    leaf_col:   Col,
    branch_col: Col,
    root_col:   Col,
    store:      &'a CotaRocksDB,
}

impl<'a> SMTStore<'a> {
    pub fn new(
        lock_hash: [u8; 32],
        leaf_col: Col,
        branch_col: Col,
        root_col: Col,
        store: &'a CotaRocksDB,
    ) -> Self {
        SMTStore {
            lock_hash,
            leaf_col,
            branch_col,
            root_col,
            store,
        }
    }

    pub fn inner_store(&self) -> &CotaRocksDB {
        self.store
    }

    pub fn save_root(&self, root: H256) -> Result<(), SMTError> {
        self.store
            .insert_raw(self.root_col, &self.lock_hash, root.as_slice())
            .map_err(|err| SMTError::Store(format!("insert error {:?}", err)))?;
        Ok(())
    }

    pub fn get_root(&self) -> Result<Option<H256>, SMTError> {
        match self.store.get(self.root_col, &self.lock_hash) {
            Some(slice) => Ok(Some(H256::from(parse_vec_n(slice.to_vec())))),
            None => Ok(None),
        }
    }
}

impl<'a> Store<H256> for SMTStore<'a> {
    fn get_branch(&self, branch_key: &BranchKey) -> Result<Option<BranchNode>, SMTError> {
        match self.store.get(
            self.branch_col,
            &branch_key_to_vec(self.lock_hash, branch_key),
        ) {
            Some(slice) => Ok(Some(slice_to_branch_node(&slice))),
            None => Ok(None),
        }
    }

    fn get_leaf(&self, leaf_key: &H256) -> Result<Option<H256>, SMTError> {
        match self
            .store
            .get(self.leaf_col, &leaf_key_to_vec(self.lock_hash, leaf_key))
        {
            Some(slice) if 32 == slice.len() => {
                let mut leaf = [0u8; 32];
                leaf.copy_from_slice(slice.as_ref());
                Ok(Some(H256::from(leaf)))
            }
            Some(_) => Err(SMTError::Store("get corrupted leaf".to_string())),
            None => Ok(None),
        }
    }

    fn insert_branch(&mut self, branch_key: BranchKey, branch: BranchNode) -> Result<(), SMTError> {
        self.store
            .insert_raw(
                self.branch_col,
                &branch_key_to_vec(self.lock_hash, &branch_key),
                &branch_node_to_vec(&branch),
            )
            .map_err(|err| SMTError::Store(format!("insert error {:?}", err)))?;

        Ok(())
    }

    fn insert_leaf(&mut self, leaf_key: H256, leaf: H256) -> Result<(), SMTError> {
        self.store
            .insert_raw(
                self.leaf_col,
                &leaf_key_to_vec(self.lock_hash, &leaf_key),
                leaf.as_slice(),
            )
            .map_err(|err| SMTError::Store(format!("insert error {:?}", err)))?;

        Ok(())
    }

    fn remove_branch(&mut self, branch_key: &BranchKey) -> Result<(), SMTError> {
        self.store
            .delete(
                self.branch_col,
                &branch_key_to_vec(self.lock_hash, branch_key),
            )
            .map_err(|err| SMTError::Store(format!("delete error {:?}", err)))?;

        Ok(())
    }

    fn remove_leaf(&mut self, leaf_key: &H256) -> Result<(), SMTError> {
        self.store
            .delete(self.leaf_col, &leaf_key_to_vec(self.lock_hash, &leaf_key))
            .map_err(|err| SMTError::Store(format!("delete error {:?}", err)))?;

        Ok(())
    }
}
