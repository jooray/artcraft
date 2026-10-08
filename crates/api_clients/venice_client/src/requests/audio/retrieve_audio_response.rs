use serde::Deserialize;

/// Raw JSON status body returned while audio is still generating.
#[derive(Clone, Debug, Deserialize)]
pub struct VeniceAudioRetrieveStatus {
  /// `PROCESSING`
  pub status: String,

  #[serde(default)]
  pub average_execution_time: Option<f64>,

  #[serde(default)]
  pub execution_duration: Option<f64>,
}

/// What `/audio/retrieve` returned.
#[derive(Clone, Debug)]
pub enum VeniceAudioRetrieveResponse {
  Processing {
    status: String,
    maybe_average_execution_time: Option<f64>,
    maybe_execution_duration: Option<f64>,
  },

  /// The finished audio file (`audio/mpeg`, `audio/wav`, `audio/flac`).
  Audio {
    bytes: Vec<u8>,
    content_type: String,
  },
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn parses_a_processing_status() {
    let status: VeniceAudioRetrieveStatus = serde_json::from_str(
      r#"{"status":"PROCESSING","average_execution_time":20000,"execution_duration":5200}"#,
    )
    .unwrap();
    assert_eq!(status.status, "PROCESSING");
    assert_eq!(status.average_execution_time, Some(20000.0));
  }
}
