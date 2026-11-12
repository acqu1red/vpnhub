//! Machine-context DPAPI primitive, NOT a service-only filesystem vault.
//! Before persistence the caller must enforce SYSTEM-only ACLs and trusted paths.
use std::{
    fmt, io,
    ptr::{null, null_mut},
};
use windows_sys::Win32::{
    Foundation::LocalFree,
    Security::Cryptography::{
        CRYPT_INTEGER_BLOB, CRYPTPROTECT_LOCAL_MACHINE, CRYPTPROTECT_UI_FORBIDDEN,
        CryptProtectData, CryptUnprotectData,
    },
};
use zeroize::{Zeroize, Zeroizing};

const MAX_SECRET: usize = 8192;
const MAX_CIPHERTEXT: usize = 16384;
const ENTROPY: &[u8] = b"scrumevpn/service-enrollment/v1";

pub struct SecretBytes(Zeroizing<Vec<u8>>);
impl SecretBytes {
    pub fn as_bytes(&self) -> &[u8] {
        &self.0
    }
}
impl fmt::Debug for SecretBytes {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("SecretBytes([REDACTED])")
    }
}

pub struct ProtectedBytes(Vec<u8>);
impl ProtectedBytes {
    pub fn as_bytes(&self) -> &[u8] {
        &self.0
    }
    pub fn from_bytes(bytes: Vec<u8>) -> io::Result<Self> {
        if bytes.is_empty() || bytes.len() > MAX_CIPHERTEXT {
            return Err(invalid());
        }
        Ok(Self(bytes))
    }
}
impl fmt::Debug for ProtectedBytes {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("ProtectedBytes([REDACTED])")
    }
}

fn invalid() -> io::Error {
    io::Error::new(io::ErrorKind::InvalidData, "DPAPI_INVALID_BLOB")
}
struct NativeBlob(CRYPT_INTEGER_BLOB);
impl Drop for NativeBlob {
    fn drop(&mut self) {
        if !self.0.pbData.is_null() {
            // SAFETY: API-allocated buffer with API-provided size, owned once.
            // Wipe native plaintext before release as well as the Rust-owned copy.
            unsafe {
                std::slice::from_raw_parts_mut(self.0.pbData, self.0.cbData as usize).zeroize();
                LocalFree(self.0.pbData.cast());
            }
        }
    }
}

fn input(bytes: &[u8]) -> CRYPT_INTEGER_BLOB {
    // Windows API uses a mutable pointer type but documents these input blobs
    // as read-only. The slices remain valid for the synchronous call.
    CRYPT_INTEGER_BLOB {
        cbData: bytes.len() as u32,
        pbData: bytes.as_ptr().cast_mut(),
    }
}

pub fn protect_machine(secret: &[u8]) -> io::Result<ProtectedBytes> {
    if secret.is_empty() || secret.len() > MAX_SECRET {
        return Err(invalid());
    }
    let mut out = CRYPT_INTEGER_BLOB {
        cbData: 0,
        pbData: null_mut(),
    };
    // SAFETY: bounded read-only inputs, no prompt, machine scope, valid output.
    if unsafe {
        CryptProtectData(
            &input(secret),
            null(),
            &input(ENTROPY),
            null(),
            null(),
            CRYPTPROTECT_LOCAL_MACHINE | CRYPTPROTECT_UI_FORBIDDEN,
            &mut out,
        )
    } == 0
    {
        return Err(io::Error::other("DPAPI_PROTECT_FAILED"));
    }
    let out = NativeBlob(out);
    if out.0.pbData.is_null() || out.0.cbData as usize > MAX_CIPHERTEXT || out.0.cbData == 0 {
        return Err(invalid());
    }
    // SAFETY: successful DPAPI output, size checked; NativeBlob frees afterward.
    let bytes = unsafe { std::slice::from_raw_parts(out.0.pbData, out.0.cbData as usize) }.to_vec();
    Ok(ProtectedBytes(bytes))
}

pub fn unprotect_machine(blob: &ProtectedBytes) -> io::Result<SecretBytes> {
    let mut out = CRYPT_INTEGER_BLOB {
        cbData: 0,
        pbData: null_mut(),
    };
    // SAFETY: bounded ciphertext created by constructor, fixed context entropy,
    // no UI allowed; no plaintext fallback on failure.
    if unsafe {
        CryptUnprotectData(
            &input(blob.as_bytes()),
            null_mut(),
            &input(ENTROPY),
            null(),
            null(),
            CRYPTPROTECT_UI_FORBIDDEN,
            &mut out,
        )
    } == 0
    {
        return Err(io::Error::other("DPAPI_UNPROTECT_FAILED"));
    }
    let out = NativeBlob(out);
    if out.0.pbData.is_null() || out.0.cbData as usize > MAX_SECRET || out.0.cbData == 0 {
        return Err(invalid());
    }
    // SAFETY: checked successful API output copied to a zeroizing allocation.
    let bytes = Zeroizing::new(
        unsafe { std::slice::from_raw_parts(out.0.pbData, out.0.cbData as usize) }.to_vec(),
    );
    Ok(SecretBytes(bytes))
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn native_machine_roundtrip_and_redaction() -> io::Result<()> {
        let data = b"mock-only-dpapi-roundtrip";
        let protected = protect_machine(data)?;
        assert!(!protected.as_bytes().windows(data.len()).any(|w| w == data));
        let secret = unprotect_machine(&protected)?;
        assert_eq!(secret.as_bytes(), data);
        assert!(!format!("{secret:?}").contains("mock-only"));
        Ok(())
    }
    #[test]
    fn corrupt_ciphertext_never_returns_plaintext() -> io::Result<()> {
        let mut protected = protect_machine(b"public-test-data")?.0;
        let last = protected.len() - 1;
        protected[last] ^= 0x80;
        assert!(unprotect_machine(&ProtectedBytes::from_bytes(protected)?).is_err());
        Ok(())
    }
    #[test]
    fn bounds_fail_before_api() {
        assert!(protect_machine(&[]).is_err());
        assert!(protect_machine(&vec![0; MAX_SECRET + 1]).is_err());
        assert!(ProtectedBytes::from_bytes(vec![]).is_err());
        assert!(ProtectedBytes::from_bytes(vec![0; MAX_CIPHERTEXT + 1]).is_err());
    }
}
