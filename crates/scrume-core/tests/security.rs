use base64::{Engine, engine::general_purpose::URL_SAFE_NO_PAD};
use ed25519_dalek::VerifyingKey;
use scrume_core::{
    bundle::{MAX_ENVELOPE, VerifiedBundle, VerifyContext, VerifyError, VersionFloor, verify},
    ipc::{Command, MAX_FRAME, ProtocolError, parse},
    state::{Effect, Machine, State, StateError},
};
use serde::Deserialize;
use serde_json::{Value, json};
use std::collections::BTreeMap;
use uuid::Uuid;

#[derive(Deserialize)]
struct Fixture {
    key_id: String,
    public_key: String,
    enrollment_id: Uuid,
    now: u64,
    cases: Vec<Case>,
}
#[derive(Deserialize)]
struct Case {
    name: String,
    expected: String,
    envelope: Value,
}

fn fixture() -> Result<Fixture, Box<dyn std::error::Error>> {
    Ok(serde_json::from_str(include_str!(
        "../../../tests/fixtures/signing-v1.json"
    ))?)
}
fn keys(f: &Fixture) -> Result<BTreeMap<String, VerifyingKey>, Box<dyn std::error::Error>> {
    let bytes: [u8; 32] = URL_SAFE_NO_PAD
        .decode(&f.public_key)?
        .try_into()
        .map_err(|_| "key size")?;
    Ok(BTreeMap::from([(
        f.key_id.clone(),
        VerifyingKey::from_bytes(&bytes)?,
    )]))
}
fn context<'a>(f: &Fixture, keys: &'a BTreeMap<String, VerifyingKey>) -> VerifyContext<'a> {
    VerifyContext {
        enrollment_id: f.enrollment_id,
        now: f.now,
        trusted_time_floor: f.now,
        version_floor: VersionFloor::default(),
        keyring: keys,
    }
}
fn good(f: &Fixture) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
    let c = f
        .cases
        .iter()
        .find(|c| c.name == "valid")
        .ok_or("valid fixture missing")?;
    Ok(serde_json::to_vec(&c.envelope)?)
}
fn verified(f: &Fixture) -> Result<VerifiedBundle, Box<dyn std::error::Error>> {
    Ok(verify(&good(f)?, &context(f, &keys(f)?))?)
}

#[test]
fn independent_signed_vectors() -> Result<(), Box<dyn std::error::Error>> {
    let f = fixture()?;
    let keys = keys(&f)?;
    let ctx = context(&f, &keys);
    for case in &f.cases {
        let result = verify(&serde_json::to_vec(&case.envelope)?, &ctx);
        if case.expected == "ok" {
            assert!(result.is_ok(), "{}", case.name);
        } else {
            assert_eq!(
                result.err().ok_or("expected rejection")?.to_string(),
                case.expected,
                "{}",
                case.name
            );
        }
    }
    Ok(())
}

#[test]
fn same_version_requires_same_payload() -> Result<(), Box<dyn std::error::Error>> {
    let f = fixture()?;
    let keys = keys(&f)?;
    let mut ctx = context(&f, &keys);
    let data = good(&f)?;
    let first = verify(&data, &ctx)?;
    ctx.version_floor = first.version_floor();
    assert!(verify(&data, &ctx).is_ok());
    ctx.version_floor.payload_sha256 = [0; 32];
    assert_eq!(
        verify(&data, &ctx).err(),
        Some(VerifyError::VersionConflict)
    );
    Ok(())
}

#[test]
fn older_version_rejected() -> Result<(), Box<dyn std::error::Error>> {
    let f = fixture()?;
    let keys = keys(&f)?;
    let mut ctx = context(&f, &keys);
    ctx.version_floor.version = 8;
    assert_eq!(verify(&good(&f)?, &ctx).err(), Some(VerifyError::Rollback));
    Ok(())
}

#[test]
fn clock_rollback_rejected() -> Result<(), Box<dyn std::error::Error>> {
    let f = fixture()?;
    let keys = keys(&f)?;
    let mut ctx = context(&f, &keys);
    ctx.trusted_time_floor = f.now + 121;
    assert_eq!(verify(&good(&f)?, &ctx).err(), Some(VerifyError::Clock));
    Ok(())
}

#[test]
fn expired_cached_envelope_cannot_reconnect() -> Result<(), Box<dyn std::error::Error>> {
    let f = fixture()?;
    let keys = keys(&f)?;
    let mut ctx = context(&f, &keys);
    ctx.now = f.now + 3600;
    assert_eq!(verify(&good(&f)?, &ctx).err(), Some(VerifyError::Lease));
    Ok(())
}

