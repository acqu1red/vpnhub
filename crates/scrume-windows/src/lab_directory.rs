//! Disposable lab installer helper, not the production vault/installer.
//! Creates one new fixed directory atomically with protected inherited ACLs.
use std::{
    ffi::OsString,
    io,
    mem::size_of,
    os::windows::ffi::{OsStrExt, OsStringExt},
    path::PathBuf,
    ptr::null_mut,
};
use windows_sys::Win32::{
    Foundation::LocalFree,
    Security::{
        Authorization::{ConvertStringSecurityDescriptorToSecurityDescriptorW, SDDL_REVISION_1},
        SECURITY_ATTRIBUTES,
    },
    Storage::FileSystem::{
        CreateDirectoryW, FILE_ATTRIBUTE_REPARSE_POINT, GetFileAttributesW, INVALID_FILE_ATTRIBUTES,
    },
    System::Com::CoTaskMemFree,
    UI::Shell::{FOLDERID_ProgramFiles, IsUserAnAdmin, SHGetKnownFolderPath},
};

pub fn create() -> io::Result<PathBuf> {
    // SAFETY: read-only membership query against the effective process token.
    if unsafe { IsUserAnAdmin() } == 0 {
        return Err(io::Error::new(
            io::ErrorKind::PermissionDenied,
            "LAB_ELEVATION_REQUIRED",
        ));
    }
    let mut raw = null_mut();
    // SAFETY: known constant GUID, no impersonation, valid output pointer.
    let result = unsafe { SHGetKnownFolderPath(&FOLDERID_ProgramFiles, 0, null_mut(), &mut raw) };
    if result < 0 || raw.is_null() {
        // API documentation requires freeing output even after a failed call.
        unsafe {
            CoTaskMemFree(raw.cast());
        }
        return Err(io::Error::other("LAB_PROGRAM_FILES_LOOKUP"));
    }
    let mut len = 0;
    // SAFETY: API-allocated NUL-terminated string, scanned only until its NUL.
    while unsafe { *raw.add(len) } != 0 {
        len += 1;
    }
    let folder = PathBuf::from(OsString::from_wide(unsafe {
        std::slice::from_raw_parts(raw, len)
    }));
    unsafe {
        CoTaskMemFree(raw.cast());
    }
    if !folder.is_absolute() {
        return Err(io::Error::other("LAB_PATH_INVALID"));
    }
    for ancestor in folder.ancestors() {
        let wide: Vec<u16> = ancestor.as_os_str().encode_wide().chain(Some(0)).collect();
        // SAFETY: NUL-terminated path from the trusted OS known folder result.
        let attrs = unsafe { GetFileAttributesW(wide.as_ptr()) };
        if attrs == INVALID_FILE_ATTRIBUTES || attrs & FILE_ATTRIBUTE_REPARSE_POINT != 0 {
            return Err(io::Error::other("LAB_REPARSE_OR_INVALID_PARENT"));
        }
    }
    let directory = folder.join("ScrumeVPN-Lab");
    let path: Vec<u16> = directory.as_os_str().encode_wide().chain(Some(0)).collect();
    let sddl: Vec<u16> = "D:P(A;OICI;FA;;;SY)(A;OICI;FA;;;BA)(A;OICI;FRFX;;;BU)"
        .encode_utf16()
        .chain(Some(0))
        .collect();
    let mut descriptor = null_mut();
    // SAFETY: constant SDDL, valid output pointer, freed after CreateDirectoryW.
    if unsafe {
        ConvertStringSecurityDescriptorToSecurityDescriptorW(
            sddl.as_ptr(),
            SDDL_REVISION_1,
            &mut descriptor,
            null_mut(),
        )
    } == 0
    {
        return Err(io::Error::last_os_error());
    }
    let attributes = SECURITY_ATTRIBUTES {
        nLength: size_of::<SECURITY_ATTRIBUTES>() as u32,
        lpSecurityDescriptor: descriptor,
        bInheritHandle: 0,
    };
    // SAFETY: both buffers live across synchronous call. Existing directories
    // are rejected by the API, never reused or repaired. ACL applies at creation.
    let ok = unsafe { CreateDirectoryW(path.as_ptr(), &attributes) };
    let error = io::Error::last_os_error();
    unsafe {
        LocalFree(descriptor);
    }
    if ok == 0 {
        return Err(error);
    }
    Ok(directory)
}
