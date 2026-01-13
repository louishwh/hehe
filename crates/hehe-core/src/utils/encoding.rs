use base64::{engine::general_purpose::STANDARD as BASE64, Engine};
use bytes::Bytes;

pub fn encode_base64(data: &[u8]) -> String {
    BASE64.encode(data)
}

pub fn decode_base64(data: &str) -> Result<Bytes, base64::DecodeError> {
    BASE64.decode(data).map(Bytes::from)
}

pub fn encode_base64_url_safe(data: &[u8]) -> String {
    base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(data)
}

pub fn decode_base64_url_safe(data: &str) -> Result<Bytes, base64::DecodeError> {
    base64::engine::general_purpose::URL_SAFE_NO_PAD
        .decode(data)
        .map(Bytes::from)
}