#[test]
fn unknown_key_and_oversized_envelope_rejected() -> Result<(), Box<dyn std::error::Error>> {
    let f = fixture()?;
    let empty = BTreeMap::new();
    assert_eq!(
        verify(&good(&f)?, &context(&f, &empty)).err(),
        Some(VerifyError::UnknownKey)
    );
    assert_eq!(
        verify(&vec![b' '; MAX_ENVELOPE + 1], &context(&f, &empty)).err(),
        Some(VerifyError::TooLarge)
    );
    Ok(())
}

#[test]
fn outer_unsigned_expiry_rejected() -> Result<(), Box<dyn std::error::Error>> {
    let f = fixture()?;
    let keys = keys(&f)?;
    let mut data: Value = serde_json::from_slice(&good(&f)?)?;
    data["expires_at"] = json!(f.now + 999999);
    assert_eq!(
        verify(&serde_json::to_vec(&data)?, &context(&f, &keys)).err(),
        Some(VerifyError::Schema)
    );
    Ok(())
}

#[test]
fn duplicate_outer_field_rejected() -> Result<(), Box<dyn std::error::Error>> {
    let f = fixture()?;
    let keys = keys(&f)?;
    let raw = String::from_utf8(good(&f)?)?;
    let data = raw.replacen('{', "{\"key_id\":\"fixture-only-1\",", 1);
    assert_eq!(
        verify(data.as_bytes(), &context(&f, &keys)).err(),
        Some(VerifyError::Schema)
    );
    Ok(())
}

#[test]
fn redacted_debug_omits_profile_credentials() -> Result<(), Box<dyn std::error::Error>> {
    let f = fixture()?;
    let b = verified(&f)?;
    let debug = format!("{b:?}");
    assert!(!debug.contains(&b.bundle().profile.uuid.to_string()));
    assert!(!debug.contains(&b.bundle().profile.public_key));
    assert!(!debug.contains(&b.bundle().profile.server));
    Ok(())
}

