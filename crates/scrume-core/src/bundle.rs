use base64::{Engine, engine::general_purpose::URL_SAFE_NO_PAD};
use ed25519_dalek::{Signature, VerifyingKey};
use ipnet::IpNet;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::{collections::BTreeMap, fmt, net::IpAddr};
use thiserror::Error;
use uuid::Uuid;

pub const CONTEXT: &str = "scrumevpn/bundle/v1";
pub const SIGNING_PREFIX: &[u8] = b"scrumevpn/bundle/v1\0";
pub const MAX_ENVELOPE: usize = 65536;
pub const MAX_PAYLOAD: usize = 32768;
const MAX_LEASE: u64 = 86400;
const CLOCK_SKEW: u64 = 120;

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Envelope {
    key_id: String,
    payload: String,
    signature: String,
}

#[derive(Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Bundle {
    pub schema_version: u32,
    pub context: String,
    pub key_id: String,
    pub enrollment_id: Uuid,
    pub version: u64,
    pub issued_at: u64,
    pub expires_at: u64,
    pub profile: Profile,
    pub policy: Policy,
}

// Deliberately no Debug on credential-bearing profile or envelope, and no Serialize
// on VerifiedBundle. Status DTOs must never serialize a raw bundle into WebView.
#[derive(Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Profile {
    pub server: String,
    pub port: u16,
    pub uuid: Uuid,
    pub sni: String,
    pub public_key: String,
    pub short_id: String,
    pub fingerprint: Fingerprint,
    pub mtu: u16,
    pub dns: Vec<IpAddr>,
    pub exclude_cidrs: Vec<IpNet>,
    pub exclude_domains: Vec<String>,
}

#[derive(Clone, Copy, Debug, Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Fingerprint {
    Chrome,
}

#[derive(Clone, Copy, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Policy {
    pub auto_connect: bool,
    pub kill_switch: bool,
    pub refresh_interval_seconds: u32,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct VersionFloor {
    pub version: u64,
    pub payload_sha256: [u8; 32],
}

pub struct VerifyContext<'a> {
    pub enrollment_id: Uuid,
    pub now: u64,
    pub trusted_time_floor: u64,
    pub version_floor: VersionFloor,
    pub keyring: &'a BTreeMap<String, VerifyingKey>,
}

pub struct VerifiedBundle {
    bundle: Bundle,
    floor: VersionFloor,
}

impl fmt::Debug for VerifiedBundle {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("VerifiedBundle")
            .field("version", &self.bundle.version)
            .field("expires_at", &self.bundle.expires_at)
            .finish_non_exhaustive()
    }
}

impl VerifiedBundle {
    pub fn bundle(&self) -> &Bundle {
        &self.bundle
    }

    pub fn version_floor(&self) -> VersionFloor {
        self.floor
    }

    pub fn valid_at(&self, now: u64) -> bool {
        now < self.bundle.expires_at && self.bundle.issued_at <= now.saturating_add(CLOCK_SKEW)
    }
}

#[derive(Debug, Error, PartialEq, Eq)]
pub enum VerifyError {
    #[error("BUNDLE_TOO_LARGE")]
    TooLarge,
    #[error("BUNDLE_ENCODING")]
    Encoding,
    #[error("BUNDLE_SCHEMA")]
    Schema,
    #[error("BUNDLE_UNKNOWN_KEY")]
    UnknownKey,
    #[error("BUNDLE_SIGNATURE")]
    Signature,
    #[error("BUNDLE_BINDING")]
    Binding,
    #[error("BUNDLE_CLOCK")]
    Clock,
    #[error("BUNDLE_EXPIRED_OR_INVALID_LEASE")]
    Lease,
    #[error("BUNDLE_ROLLBACK")]
    Rollback,
    #[error("BUNDLE_VERSION_CONFLICT")]
    VersionConflict,
    #[error("PROFILE_INVALID")]
    Profile,
    #[error("POLICY_UNSUPPORTED")]
    UnsupportedPolicy,
}

