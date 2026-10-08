use base64::engine::general_purpose::STANDARD;
use base64::Engine;

use crate::error::venice_error::VeniceError;

/// Decode one image from an `/image/generate` response into raw bytes.
///
/// Venice documents plain base64, but a `data:` URI prefix is tolerated in case
/// the response format changes.
pub fn decode_base64_image(image_data: &str) -> Result<Vec<u8>, VeniceError> {
  let raw = match image_data.strip_prefix("data:") {
    Some(rest) => rest.split_once(',').map(|(_, data)| data).unwrap_or(rest),
    None => image_data,
  };
  Ok(STANDARD.decode(raw)?)
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn decodes_plain_base64() {
    assert_eq!(decode_base64_image("aGVsbG8=").unwrap(), b"hello");
  }

  #[test]
  fn decodes_data_uri_base64() {
    assert_eq!(decode_base64_image("data:image/webp;base64,aGVsbG8=").unwrap(), b"hello");
  }

  #[test]
  fn rejects_invalid_base64() {
    assert!(decode_base64_image("!!!not-base64!!!").is_err());
  }
}
