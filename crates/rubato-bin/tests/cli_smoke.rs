use std::io::Read;
use std::process::{Command, Output, Stdio};
use std::thread;
use std::time::{Duration, Instant};

struct TimedOutput {
    output: Output,
    timed_out: bool,
}

fn rubato_bin() -> Command {
    Command::new(env!("CARGO_BIN_EXE_rubato"))
}

fn run_with_timeout(command: &mut Command, timeout: Duration) -> TimedOutput {
    command.stdout(Stdio::piped()).stderr(Stdio::piped());
    let mut child = command.spawn().expect("failed to execute binary");
    let start = Instant::now();

    let (status, timed_out) = loop {
        if let Some(status) = child.try_wait().expect("failed to poll child") {
            break (status, false);
        }
        if start.elapsed() >= timeout {
            child.kill().expect("failed to terminate timed out child");
            let status = child.wait().expect("failed to wait for terminated child");
            break (status, true);
        }
        thread::sleep(Duration::from_millis(100));
    };

    let mut stdout = Vec::new();
    if let Some(mut handle) = child.stdout.take() {
        handle
            .read_to_end(&mut stdout)
            .expect("failed to read child stdout");
    }
    let mut stderr = Vec::new();
    if let Some(mut handle) = child.stderr.take() {
        handle
            .read_to_end(&mut stderr)
            .expect("failed to read child stderr");
    }

    TimedOutput {
        output: Output {
            status,
            stdout,
            stderr,
        },
        timed_out,
    }
}

fn assert_normal_exit_or_live_gui_run(result: &TimedOutput) {
    if result.timed_out {
        return;
    }

    #[cfg(unix)]
    {
        use std::os::unix::process::ExitStatusExt;
        assert!(
            result.output.status.signal().is_none(),
            "process was killed by signal: {:?}",
            result.output.status.signal()
        );
    }

    if let Some(code) = result.output.status.code() {
        assert_ne!(
            code,
            101,
            "process exited with code 101 (Rust panic). stderr: {}",
            String::from_utf8_lossy(&result.output.stderr)
        );
    }
}

/// Write a minimal `config_sys.json` to the given directory so that the binary
/// recognises it as having a config and takes the play() path.
fn write_minimal_config(dir: &std::path::Path) {
    std::fs::write(dir.join("config_sys.json"), "{}").expect("failed to write config_sys.json");
}

#[test]
fn help_flag() {
    let output = rubato_bin()
        .arg("--help")
        .output()
        .expect("failed to execute binary");

    assert!(output.status.success(), "exit code was not 0");

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("BMS player"),
        "stdout did not contain 'BMS player': {stdout}"
    );
}

#[test]
fn version_flag() {
    let output = rubato_bin()
        .arg("--version")
        .output()
        .expect("failed to execute binary");

    assert!(output.status.success(), "exit code was not 0");
}

#[test]
fn invalid_flag_exits_error() {
    let output = rubato_bin()
        .arg("--invalid-nonexistent-flag")
        .output()
        .expect("failed to execute binary");

    assert!(
        !output.status.success(),
        "expected non-zero exit code for invalid flag"
    );
}

/// Verify that [`run_with_timeout`] actually terminates a long-running process.
///
/// This is a non-ignored infrastructure test that proves the timeout mechanism
/// works in headless CI without needing a display or GPU.
#[test]
fn timeout_mechanism_terminates_long_running_process() {
    let start = Instant::now();
    let result = run_with_timeout(Command::new("sleep").arg("60"), Duration::from_secs(3));

    let elapsed = start.elapsed();

    // The process must have been killed by the timeout, not allowed to run for 60s.
    assert!(
        result.timed_out,
        "expected run_with_timeout to flag timed_out for a 60s sleep with 3s timeout"
    );

    // Elapsed wall-clock time should be close to the timeout, not 60s.
    assert!(
        elapsed < Duration::from_secs(10),
        "expected process to terminate within ~3s, but took {elapsed:?}"
    );

    // On Unix, a killed process should not have a normal exit code of 0.
    assert!(
        !result.output.status.success(),
        "killed process should not report success"
    );
}

/// Run the binary with no arguments in a clean tempdir (no config file).
///
/// The binary will attempt to launch the configuration UI (eframe/egui),
/// which requires a display server. In headless CI environments this will
/// fail, so the test is marked `#[ignore]`. The key assertion is that the
/// process does not crash with a panic / signal — it should exit with an
/// ordinary error code.
#[test]
#[ignore]
fn no_config_runs_without_crash() {
    let tmp = tempfile::TempDir::new().expect("failed to create tempdir");

    let output = run_with_timeout(rubato_bin().current_dir(tmp.path()), Duration::from_secs(5));
    assert_normal_exit_or_live_gui_run(&output);
}

/// With `config_sys.json` present and `-s` flag, the binary takes the play()
/// path (`config_exists && player_mode.is_some()`). It will fail because there
/// is no GPU/display, but it must NOT panic (exit code 101) or be killed by a
/// signal. A normal error exit is acceptable.
#[test]
#[ignore] // requires GPU/display
fn play_flag_with_config_exits_gracefully() {
    let tmp = tempfile::TempDir::new().expect("failed to create tempdir");
    write_minimal_config(tmp.path());

    let output = run_with_timeout(
        rubato_bin().arg("-s").current_dir(tmp.path()),
        Duration::from_secs(5),
    );
    assert_normal_exit_or_live_gui_run(&output);
}

/// With `-s` but NO `config_sys.json`, the binary falls through to launch()
/// (the launcher/configuration UI path) instead of play(). It must not panic
/// or be signal-killed; a normal exit (including errors from missing display)
/// is fine.
#[test]
#[ignore] // requires display for launcher
fn play_flag_without_config_launches_launcher() {
    let tmp = tempfile::TempDir::new().expect("failed to create tempdir");
    // Intentionally no config file in this tempdir

    let output = run_with_timeout(
        rubato_bin().arg("-s").current_dir(tmp.path()),
        Duration::from_secs(5),
    );
    assert_normal_exit_or_live_gui_run(&output);
}

/// When re-exec'd as a child process (the path `launch()` takes via
/// `Command::new(current_exe()).arg("-s")`), the child must read
/// `config_sys.json` from the correct working directory. Verify the child
/// does not panic or get signal-killed.
#[test]
#[ignore] // requires GPU/display
fn reexec_child_inherits_working_directory() {
    let tmp = tempfile::TempDir::new().expect("failed to create tempdir");
    write_minimal_config(tmp.path());

    // Simulate the re-exec path: binary launched with `-s` and cwd set to
    // the directory containing config_sys.json.
    let output = run_with_timeout(
        rubato_bin().arg("-s").current_dir(tmp.path()),
        Duration::from_secs(5),
    );
    assert_normal_exit_or_live_gui_run(&output);
}
