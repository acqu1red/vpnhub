//! Explicitly invoked, unsigned disposable-VM SCM test. Never auto-elevates.
#[cfg(windows)]
#[path = "../scm_smoke.rs"]
mod native;

fn main() -> std::process::ExitCode {
    if std::env::args().skip(1).collect::<Vec<_>>() != ["--disposable-lab"] {
        eprintln!("Usage: ScrumeVPNLab --disposable-lab (elevated disposable Windows VM only)");
        return std::process::ExitCode::from(2);
    }
    #[cfg(windows)]
    {
        match native::run() {
            Ok(report) => {
                println!("{report}");
                std::process::ExitCode::SUCCESS
            }
            Err(error) => {
                eprintln!("SCM_LAB_FAILED: {error}");
                std::process::ExitCode::FAILURE
            }
        }
    }
    #[cfg(not(windows))]
    {
        eprintln!("WINDOWS_ONLY: no service or host network changes performed");
        std::process::ExitCode::from(2)
    }
}
