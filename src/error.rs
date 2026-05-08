//! Types for errors received from the API and occurring in the clients.
use thiserror::Error;

/// An error that can occur while working with Databento's API.
#[derive(Debug, Error)]
#[non_exhaustive]
pub enum Error {
    /// An invalid argument was passed to a function.
    #[error("bad argument `{param_name}`: {desc}")]
    BadArgument {
        /// The name of the parameter to which the bad argument was passed.
        param_name: &'static str,
        /// The description of why the argument was invalid.
        desc: String,
    },
    /// An I/O error while reading or writing DBN or another encoding.
    #[error("I/O error: {0:?}")]
    Io(#[from] std::io::Error),
    /// An HTTP error.
    #[cfg(feature = "historical")]
    #[error("HTTP error: {0:?}")]
    Http(#[from] reqwest::Error),
    /// An error related to JSON decoding.
    #[cfg(feature = "historical")]
    #[error("JSON error: {0:?}")]
    Json(#[from] serde_json::Error),
    /// An error related to JSON decoding.
    #[cfg(feature = "historical")]
    #[error("invalid UTF-8: {0:?}")]
    Utf8(#[from] std::str::Utf8Error),
    /// An error from the Databento API.
    #[cfg(feature = "historical")]
    #[error("API error: {0}")]
    Api(ApiError),
    /// An error internal to the client.
    #[error("internal error: {0}")]
    Internal(String),
    /// An error related to DBN encoding or decoding.
    #[error("DBN error: {0}")]
    Dbn(#[source] dbn::Error),
    /// An error when authentication failed.
    #[error("authentication failed: {0}")]
    Auth(String),
    /// A heartbeat timeout, i.e. no data received within the expected interval.
    #[cfg(feature = "live")]
    #[error("heartbeat timeout: no data received for {0:?}")]
    HeartbeatTimeout(time::Duration),
    /// The TCP connection to the gateway timed out.
    #[cfg(feature = "live")]
    #[error("connect timeout: failed to connect within {0:?}")]
    ConnectTimeout(time::Duration),
    /// Authentication with the gateway timed out.
    #[cfg(feature = "live")]
    #[error("auth timeout: authentication did not complete within {0:?}")]
    AuthTimeout(time::Duration),
}
/// An alias for a `Result` with [`databento::Error`](crate::Error) as the error type.
pub type Result<T> = std::result::Result<T, Error>;

/// An error from the Databento API.
#[cfg(feature = "historical")]
#[derive(Debug)]
pub struct ApiError {
    /// The request ID.
    pub request_id: Option<String>,
    /// The HTTP status code of the response.
    pub status_code: reqwest::StatusCode,
    /// A machine-readable identifier for the error case, when the server returns a
    /// structured error envelope. `None` for unstructured errors.
    pub case: Option<String>,
    /// The message from the Databento API.
    pub message: String,
    /// The link to documentation related to the error.
    pub docs_url: Option<String>,
    /// Additional context for the error, when the server provides one. Common keys
    /// include `dataset`, `start`, `end`, `available_start`, and `available_end`.
    // Boxed to keep `Error::Api` small on the stack on the common no-payload path.
    pub payload: Option<Box<std::collections::HashMap<String, serde_json::Value>>>,
}

impl Error {
    pub(crate) fn bad_arg(param_name: &'static str, desc: impl ToString) -> Self {
        Self::BadArgument {
            param_name,
            desc: desc.to_string(),
        }
    }

    pub(crate) fn internal(msg: impl ToString) -> Self {
        Self::Internal(msg.to_string())
    }
}

impl From<dbn::Error> for Error {
    fn from(dbn_err: dbn::Error) -> Self {
        match dbn_err {
            // Convert to our own error type.
            dbn::Error::Io { source, .. } => Self::Io(source),
            dbn_err => Self::Dbn(dbn_err),
        }
    }
}

#[cfg(feature = "historical")]
impl std::fmt::Display for ApiError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let doc = self
            .docs_url
            .as_ref()
            .map(|d| format!(" See {d} for documentation."))
            .unwrap_or_default();
        let case = self
            .case
            .as_ref()
            .map(|c| format!(" (case: {c})"))
            .unwrap_or_default();
        let status = self.status_code;
        let msg = &self.message;
        if let Some(ref request_id) = self.request_id {
            write!(f, "{request_id} failed with {status} {msg}{doc}{case}")
        } else {
            write!(f, "{status} {msg}{doc}{case}")
        }
    }
}
