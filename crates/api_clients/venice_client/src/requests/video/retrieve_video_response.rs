use serde::Deserialize;

/// Raw JSON status body returned by `POST /video/retrieve` while the job is
/// still in progress (or when the media must be fetched from `download_url`).
#[derive(Clone, Debug, Deserialize)]
pub struct VeniceVideoRetrieveStatus {
  /// `PROCESSING` | `COMPLETED`
  pub status: String,

  #[serde(default)]
  pub average_execution_time: Option<f64>,

  #[serde(default)]
  pub execution_duration: Option<f64>,
}

/// What `/video/retrieve` returned.
#[derive(Clone, Debug)]
pub enum VeniceVideoRetrieveResponse {
  /// Still generating (JSON status).
  Processing {
    status: String,
    maybe_average_execution_time: Option<f64>,
    maybe_execution_duration: Option<f64>,
  },

  /// The job is complete but the video must be fetched from the queue's
  /// `download_url` (JSON status only; VPS-backed models).
  CompletedWithDownloadUrl,

  /// The endpoint returned the finished video file directly.
  Video { bytes: Vec<u8> },
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn parses_a_processing_status() {
    let status: VeniceVideoRetrieveStatus = serde_json::from_str(
      r#"{"status":"PROCESSING","average_execution_time":120.5,"execution_duration":30.0}"#,
    )
    .unwrap();
    assert_eq!(status.status, "PROCESSING");
    assert_eq!(status.execution_duration, Some(30.0));
  }
}
