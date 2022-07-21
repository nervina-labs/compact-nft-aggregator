use crate::utils::error::Error;
use crate::utils::helper::{get_secp256k1_batch_code_hash, parse_bytes, parse_vec_n, remove_0x};
use ckb_types::packed::Script;
use jsonrpc_http_server::jsonrpc_core::serde_json::Map;
use jsonrpc_http_server::jsonrpc_core::Value;
use molecule::prelude::Entity;

pub trait ReqParser: Sized {
    fn from_map(map: &Map<String, Value>) -> Result<Self, Error>;
}

pub fn parse_vec_map<T: ReqParser>(map: &Map<String, Value>, key: &str) -> Result<Vec<T>, Error> {
    let value = map
        .get(key)
        .ok_or(Error::RequestParamNotFound(key.to_owned()))?;
    if !value.is_array() {
        return Err(Error::RequestParamTypeError(key.to_owned()));
    }
    let mut vec: Vec<T> = Vec::new();
    for element in value.as_array().unwrap() {
        if !element.is_object() {
            return Err(Error::RequestParamTypeError(key.to_owned()));
        }
        vec.push(T::from_map(element.as_object().unwrap())?)
    }
    Ok(vec)
}

pub trait HexParser {
    fn get_hex_bytes_field<const N: usize>(&self, key: &str) -> Result<[u8; N], Error>;
    fn get_hex_vec_field(&self, key: &str) -> Result<Vec<u8>, Error>;
    fn get_i64_field(&self, key: &str) -> Result<i64, Error>;
    fn get_u8_field(&self, key: &str) -> Result<u8, Error>;
    fn get_script_field(&self, key: &str) -> Result<Vec<u8>, Error>;
}

impl HexParser for Map<String, Value> {
    fn get_hex_bytes_field<const N: usize>(&self, key: &str) -> Result<[u8; N], Error> {
        let v = self
            .get(key)
            .ok_or(Error::RequestParamNotFound(key.to_owned()))?;
        if !v.is_string() {
            return Err(Error::RequestParamHexInvalid(v.to_string()));
        }
        let hex_str = v.as_str().unwrap();
        if !hex_str.contains("0x") {
            return Err(Error::RequestParamHexInvalid(v.to_string()));
        }
        let hex_without_0x = remove_0x(hex_str);
        let result = hex::decode(hex_without_0x)
            .map_err(|_| Error::RequestParamHexInvalid(v.to_string()))?;
        if result.len() != N {
            return Err(Error::RequestParamHexLenError {
                msg:      key.to_owned(),
                got:      result.len(),
                expected: N,
            });
        }
        Ok(parse_vec_n(result))
    }

    fn get_hex_vec_field(&self, key: &str) -> Result<Vec<u8>, Error> {
        let v = self
            .get(key)
            .ok_or(Error::RequestParamNotFound(key.to_owned()))?;
        if !v.is_string() {
            return Err(Error::RequestParamTypeError(key.to_owned()));
        }
        let result = parse_bytes(v.as_str().unwrap().to_owned())?;
        Ok(result)
    }

    fn get_i64_field(&self, key: &str) -> Result<i64, Error> {
        let v = self
            .get(key)
            .ok_or(Error::RequestParamNotFound(key.to_owned()))?;
        if !v.is_string() {
            return Err(Error::RequestParamTypeError(key.to_owned()));
        }
        let result: i64 = v.as_str().unwrap().parse().unwrap();
        if result < 0 {
            return Err(Error::RequestParamTypeError(key.to_owned()));
        }
        Ok(result)
    }

    fn get_u8_field(&self, key: &str) -> Result<u8, Error> {
        let v = self
            .get(key)
            .ok_or(Error::RequestParamNotFound(key.to_owned()))?;
        if !v.is_string() {
            return Err(Error::RequestParamTypeError(key.to_owned()));
        }
        let result: u8 = v.as_str().unwrap().parse().unwrap();
        Ok(result)
    }

    fn get_script_field(&self, key: &str) -> Result<Vec<u8>, Error> {
        let script = self.get_hex_vec_field(key)?;
        if Script::from_slice(&script).is_err() {
            return Err(Error::RequestParamTypeError("Script".to_string()));
        }
        Ok(script)
    }
}

pub fn check_secp256k1_batch_master_lock(lock_script: &[u8]) -> Result<(), Error> {
    let script = Script::from_slice(lock_script).map_err(|_e| Error::CKBScriptError)?;
    let secp256k1_batch_code_hash = hex::decode(get_secp256k1_batch_code_hash()).unwrap();
    if script.code_hash().as_slice() == &secp256k1_batch_code_hash && script.args().len() != 20 {
        return Err(Error::Secp256k1BatchLockArgsError);
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use jsonrpc_http_server::jsonrpc_core::Value;

    #[test]
    fn test_get_hex_bytes_field() {
        let mut map = Map::new();
        map.insert(
            "lock_hash".to_owned(),
            Value::String(
                "0x1c5a6f36e6f1485e4df40906f22247888545dd00590a22d985d3be1f63b62db1".to_owned(),
            ),
        );
        map.insert(
            "cota_id".to_owned(),
            Value::String("f14aca18aae9df753af304469d8f4ebbc174a938".to_owned()),
        );
        map.insert("total".to_owned(), Value::String("0x0000008g".to_owned()));
        map.insert("page".to_owned(), Value::String("32".to_owned()));

        assert_eq!(
            map.get_hex_vec_field("lock_hash").unwrap(),
            hex::decode("1c5a6f36e6f1485e4df40906f22247888545dd00590a22d985d3be1f63b62db1")
                .unwrap()
        );

        assert_eq!(
            map.get_hex_bytes_field::<30>("lock_hash"),
            Err(Error::RequestParamHexLenError {
                msg:      "lock_hash".to_owned(),
                got:      32,
                expected: 30,
            })
        );

        assert_eq!(
            map.get_hex_bytes_field::<32>("lock_has"),
            Err(Error::RequestParamNotFound("lock_has".to_owned()))
        );

        assert_eq!(
            map.get_hex_bytes_field::<20>("cota_id"),
            Err(Error::RequestParamHexInvalid(
                "\"f14aca18aae9df753af304469d8f4ebbc174a938\"".to_owned()
            ))
        );

        assert_eq!(
            map.get_hex_bytes_field::<4>("total"),
            Err(Error::RequestParamHexInvalid("\"0x0000008g\"".to_owned()))
        );

        assert_eq!(map.get_i64_field("page"), Ok(32));
    }

    // TODO: Add more tests
}
