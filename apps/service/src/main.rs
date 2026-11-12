//! SCM lifecycle and read-only status IPC. Does not claim a working VPN.
#[cfg(windows)]
mod scm;

fn main() -> std::process::ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    if args == ["--capabilities"] {
        println!("SCM; IPC=STATUS_ONLY_UNVERIFIED; VAULT=NOT_IMPLEMENTED; TUN=NOT_IMPLEMENTED");
        return std::process::ExitCode::SUCCESS;
    }
    #[cfg(windows)]
    if args == ["--status"] {
        return match scrume_windows::pipe::status() {
            Ok(bytes) => {
                println!("{}", String::from_utf8_lossy(&bytes));
                std::process::ExitCode::SUCCESS
            }
            Err(_) => {
                eprintln!("IPC_UNAVAILABLE");
                std::process::ExitCode::FAILURE
            }
        };
    }
    if !args.is_empty() {
        eprintln!("UNSUPPORTED_ARGUMENTS");
        return std::process::ExitCode::from(2);
    }
    #[cfg(windows)]
    {
        if scm::dispatch().is_err() {
            eprintln!("SERVICE_DISPATCH_FAILED: launch through Windows SCM");
            return std::process::ExitCode::FAILURE;
        }
        std::process::ExitCode::SUCCESS
    }
    #[cfg(not(windows))]
    {
        eprintln!("WINDOWS_ONLY: scaffold does not implement a tunnel on this host");
        std::process::ExitCode::from(2)
    }
}
