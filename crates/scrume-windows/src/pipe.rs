//! Single-instance local message pipe. The same handle is retained for its entire
//! lifetime to prevent namespace takeover between connections. No arbitrary IO.
use scrume_core::{authorization::Peer, ipc::MAX_FRAME};
use std::{
    ffi::c_void,
    io,
    mem::{size_of, size_of_val},
    ptr::{null, null_mut},
    thread,
    time::{Duration, Instant},
};
use windows_sys::Win32::{
    Foundation::{
        CloseHandle, ERROR_BROKEN_PIPE, ERROR_NO_DATA, ERROR_PIPE_CONNECTED, ERROR_PIPE_LISTENING,
        GetLastError, HANDLE, INVALID_HANDLE_VALUE, LocalFree,
    },
    Security::{
        Authorization::{
            ConvertSidToStringSidW, ConvertStringSecurityDescriptorToSecurityDescriptorW,
            SDDL_REVISION_1,
        },
        GetTokenInformation, RevertToSelf, SECURITY_ATTRIBUTES, TOKEN_QUERY, TOKEN_USER,
        TokenSessionId, TokenUser,
    },
    Storage::FileSystem::{
        CreateFileW, FILE_FLAG_FIRST_PIPE_INSTANCE, FILE_GENERIC_READ, FILE_WRITE_ATTRIBUTES,
        FILE_WRITE_DATA, OPEN_EXISTING, PIPE_ACCESS_DUPLEX, ReadFile, SECURITY_IDENTIFICATION,
        SECURITY_SQOS_PRESENT, WriteFile,
    },
    System::{
        Pipes::{
            ConnectNamedPipe, CreateNamedPipeW, DisconnectNamedPipe, GetNamedPipeClientSessionId,
            ImpersonateNamedPipeClient, PIPE_NOWAIT, PIPE_READMODE_MESSAGE,
            PIPE_REJECT_REMOTE_CLIENTS, PIPE_TYPE_MESSAGE, PeekNamedPipe, SetNamedPipeHandleState,
        },
        RemoteDesktop::WTSGetActiveConsoleSessionId,
        Threading::{GetCurrentThread, OpenThreadToken},
    },
};

pub const NAME: &str = r"\\.\pipe\ScrumeVPN-v1";
// IU receives individual data/attribute rights, NOT FILE_CREATE_PIPE_INSTANCE
// (which is included in FILE_GENERIC_WRITE). The client must request exact rights.
const SDDL: &str = "D:P(A;;GA;;;SY)(A;;0x0012018b;;;IU)";
const DEADLINE: Duration = Duration::from_secs(1);
const POLL: Duration = Duration::from_millis(10);

fn wide(s: &str) -> Vec<u16> {
    s.encode_utf16().chain(Some(0)).collect()
}
fn last() -> io::Error {
    io::Error::last_os_error()
}
fn denied() -> io::Error {
    io::Error::new(io::ErrorKind::PermissionDenied, "IPC_IDENTITY_REJECTED")
}

struct Handle(HANDLE);
impl Drop for Handle {
    fn drop(&mut self) {
        // SAFETY: owns one valid Win32 handle; no aliases close it.
        unsafe {
            CloseHandle(self.0);
        }
    }
}
struct Allocation(*mut c_void);
impl Drop for Allocation {
    fn drop(&mut self) {
        // SAFETY: allocated by LocalAlloc-family Win32 APIs, freed exactly once.
        unsafe {
            LocalFree(self.0);
        }
    }
}
struct Impersonation;
impl Drop for Impersonation {
    fn drop(&mut self) {
        // SAFETY: same thread that successfully impersonated; no await/thread hop.
        // Continuing with an unexpected identity is forbidden.
        if unsafe { RevertToSelf() } == 0 {
            std::process::abort();
        }
    }
}

pub struct Server {
    handle: Handle,
}
impl Server {
    pub fn create() -> io::Result<Self> {
        let mut descriptor = null_mut();
        let sddl = wide(SDDL);
        // SAFETY: NUL-terminated descriptor, valid out pointer. RAII frees allocation.
        if unsafe {
            ConvertStringSecurityDescriptorToSecurityDescriptorW(
                sddl.as_ptr(),
                SDDL_REVISION_1,
                &mut descriptor,
                null_mut(),
            )
        } == 0
        {
            return Err(last());
        }
        let descriptor = Allocation(descriptor);
        let attributes = SECURITY_ATTRIBUTES {
            nLength: size_of::<SECURITY_ATTRIBUTES>() as u32,
            lpSecurityDescriptor: descriptor.0,
            bInheritHandle: 0,
        };
        let name = wide(NAME);
        // SAFETY: attributes live through call; noninheritable handle; fixed local
        // name, one instance, message boundaries, bounded buffers, nonblocking IO.
        let raw = unsafe {
            CreateNamedPipeW(
                name.as_ptr(),
                PIPE_ACCESS_DUPLEX | FILE_FLAG_FIRST_PIPE_INSTANCE,
                PIPE_TYPE_MESSAGE
                    | PIPE_READMODE_MESSAGE
                    | PIPE_NOWAIT
                    | PIPE_REJECT_REMOTE_CLIENTS,
                1,
                8192,
                8192,
                1000,
                &attributes,
            )
        };
        if raw == INVALID_HANDLE_VALUE {
            return Err(last());
        }
        Ok(Self {
            handle: Handle(raw),
        })
    }