pub fn verify(bytes: &[u8], ctx: &VerifyContext<'_>) -> Result<VerifiedBundle, VerifyError> {
    if bytes.len() > MAX_ENVELOPE {
        return Err(VerifyError::TooLarge);
    }
    let envelope: Envelope = serde_json::from_slice(bytes).map_err(|_| VerifyError::Schema)?;
    if envelope.key_id.len() > 64 {
        return Err(VerifyError::UnknownKey);
    }
    let key = ctx
        .keyring
        .get(&envelope.key_id)
        .ok_or(VerifyError::UnknownKey)?;
    let payload = URL_SAFE_NO_PAD
        .decode(&envelope.payload)
        .map_err(|_| VerifyError::Encoding)?;
    if payload.len() > MAX_PAYLOAD {
        return Err(VerifyError::TooLarge);
    }
    let sig_bytes = URL_SAFE_NO_PAD
        .decode(&envelope.signature)
        .map_err(|_| VerifyError::Encoding)?;
    let signature = Signature::from_slice(&sig_bytes).map_err(|_| VerifyError::Signature)?;
    let mut message = Vec::with_capacity(SIGNING_PREFIX.len() + payload.len());
    message.extend_from_slice(SIGNING_PREFIX);
    message.extend_from_slice(&payload);
    key.verify_strict(&message, &signature)
        .map_err(|_| VerifyError::Signature)?;
    let bundle: Bundle = serde_json::from_slice(&payload).map_err(|_| VerifyError::Schema)?;
    if bundle.schema_version != 1
        || bundle.context != CONTEXT
        || bundle.key_id != envelope.key_id
        || bundle.enrollment_id != ctx.enrollment_id
        || bundle.enrollment_id.is_nil()
    {
        return Err(VerifyError::Binding);
    }
    if ctx.now.saturating_add(CLOCK_SKEW) < ctx.trusted_time_floor
        || bundle.issued_at > ctx.now.saturating_add(CLOCK_SKEW)
    {
        return Err(VerifyError::Clock);
    }
    if bundle.expires_at <= ctx.now
        || bundle.expires_at <= bundle.issued_at
        || bundle.expires_at - bundle.issued_at > MAX_LEASE
    {
        return Err(VerifyError::Lease);
    }
    if bundle.version == 0 || bundle.version < ctx.version_floor.version {
        return Err(VerifyError::Rollback);
    }
    let hash: [u8; 32] = Sha256::digest(&payload).into();
    if bundle.version == ctx.version_floor.version && hash != ctx.version_floor.payload_sha256 {
        return Err(VerifyError::VersionConflict);
    }
    validate_profile(&bundle.profile)?;
    if bundle.policy.kill_switch {
        return Err(VerifyError::UnsupportedPolicy);
    }
    if !(60..=86400).contains(&bundle.policy.refresh_interval_seconds) {
        return Err(VerifyError::Profile);
    }
    let floor = VersionFloor {
        version: bundle.version,
        payload_sha256: hash,
    };
    Ok(VerifiedBundle { bundle, floor })
}

fn hostname(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= 253
        && value.is_ascii()
        && value.split('.').all(|label| {
            !label.is_empty()
                && label.len() <= 63
                && !label.starts_with('-')
                && !label.ends_with('-')
                && label
                    .bytes()
                    .all(|b| b.is_ascii_alphanumeric() || b == b'-')
        })
}

fn usable_ip(ip: IpAddr) -> bool {
    !ip.is_unspecified() && !ip.is_loopback() && !ip.is_multicast()
}

fn validate_profile(p: &Profile) -> Result<(), VerifyError> {
    let server_ok = match p.server.parse::<IpAddr>() {
        Ok(ip) => usable_ip(ip),
        Err(_) => hostname(&p.server),
    };
    if !server_ok
        || !hostname(&p.sni)
        || p.port == 0
        || p.uuid.is_nil()
        || !(1280..=1500).contains(&p.mtu)
        || p.dns.is_empty()
        || p.dns.len() > 4
        || p.dns.iter().any(|ip| !usable_ip(*ip))
        || p.exclude_cidrs.len() > 128
        || !p.exclude_domains.is_empty()
        || p.short_id.len() > 16
        || !p.short_id.len().is_multiple_of(2)
        || !p.short_id.bytes().all(|b| b.is_ascii_hexdigit())
        || URL_SAFE_NO_PAD
            .decode(&p.public_key)
            .map_or(true, |key| key.len() != 32)
    {
        return Err(VerifyError::Profile);
    }
    Ok(())
}
