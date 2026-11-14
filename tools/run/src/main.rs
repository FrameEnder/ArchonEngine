use std::process::Stdio;
use tokio::process::Command;

async fn start_server(name: &str) {
    let mut dir = std::env::current_exe().expect("Couldn't get current executable path");
    dir.pop();

    let mut extension = std::env::consts::EXE_EXTENSION.to_string();
    if !extension.is_empty() {
        extension = format!(".{extension}");
    }

    dir.push(format!("{name}{}", extension));

    let library_path = if std::env::var("CARGO").is_ok() {
        "./oodle"
    } else {
        "."
    };

    // A server that ends is started again (FrameEnder fork, 2026-11-14): it was waited for once, so a world server
    // that crashed stayed down, with the others up and the container looking healthy, until someone restarted it
    // (Bob's bundles whose log ends in a panic). Straight away the first time; a server that keeps ending within
    // `SETTLED` waits longer each time, up to `LONGEST_WAIT`, so a broken install doesn't spin.
    let mut wait = FIRST_WAIT;
    loop {
        let started = std::time::Instant::now();
        let status = Command::new(&dir)
            .env("LD_LIBRARY_PATH", library_path) // ensure we find the oodle .so at the right location
            .env("RUST_BACKTRACE", "1") // Print backtraces on asserts
            .stdout(Stdio::inherit())
            .spawn()
            .expect("Failed to run server")
            .wait()
            .await
            .expect("Failed to run server");
        wait = next_wait(wait, started.elapsed());
        eprintln!(
            "kawari-run: {name} ended ({status}); starting it again in {} s",
            wait.as_secs()
        );
        tokio::time::sleep(wait).await;
    }
}

const FIRST_WAIT: std::time::Duration = std::time::Duration::from_secs(1);
const LONGEST_WAIT: std::time::Duration = std::time::Duration::from_secs(60);
/// A server that ran this long was working: the next restart is quick again.
const SETTLED: std::time::Duration = std::time::Duration::from_secs(120);

/// How long to wait before starting a server again, from the last wait and how long it ran.
fn next_wait(last: std::time::Duration, ran: std::time::Duration) -> std::time::Duration {
    if ran >= SETTLED {
        FIRST_WAIT
    } else {
        (last * 2).min(LONGEST_WAIT)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    #[test]
    fn a_server_that_keeps_ending_waits_longer_and_one_that_ran_doesn_t() {
        let mut wait = FIRST_WAIT;
        let mut waits = Vec::new();
        for _ in 0..8 {
            wait = next_wait(wait, Duration::from_secs(3));
            waits.push(wait.as_secs());
        }
        assert_eq!(waits, [2, 4, 8, 16, 32, 60, 60, 60]);
        assert_eq!(next_wait(wait, Duration::from_secs(3600)), FIRST_WAIT);
    }
}

#[tokio::main]
async fn main() {
    // Enables ANSI code support on Windows. See https://github.com/tokio-rs/tracing/issues/3068
    #[cfg(windows)]
    nu_ansi_term::enable_ansi_support().ok();

    // If being invoked by Cargo, build the workspace first.
    if let Ok(cargo) = std::env::var("CARGO") {
        let build_exit_status = Command::new(cargo)
            .args(if cfg!(debug_assertions) {
                vec!["build", "--features", "oodle"]
            } else {
                vec!["build", "--release", "--features", "oodle"]
            })
            .stdout(Stdio::inherit())
            .spawn()
            .expect("Failed to run Cargo build")
            .wait()
            .await
            .expect("Failed to run Cargo build");

        // Silently exit if build failed
        if !build_exit_status.success() {
            return;
        }
    }

    tokio::join!(
        start_server("kawari-admin"),
        start_server("kawari-frontier"),
        start_server("kawari-lobby"),
        start_server("kawari-login"),
        start_server("kawari-patch"),
        start_server("kawari-web"),
        start_server("kawari-world"),
    );
}