    /// At most one bounded request is served per call. No raw error/payload logs.
    pub fn poll(
        &self,
        mut reply: impl FnMut(&Peer, &[u8]) -> Vec<u8>,
        cancelled: impl Fn() -> bool,
    ) -> io::Result<()> {
        // SAFETY: synchronous nonblocking handle; no OVERLAPPED needed.
        let connected = unsafe { ConnectNamedPipe(self.handle.0, null_mut()) };
        if connected == 0 {
            // SAFETY: immediately following the failed call.
            match unsafe { GetLastError() } {
                ERROR_PIPE_LISTENING => return Ok(()),
                ERROR_PIPE_CONNECTED => {}
                ERROR_NO_DATA | ERROR_BROKEN_PIPE => {
                    self.disconnect();
                    return Ok(());
                }
                _ => return Err(last()),
            }
        }
        let _ = self.serve(&mut reply, &cancelled);
        self.disconnect();
        // Invalid clients are dropped; only listener creation/system failures
        // should prevent SCM startup. No error text is exposed to clients.
        Ok(())
    }

    fn serve(
        &self,
        reply: &mut impl FnMut(&Peer, &[u8]) -> Vec<u8>,
        cancelled: &impl Fn() -> bool,
    ) -> io::Result<()> {
        let deadline = Instant::now() + DEADLINE;
        let mut buffer = vec![0u8; MAX_FRAME + 1];
        let count = loop {
            if cancelled() || Instant::now() >= deadline {
                return Ok(());
            }
            let mut read = 0;
            // SAFETY: writable buffer and valid synchronous handle. Message pipe
            // returns ERROR_MORE_DATA for oversized input, which is rejected.
            if unsafe {
                ReadFile(
                    self.handle.0,
                    buffer.as_mut_ptr(),
                    buffer.len() as u32,
                    &mut read,
                    null_mut(),
                )
            } != 0
            {
                if read as usize > MAX_FRAME || read == 0 {
                    return Ok(());
                }
                break read as usize;
            }
            // SAFETY: immediately after ReadFile.
            match unsafe { GetLastError() } {
                ERROR_NO_DATA | ERROR_PIPE_LISTENING => {}
                _ => return Err(last()),
            }
            thread::sleep(POLL);
        };
        let peer = self.peer()?;
        // Impersonation guard has reverted before any handler logic or IO.
        let response = reply(&peer, &buffer[..count]);
        if response.len() > 4096 {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "IPC_RESPONSE_TOO_LARGE",
            ));
        }
        let mut written = 0;
        // SAFETY: bounded immutable response bytes live through synchronous call.
        if unsafe {
            WriteFile(
                self.handle.0,
                response.as_ptr(),
                response.len() as u32,
                &mut written,
                null_mut(),
            )
        } == 0
            || written as usize != response.len()
        {
            return Err(last());
        }
        // Do not FlushFileBuffers: a malicious client could block service stop.
        // Give the client a bounded window to read and close before disconnect.
        while !cancelled() && Instant::now() < deadline {
            // SAFETY: query only, optional output pointers are NULL.
            if unsafe {
                PeekNamedPipe(
                    self.handle.0,
                    null_mut(),
                    0,
                    null_mut(),
                    null_mut(),
                    null_mut(),
                )
            } == 0
            {
                break;
            }
            thread::sleep(POLL);
        }
        Ok(())
    }

    fn peer(&self) -> io::Result<Peer> {
        let mut transport_session = 0;
        // SAFETY: valid pipe, valid output storage.
        if unsafe { GetNamedPipeClientSessionId(self.handle.0, &mut transport_session) } == 0 {
            return Err(denied());
        }
        // SAFETY: called after reading the request; always revert on this thread.
        if unsafe { ImpersonateNamedPipeClient(self.handle.0) } == 0 {
            return Err(denied());
        }
        let guard = Impersonation;
        let mut raw = null_mut();
        // SAFETY: output handle; query-only access, use server identity for open check.
        if unsafe { OpenThreadToken(GetCurrentThread(), TOKEN_QUERY, 1, &mut raw) } == 0 {
            return Err(denied());
        }
        let token = Handle(raw);
        let mut token_session: u32 = 0;
        let mut required = 0;
        // SAFETY: correct class and storage size for u32 session id.
        if unsafe {
            GetTokenInformation(
                token.0,
                TokenSessionId,
                (&mut token_session as *mut u32).cast(),
                size_of_val(&token_session) as u32,
                &mut required,
            )
        } == 0
        {
            return Err(denied());
        }
        // SAFETY: probe for buffer size; expected insufficient-buffer error.
        unsafe {
            GetTokenInformation(token.0, TokenUser, null_mut(), 0, &mut required);
        }
        if required < size_of::<TOKEN_USER>() as u32 || required > 65536 {
            return Err(denied());
        }
        // usize backing guarantees alignment sufficient for TOKEN_USER.
        let mut storage = vec![0usize; (required as usize).div_ceil(size_of::<usize>())];
        let capacity = size_of_val(storage.as_slice()) as u32;
        // SAFETY: aligned writable storage of sufficient size, result checked.
        if unsafe {
            GetTokenInformation(
                token.0,
                TokenUser,
                storage.as_mut_ptr().cast(),
                capacity,
                &mut required,
            )
        } == 0
        {
            return Err(denied());
        }
        // SAFETY: successful TokenUser query populated aligned TOKEN_USER.
        let user = unsafe { &*storage.as_ptr().cast::<TOKEN_USER>() };
        let mut sid_text = null_mut();
        // SAFETY: SID pointer belongs to valid token query buffer.
        if unsafe { ConvertSidToStringSidW(user.User.Sid, &mut sid_text) } == 0 {
            return Err(denied());
        }
        let allocation = Allocation(sid_text.cast());
        let mut length = 0;
        // SAFETY: API returns a NUL-terminated SID string; hard cap additionally
        // bounds the scan to the documented SID string maximum-sized allocation.
        while length < 184 && unsafe { *sid_text.add(length) } != 0 {
            length += 1;
        }
        if length == 184 {
            return Err(denied());
        }
        // SAFETY: length bounded by NUL scan over API-provided string.
        let sid = String::from_utf16(unsafe { std::slice::from_raw_parts(sid_text, length) })
            .map_err(|_| denied())?;
        drop(allocation);
        drop(token);
        drop(guard);
        // SAFETY: read-only system query, no parameters.
        Ok(Peer {
            sid,
            token_session,
            transport_session,
            active_console_session: unsafe { WTSGetActiveConsoleSessionId() },
        })
    }

    fn disconnect(&self) {
        // SAFETY: server-owned handle retained for listener lifetime; no close.
        unsafe {
            DisconnectNamedPipe(self.handle.0);
        }
    }
}

