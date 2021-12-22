use crate::error::Error;
use jsonrpc_http_server::jsonrpc_core::serde_json::Map;
use jsonrpc_http_server::jsonrpc_core::Value;

fn remove_0x(str: &str) -> &str {
    if str.contains("0x") {
        &str[2..]
    } else {
        str
    }
}

pub trait HexParser {
    fn get_hex_bytes_filed(&self, key: &str, bytes_count: usize) -> Result<Vec<u8>, Error>;
}

impl HexParser for Map<String, Value> {
    fn get_hex_bytes_filed(&self, key: &str, bytes_count: usize) -> Result<Vec<u8>, Error> {
        let v = self
            .get(key)
            .ok_or(Error::RequestParamNotFound("lock_hash".to_owned()))?;
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
        if result.len() != bytes_count {
            return Err(Error::RequestParamHexLenError {
                got:      result.len(),
                expected: bytes_count,
            });
        }
        Ok(result)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use jsonrpc_http_server::jsonrpc_core::Value;

    #[test]
    fn text_get_hex_bytes_filed() {
        let mut map = Map::new();
        map.insert(
            "lock_hash".to_owned(),
            Value::String(
                "0x1c5a6f36e6f1485e4df40906f22247888545dd00590a22d985d3be1f63b62db1".to_owned(),
            ),
        );
        map.insert(
            "cota_id".to_owned(),
            Value::String("0xf14aca18aae9df753af304469d8f4ebbc174a938".to_owned()),
        );

        assert_eq!(
            map.get_hex_bytes_filed("lock_hash", 32).unwrap(),
            hex::decode("1c5a6f36e6f1485e4df40906f22247888545dd00590a22d985d3be1f63b62db1")
                .unwrap()
        )
    }
}