#[test]
fn ipc_allowlist_and_no_user_sid() -> Result<(), Box<dyn std::error::Error>> {
    let request =
        parse(br#"{"protocol_version":1,"request_id":1,"method":"status.get","params":{}}"#)?;
    assert_eq!(request.method, Command::Status);
    for method in [
        "exec",
        "vpn.healthy",
        "service.install",
        "run",
        "profile.apply",
    ] {
        let raw =
            serde_json::to_vec(&json!({"protocol_version":1,"request_id":1,"method":method}))?;
        assert_eq!(parse(&raw).err(), Some(ProtocolError::Schema));
    }
    assert_eq!(
        parse(br#"{"protocol_version":1,"request_id":1,"method":"status.get","sid":"S-1-5-18"}"#)
            .err(),
        Some(ProtocolError::Schema)
    );
    Ok(())
}

#[test]
fn ipc_rejects_bad_params_types_duplicates_and_sizes() -> Result<(), Box<dyn std::error::Error>> {
    for raw in [
        r#"{"protocol_version":1,"request_id":1,"method":"vpn.connect","params":{"path":"evil"}}"#,
        r#"{"protocol_version":1,"request_id":1,"method":"vpn.connect","params":null}"#,
        r#"{"protocol_version":1,"request_id":1,"method":"vpn.connect","method":"status.get"}"#,
        r#"{"protocol_version":1,"request_id":"1","method":"status.get"}"#,
        r#"{"protocol_version":1,"request_id":1,"method":"status.get"} {}"#,
    ] {
        assert_eq!(parse(raw.as_bytes()).err(), Some(ProtocolError::Schema));
    }
    assert_eq!(
        parse(&vec![b' '; MAX_FRAME + 1]).err(),
        Some(ProtocolError::TooLarge)
    );
    assert_eq!(
        parse(br#"{"protocol_version":2,"request_id":1,"method":"status.get"}"#).err(),
        Some(ProtocolError::Version)
    );
    Ok(())
}

#[test]
fn no_boot_before_initial_enrollment() -> Result<(), Box<dyn std::error::Error>> {
    let mut m = Machine::default();
    assert_eq!(m.boot(fixture()?.now)?, Effect::None);
    assert_eq!(m.state(), State::Unenrolled);
    Ok(())
}

#[test]
fn signed_policy_can_disable_boot_connect() -> Result<(), Box<dyn std::error::Error>> {
    let f = fixture()?;
    let keys = keys(&f)?;
    let case = f
        .cases
        .iter()
        .find(|c| c.name == "auto_connect_disabled")
        .ok_or("missing policy vector")?;
    let bundle = verify(&serde_json::to_vec(&case.envelope)?, &context(&f, &keys))?;
    let mut m = Machine::default();
    m.enroll(bundle, f.now)?;
    assert_eq!(m.boot(f.now)?, Effect::None);
    assert_eq!(m.state(), State::Ready);
    assert!(matches!(m.connect(f.now)?, Effect::StartTunnel { .. }));
    Ok(())
}

#[test]
fn valid_signature_cannot_override_same_version_payload() -> Result<(), Box<dyn std::error::Error>>
{
    let f = fixture()?;
    let keys = keys(&f)?;
    let mut ctx = context(&f, &keys);
    ctx.version_floor = verified(&f)?.version_floor();
    let case = f
        .cases
        .iter()
        .find(|c| c.name == "same_version_different_valid")
        .ok_or("missing vector")?;
    assert_eq!(
        verify(&serde_json::to_vec(&case.envelope)?, &ctx).err(),
        Some(VerifyError::VersionConflict)
    );
    Ok(())
}

#[test]
fn boot_connect_does_not_claim_connected() -> Result<(), Box<dyn std::error::Error>> {
    let f = fixture()?;
    let mut m = Machine::default();
    m.enroll(verified(&f)?, f.now)?;
    assert_eq!(m.boot(f.now)?, Effect::StartTunnel { generation: 1 });
    assert_eq!(m.state(), State::Connecting);
    assert_eq!(m.connect(f.now)?, Effect::None);
    m.healthy(1, f.now)?;
    assert_eq!(m.state(), State::Connected);
    Ok(())
}

#[test]
fn disconnect_invalidates_late_health() -> Result<(), Box<dyn std::error::Error>> {
    let f = fixture()?;
    let mut m = Machine::default();
    m.enroll(verified(&f)?, f.now)?;
    m.connect(f.now)?;
    assert_eq!(m.disconnect(), Effect::StopAndRecover);
    assert_eq!(m.healthy(1, f.now), Err(StateError::Stale));
    m.recovered()?;
    assert_eq!(m.state(), State::Ready);
    Ok(())
}

#[test]
fn active_expiry_requires_cleanup_before_terminal_state() -> Result<(), Box<dyn std::error::Error>>
{
    let f = fixture()?;
    let mut m = Machine::default();
    m.enroll(verified(&f)?, f.now)?;
    m.connect(f.now)?;
    m.healthy(1, f.now)?;
    assert_eq!(m.tick(f.now + 3600), Effect::StopAndRecover);
    assert_eq!(m.state(), State::Recovering);
    m.recovered()?;
    assert_eq!(m.state(), State::Expired);
    assert!(m.connect(f.now + 3600).is_err());
    Ok(())
}

#[test]
fn revoked_pending_connect_never_becomes_connected() -> Result<(), Box<dyn std::error::Error>> {
    let f = fixture()?;
    let mut m = Machine::default();
    m.enroll(verified(&f)?, f.now)?;
    m.connect(f.now)?;
    assert_eq!(m.revoke(), Effect::StopAndRecover);
    assert_eq!(m.healthy(1, f.now), Err(StateError::Stale));
    m.recovered()?;
    assert_eq!(m.state(), State::Revoked);
    assert!(m.connect(f.now).is_err());
    Ok(())
}

#[test]
fn crash_requires_recovery_not_immediate_success() -> Result<(), Box<dyn std::error::Error>> {
    let f = fixture()?;
    let mut m = Machine::default();
    m.enroll(verified(&f)?, f.now)?;
    m.connect(f.now)?;
    m.healthy(1, f.now)?;
    assert_eq!(m.child_crashed(), Effect::StopAndRecover);
    assert_eq!(m.state(), State::Recovering);
    assert!(m.connect(f.now).is_err());
    m.recovered()?;
    assert_eq!(m.connect(f.now)?, Effect::StartTunnel { generation: 3 });
    Ok(())
}

#[test]
fn live_bundle_replacement_is_not_silent() -> Result<(), Box<dyn std::error::Error>> {
    let f = fixture()?;
    let mut m = Machine::default();
    m.enroll(verified(&f)?, f.now)?;
    m.connect(f.now)?;
    assert_eq!(m.enroll(verified(&f)?, f.now), Err(StateError::NotAllowed));
    Ok(())
}
