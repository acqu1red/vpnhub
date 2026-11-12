use scrume_windows::pipe::Server;
use std::{
    ffi::OsString,
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
    thread,
    time::Duration,
};
use windows_service::{
    define_windows_service,
    service::{
        ServiceControl, ServiceControlAccept, ServiceExitCode, ServiceState, ServiceStatus,
        ServiceType,
    },
    service_control_handler::{self, ServiceControlHandlerResult},
    service_dispatcher,
};

const NAME: &str = "ScrumeVPNSvc";
define_windows_service!(ffi_service_main, service_main);

pub fn dispatch() -> windows_service::Result<()> {
    service_dispatcher::start(NAME, ffi_service_main)
}

fn status(state: ServiceState, accepted: ServiceControlAccept) -> ServiceStatus {
    ServiceStatus {
        service_type: ServiceType::OWN_PROCESS,
        current_state: state,
        controls_accepted: accepted,
        exit_code: ServiceExitCode::Win32(0),
        checkpoint: u32::from(
            state == ServiceState::StartPending || state == ServiceState::StopPending,
        ),
        wait_hint: if state == ServiceState::StartPending || state == ServiceState::StopPending {
            Duration::from_secs(10)
        } else {
            Duration::ZERO
        },
        process_id: None,
    }
}

fn service_main(_args: Vec<OsString>) {
    // Running describes SCM/status IPC availability, never VPN connectivity.
    if run().is_err() {
        eprintln!("SERVICE_LIFECYCLE_FAILED");
    }
}

fn run() -> windows_service::Result<()> {
    let cancelled = Arc::new(AtomicBool::new(false));
    let stop = Arc::clone(&cancelled);
    let handle = service_control_handler::register(NAME, move |control| match control {
        ServiceControl::Stop | ServiceControl::Shutdown => {
            stop.store(true, Ordering::Release);
            ServiceControlHandlerResult::NoError
        }
        ServiceControl::Interrogate => ServiceControlHandlerResult::NoError,
        _ => ServiceControlHandlerResult::NotImplemented,
    })?;
    handle.set_service_status(status(
        ServiceState::StartPending,
        ServiceControlAccept::empty(),
    ))?;
    let server = match Server::create() {
        Ok(server) => server,
        Err(error) => {
            let mut failed = status(ServiceState::Stopped, ServiceControlAccept::empty());
            failed.exit_code = ServiceExitCode::ServiceSpecific(1);
            handle.set_service_status(failed)?;
            return Err(windows_service::Error::Winapi(error));
        }
    };
    handle.set_service_status(status(
        ServiceState::Running,
        ServiceControlAccept::STOP | ServiceControlAccept::SHUTDOWN,
    ))?;
    let result = loop {
        if cancelled.load(Ordering::Acquire) {
            break Ok(());
        }
        if let Err(error) = server.poll(scrume_service::protocol::handle, || {
            cancelled.load(Ordering::Acquire)
        }) {
            break Err(error);
        }
        thread::sleep(Duration::from_millis(25));
    };
    handle.set_service_status(status(
        ServiceState::StopPending,
        ServiceControlAccept::empty(),
    ))?;
    drop(server);
    let mut stopped = status(ServiceState::Stopped, ServiceControlAccept::empty());
    if result.is_err() {
        stopped.exit_code = ServiceExitCode::ServiceSpecific(1);
    }
    handle.set_service_status(stopped)?;
    result.map_err(windows_service::Error::Winapi)
}
