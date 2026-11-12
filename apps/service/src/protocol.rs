use scrume_core::{
    authorization::{Peer, authorize},
    ipc::{self, Command},
};
use serde_json::{Value, json};

fn error(request_id: Option<u64>, code: &str) -> Value {
    json!({"protocol_version":1,"request_id":request_id,"error":{"code":code}})
}

/// The transport supplies Peer after a successful OS token query. Owner is loaded
/// from protected service state in a later enrollment adapter, never from JSON.
pub fn response(peer: &Peer, owner_sid: Option<&str>, frame: &[u8]) -> Value {
    let request = match ipc::parse(frame) {
        Ok(request) => request,
        Err(e) => return error(None, &e.to_string()),
    };
    if authorize(peer, owner_sid, &request.method).is_err() {
        return error(Some(request.request_id), "IPC_ACCESS_DENIED");
    }
    match request.method {
        Command::Status => json!({"protocol_version":1,"request_id":request.request_id,"result":{
            "state":"UNENROLLED","tunnel_available":false,"service_version":env!("CARGO_PKG_VERSION")
        }}),
        Command::Connect | Command::Disconnect => {
            error(Some(request.request_id), "TUNNEL_NOT_IMPLEMENTED")
        }
    }
}

pub fn handle(peer: &Peer, frame: &[u8]) -> Vec<u8> {
    // Value constructed only from trusted constants + numeric request ID.
    // Even a serialization failure returns a static redacted error, not raw data.
    serde_json::to_vec(&response(peer, None, frame)).unwrap_or_else(|_| {
        br#"{"protocol_version":1,"request_id":null,"error":{"code":"IPC_INTERNAL"}}"#.to_vec()
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    fn peer() -> Peer {
        Peer {
            sid: "S-1-5-21-1-2-3-1001".into(),
            token_session: 1,
            transport_session: 1,
            active_console_session: 1,
        }
    }
    #[test]
    fn sanitized_status_does_not_claim_tunnel_or_expose_identity() {
        let p = peer();
        let r = response(
            &p,
            None,
            br#"{"protocol_version":1,"request_id":42,"method":"status.get"}"#,
        );
        assert_eq!(r["request_id"], 42);
        assert_eq!(r["result"]["state"], "UNENROLLED");
        assert_eq!(r["result"]["tunnel_available"], false);
        assert!(!r.to_string().contains(&p.sid));
    }
    #[test]
    fn missing_owner_denies_mutation_and_owner_never_gets_fake_success() {
        let p = peer();
        let f = br#"{"protocol_version":1,"request_id":1,"method":"vpn.connect"}"#;
        assert_eq!(response(&p, None, f)["error"]["code"], "IPC_ACCESS_DENIED");
        assert_eq!(
            response(&p, Some(&p.sid), f)["error"]["code"],
            "TUNNEL_NOT_IMPLEMENTED"
        );
    }
    #[test]
    fn malformed_and_spoofed_input_not_reflected() {
        let r = response(&peer(),None,br#"{"protocol_version":1,"request_id":1,"method":"status.get","sid":"S-1-5-18","secret":"do-not-echo"}"#);
        assert_eq!(r["error"]["code"], "IPC_SCHEMA");
        assert!(!r.to_string().contains("do-not-echo"));
    }
    #[test]
    fn unauthorized_session_cannot_read_status() {
        let mut p = peer();
        p.token_session = 0;
        assert_eq!(
            response(
                &p,
                None,
                br#"{"protocol_version":1,"request_id":1,"method":"status.get"}"#
            )["error"]["code"],
            "IPC_ACCESS_DENIED"
        );
    }
}
