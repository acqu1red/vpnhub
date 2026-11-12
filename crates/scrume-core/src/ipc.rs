//! Parsing only. A successful parse is never proof of caller authorization.
use serde::Deserialize;
use thiserror::Error;

pub const MAX_FRAME: usize = 8192;

#[derive(Debug, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct Request {
    pub protocol_version: u8,
    pub request_id: u64,
    pub method: Command,
    #[serde(default)]
    pub params: EmptyParams,
}

#[derive(Debug, Default, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct EmptyParams {}

#[derive(Debug, Deserialize, PartialEq, Eq)]
pub enum Command {
    #[serde(rename = "status.get")]
    Status,
    #[serde(rename = "vpn.connect")]
    Connect,
    #[serde(rename = "vpn.disconnect")]
    Disconnect,
}

#[derive(Debug, Error, PartialEq, Eq)]
pub enum ProtocolError {
    #[error("IPC_TOO_LARGE")]
    TooLarge,
    #[error("IPC_SCHEMA")]
    Schema,
    #[error("IPC_VERSION")]
    Version,
}

pub fn parse(frame: &[u8]) -> Result<Request, ProtocolError> {
    if frame.len() > MAX_FRAME {
        return Err(ProtocolError::TooLarge);
    }
    let request: Request = serde_json::from_slice(frame).map_err(|_| ProtocolError::Schema)?;
    if request.protocol_version != 1 {
        return Err(ProtocolError::Version);
    }
    Ok(request)
}
