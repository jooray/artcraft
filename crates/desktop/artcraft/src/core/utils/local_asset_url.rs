use std::path::Path;

/// Build the URL that the webview uses to load a local file (the Tauri asset
/// protocol).
///
/// Mirrors `convertFileSrc` from `@tauri-apps/api`: the whole path is
/// percent-encoded like `encodeURIComponent`, so spaces and unicode work.
pub fn local_asset_url_for_path(path: &Path) -> Option<String> {
  let path_str = path.to_str()?;
  let encoded = encode_uri_component(path_str);

  Some(if cfg!(target_os = "windows") || cfg!(target_os = "android") {
    format!("http://asset.localhost/{}", encoded)
  } else {
    format!("asset://localhost/{}", encoded)
  })
}

/// `encodeURIComponent` for UTF-8: unreserved characters are kept literal,
/// every other byte becomes `%XX`.
fn encode_uri_component(value: &str) -> String {
  const UNRESERVED: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789-_.!~*'()";

  let mut encoded = String::with_capacity(value.len());
  for byte in value.bytes() {
    if UNRESERVED.contains(&byte) {
      encoded.push(byte as char);
    } else {
      encoded.push_str(&format!("%{:02X}", byte));
    }
  }
  encoded
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn encodes_like_encode_uri_component() {
    assert_eq!(encode_uri_component("/Users/ju raj/Artcraft/foto.webp"), "%2FUsers%2Fju%20raj%2FArtcraft%2Ffoto.webp");
    assert_eq!(encode_uri_component("a-b_c.d~e!f*g'h(i)"), "a-b_c.d~e!f*g'h(i)");
  }

  #[test]
  fn builds_a_tolerant_asset_url() {
    let url = local_asset_url_for_path(Path::new("/tmp/some file.webp")).unwrap();
    if cfg!(target_os = "windows") {
      assert_eq!(url, "http://asset.localhost/%2Ftmp%2Fsome%20file.webp");
    } else {
      assert_eq!(url, "asset://localhost/%2Ftmp%2Fsome%20file.webp");
    }
  }

  #[test]
  fn tolerates_non_utf8_paths() {
    #[cfg(unix)]
    {
      use std::ffi::OsStr;
      use std::os::unix::ffi::OsStrExt;
      let path = Path::new(OsStr::from_bytes(b"/tmp/\xff\xfe"));
      assert!(local_asset_url_for_path(path).is_none());
    }
  }
}
