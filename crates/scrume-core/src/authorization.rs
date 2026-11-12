//! Authorization facts come from the transport, never request JSON.
use crate::ipc::Command;
use thiserror::Error;

#[derive(Debug)]
pub struct Peer {
    pub sid: String,
    pub token_session: u32,
    pub transport_session: u32,
    pub active_console_session: u32,
}

#[derive(Debug, Error, PartialEq, Eq)]
#[error("IPC_ACCESS_DENIED")]
pub struct AccessDenied;

/// Pilot scope: active console users can read sanitized service status.
/// Mutation additionally requires the durable enrolled owner, not merely IU ACL.
pub fn authorize(
    peer: &Peer,
    owner_sid: Option<&str>,
    command: &Command,
) -> Result<(), AccessDenied> {
    if !peer.sid.starts_with("S-1-")
        || matches!(
            peer.sid.as_str(),
            "S-1-5-18" | "S-1-5-19" | "S-1-5-20" | "S-1-5-7"
        )
        || peer.token_session == 0
        || peer.active_console_session == u32::MAX
        || peer.token_session != peer.transport_session
        || peer.token_session != peer.active_console_session
    {
        return Err(AccessDenied);
    }
    if !matches!(command, Command::Status) && owner_sid != Some(peer.sid.as_str()) {
        return Err(AccessDenied);
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    fn user() -> Peer {
        Peer {
            sid: "S-1-5-21-1-2-3-1001".into(),
            token_session: 1,
            transport_session: 1,
            active_console_session: 1,
        }
    }
    #[test]
    fn active_console_status_and_owner_commands() {
        let p = user();
        assert!(authorize(&p, None, &Command::Status).is_ok());
        assert!(authorize(&p, None, &Command::Connect).is_err());
        assert!(authorize(&p, Some(&p.sid), &Command::Connect).is_ok());
        assert!(authorize(&p, Some("S-1-5-21-1-2-3-1002"), &Command::Disconnect).is_err());
    }
    #[test]
    fn transport_and_token_sessions_must_match_active_console() {
        let mut p = user();
        p.transport_session = 2;
        assert!(authorize(&p, Some(&p.sid), &Command::Status).is_err());
        p.transport_session = 1;
        p.active_console_session = 2;
        assert!(authorize(&p, None, &Command::Status).is_err());
        p.active_console_session = u32::MAX;
        assert!(authorize(&p, None, &Command::Status).is_err());
    }
    #[test]
    fn anonymous_system_and_session_zero_are_rejected() {
        for sid in ["S-1-5-7", "S-1-5-18", "S-1-5-19", "S-1-5-20", "forged"] {
            let mut p = user();
            p.sid = sid.into();
            assert!(authorize(&p, None, &Command::Status).is_err());
        }
        let mut p = user();
        p.token_session = 0;
        p.transport_session = 0;
        p.active_console_session = 0;
        assert!(authorize(&p, None, &Command::Status).is_err());
    }
}
