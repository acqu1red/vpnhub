//! SCM lifecycle scaffold. Does not expose IPC or claim a working VPN.
#[cfg(windows)]
mod scm;

fn main() -> std::process::ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    if args == ["--capabilities"] {
        println!("SCM_SCAFFOLD; IPC=NOT_IMPLEMENTED; VAULT=NOT_IMPLEMENTED; TUN=NOT_IMPLEMENTED");
        return std::process::ExitCode::SUCCESS;
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
