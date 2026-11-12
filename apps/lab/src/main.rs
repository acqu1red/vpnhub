use base64::{Engine, engine::general_purpose::URL_SAFE_NO_PAD};
use ed25519_dalek::VerifyingKey;
use scrume_core::bundle::{VerifyContext, VersionFloor, verify};
use serde::Deserialize;
use std::{collections::BTreeMap, process::ExitCode};
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
    envelope: serde_json::Value,
}

fn run() -> Result<(), Box<dyn std::error::Error>> {
    if std::env::args().skip(1).collect::<Vec<_>>() != ["verify-fixture"] {
        return Err("usage: scrume-lab verify-fixture (offline, public fixture only)".into());
    }
    let fixture: Fixture =
        serde_json::from_str(include_str!("../../../tests/fixtures/signing-v1.json"))?;
    let bytes: [u8; 32] = URL_SAFE_NO_PAD
        .decode(fixture.public_key)?
        .try_into()
        .map_err(|_| "invalid fixture key")?;
    let keys = BTreeMap::from([(fixture.key_id, VerifyingKey::from_bytes(&bytes)?)]);
    let ctx = VerifyContext {
        enrollment_id: fixture.enrollment_id,
        now: fixture.now,
        trusted_time_floor: fixture.now,
        version_floor: VersionFloor::default(),
        keyring: &keys,
    };
    let case = fixture
        .cases
        .iter()
        .find(|c| c.name == "valid")
        .ok_or("missing valid vector")?;
    let verified = verify(&serde_json::to_vec(&case.envelope)?, &ctx)?;
    println!(
        "VERIFIED public fixture: version={}, signature+binding+lease+schema; no network effects",
        verified.bundle().version
    );
    Ok(())
}

fn main() -> ExitCode {
    match run() {
        Ok(()) => ExitCode::SUCCESS,
        Err(_) => {
            eprintln!("LAB_FAILED: invalid command or public test fixture");
            ExitCode::FAILURE
        }
    }
}