/// Development status client, not enrollment or secret transfer. Identity-level
/// QOS prevents a spoof server from impersonating the user for privileged actions.
pub fn status() -> io::Result<Vec<u8>> {
    let name = wide(NAME);
    // SAFETY: fixed trusted pipe name; individual rights exclude pipe creation.
    let raw = unsafe {
        CreateFileW(
            name.as_ptr(),
            FILE_GENERIC_READ | FILE_WRITE_DATA | FILE_WRITE_ATTRIBUTES,
            0,
            null(),
            OPEN_EXISTING,
            SECURITY_SQOS_PRESENT | SECURITY_IDENTIFICATION,
            null_mut(),
        )
    };
    if raw == INVALID_HANDLE_VALUE {
        return Err(last());
    }
    let handle = Handle(raw);
    let mode = PIPE_READMODE_MESSAGE | PIPE_NOWAIT;
    // SAFETY: pipe client handle, valid mode storage. Requires write attributes.
    if unsafe { SetNamedPipeHandleState(handle.0, &mode, null(), null()) } == 0 {
        return Err(last());
    }
    let request = br#"{"protocol_version":1,"request_id":1,"method":"status.get"}"#;
    let mut written = 0;
    // SAFETY: fixed bounded request and valid synchronous pipe handle.
    if unsafe {
        WriteFile(
            handle.0,
            request.as_ptr(),
            request.len() as u32,
            &mut written,
            null_mut(),
        )
    } == 0
        || written as usize != request.len()
    {
        return Err(last());
    }
    let mut response = vec![0u8; 4096];
    let deadline = Instant::now() + DEADLINE;
    loop {
        let mut read = 0;
        // SAFETY: writable bounded output and valid nonblocking handle.
        if unsafe {
            ReadFile(
                handle.0,
                response.as_mut_ptr(),
                response.len() as u32,
                &mut read,
                null_mut(),
            )
        } != 0
        {
            response.truncate(read as usize);
            return Ok(response);
        }
        // SAFETY: immediately following failed ReadFile.
        if unsafe { GetLastError() } != ERROR_NO_DATA {
            return Err(last());
        }
        if Instant::now() >= deadline {
            return Err(io::Error::new(io::ErrorKind::TimedOut, "IPC_TIMEOUT"));
        }
        thread::sleep(POLL);
    }
}
