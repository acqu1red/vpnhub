use sha2::{Digest, Sha256};
use std::{
    ffi::OsString,
    fs::{File, OpenOptions},
    io::{self, Read, Write},
    thread,
    time::{Duration, Instant},
};
use windows_service::{
    service::{
        Service, ServiceAccess, ServiceErrorControl, ServiceExitCode, ServiceInfo,
        ServiceStartType, ServiceState, ServiceType,
    },
    service_manager::{ServiceManager, ServiceManagerAccess},
};
type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;
const NAME: &str = "ScrumeVPNSvc";
const DEADLINE: Duration = Duration::from_secs(10);

fn missing(error: &windows_service::Error) -> bool {
    matches!(error, windows_service::Error::Winapi(e) if e.raw_os_error() == Some(1060))
}
fn wait(service: &Service, desired: ServiceState) -> Result<()> {
    let deadline = Instant::now() + DEADLINE;
    loop {
        let status = service.query_status()?;
        if status.current_state == desired {
            return Ok(());
        }
        if (desired == ServiceState::Running && status.current_state == ServiceState::Stopped)
            || Instant::now() >= deadline
        {
            return Err("SCM_STATE_OR_DEADLINE".into());
        }
        thread::sleep(Duration::from_millis(25));
    }
}

pub fn run() -> Result<serde_json::Value> {
    let expected =
        option_env!("SCRUME_LAB_SERVICE_SHA256").ok_or("LAB_BUILD_HASH_MISSING_USE_BUILD_TOOL")?;
    if expected.len() != 64 || !expected.bytes().all(|b| b.is_ascii_hexdigit()) {
        return Err("LAB_BUILD_HASH_INVALID".into());
    }
    let manager = ServiceManager::local_computer(
        None::<&str>,
        ServiceManagerAccess::CONNECT | ServiceManagerAccess::CREATE_SERVICE,
    )?;
    match manager.open_service(NAME, ServiceAccess::QUERY_STATUS) {
        Ok(_) => return Err("EXISTING_SERVICE_REFUSED".into()),
        Err(error) if missing(&error) => {}
        Err(error) => return Err(error.into()),
    }
    let source = std::env::current_exe()?.with_file_name("ScrumeVPNSvc.exe");
    let mut content = Vec::new();
    File::open(source)?
        .take(16 * 1024 * 1024 + 1)
        .read_to_end(&mut content)?;
    if content.is_empty() || content.len() > 16 * 1024 * 1024 {
        return Err("LAB_BINARY_SIZE".into());
    }
    let hash = format!("{:x}", Sha256::digest(&content));
    if hash != expected {
        return Err("LAB_SERVICE_HASH_MISMATCH".into());
    }
    println!("[1/4] Creating protected new lab directory; service hash verified");
    let directory = scrume_windows::lab_directory::create()?;
    let installed = directory.join("ScrumeVPNSvc.exe");
    let mut file = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&installed)?;
    file.write_all(&content)?;
    file.sync_all()?;
    drop(file);
    let info = ServiceInfo {
        name: NAME.into(),
        display_name: "ScrumeVPN Lab (status only)".into(),
        service_type: ServiceType::OWN_PROCESS,
        start_type: ServiceStartType::OnDemand,
        error_control: ServiceErrorControl::Normal,
        executable_path: installed,
        launch_arguments: vec![],
        dependencies: vec![],
        account_name: None,
        account_password: None,
    };
    let service = manager.create_service(
        &info,
        ServiceAccess::START
            | ServiceAccess::STOP
            | ServiceAccess::QUERY_STATUS
            | ServiceAccess::DELETE,
    )?;
    // Every fallible check after creation returns through the cleanup path.
    let outcome = (|| -> Result<u128> {
        println!("[2/4] Starting temporary LocalSystem service");
        service.start::<OsString>(&[])?;
        wait(&service, ServiceState::Running)?;
        let bytes = scrume_windows::pipe::status()?;
        let response: serde_json::Value = serde_json::from_slice(&bytes)?;
        if response["result"]["state"] != "UNENROLLED"
            || response["result"]["tunnel_available"] != false
        {
            return Err("SCM_UNEXPECTED_STATUS".into());
        }
        println!("[3/4] Status OK; stopping service");
        let stop = Instant::now();
        service.stop()?;
        wait(&service, ServiceState::Stopped)?;
        if service.query_status()?.exit_code != ServiceExitCode::Win32(0) {
            return Err("SCM_NONZERO_EXIT".into());
        }
        Ok(stop.elapsed().as_millis())
    })();
    let cleanup = (|| -> Result<()> {
        if service.query_status()?.current_state != ServiceState::Stopped {
            service.stop()?;
            wait(&service, ServiceState::Stopped)?;
        }
        service.delete()?;
        Ok(())
    })();
    drop(service);
    cleanup?;
    println!("[4/4] Checking removal from SCM");
    let deadline = Instant::now() + DEADLINE;
    loop {
        match manager.open_service(NAME, ServiceAccess::QUERY_STATUS) {
            Err(error) if missing(&error) => break,
            Ok(_) => {}
            Err(error) => return Err(error.into()),
        }
        if Instant::now() >= deadline {
            return Err(io::Error::new(io::ErrorKind::TimedOut, "SCM_DELETE_NOT_CONFIRMED").into());
        }
        thread::sleep(Duration::from_millis(25));
    }
    let stop_ms = outcome?;
    // Program Files lab directory is retained for inspection, never recursive-deleted.
    Ok(
        serde_json::json!({"status":"PASS","scope":"SCM and active-console status IPC only; not TUN or vault", "architecture":std::env::consts::ARCH,"binary_sha256":hash,"stop_ms":stop_ms,"cleanup":"SERVICE_REMOVED"}),
    )
}
