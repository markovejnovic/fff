use std::fs;
use std::path::PathBuf;
use std::process::Command;
use std::sync::Once;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::Duration;

static NEXT_ID: AtomicUsize = AtomicUsize::new(0);
static DAEMON: Once = Once::new();

/// Max time to wait for the daemon socket to become connectable.
const DAEMON_STARTUP_TIMEOUT: Duration = Duration::from_secs(10);
/// How often to poll the socket during startup.
const DAEMON_POLL_INTERVAL: Duration = Duration::from_millis(100);

pub fn find_binary(name: &str) -> PathBuf {
    let mut bin = std::env::current_exe().unwrap();
    bin.pop();
    bin.pop();
    bin.push(name);
    bin
}

fn ensure_daemon() {
    DAEMON.call_once(|| {
        use std::os::unix::net::UnixStream;

        let socket = fff_ipc_domain::daemon_socket_path();
        if UnixStream::connect(&socket).is_ok() {
            return;
        }

        let bin = find_binary("fff-daemon");
        assert!(
            bin.exists(),
            "fff-daemon not found at {}. Run: cargo build -p fff-daemon",
            bin.display()
        );

        Command::new(&bin)
            .stdin(std::process::Stdio::null())
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null())
            .spawn()
            .unwrap_or_else(|e| panic!("failed to spawn fff-daemon: {e}"));

        let deadline = std::time::Instant::now() + DAEMON_STARTUP_TIMEOUT;
        while std::time::Instant::now() < deadline {
            std::thread::sleep(DAEMON_POLL_INTERVAL);
            if UnixStream::connect(&socket).is_ok() {
                return;
            }
        }
        panic!("fff-daemon did not start within {DAEMON_STARTUP_TIMEOUT:?}");
    });
}

pub struct Dir {
    dir: PathBuf,
}

impl Dir {
    pub fn new(name: &str) -> Self {
        ensure_daemon();

        let id = NEXT_ID.fetch_add(1, Ordering::Relaxed);
        let pid = std::process::id();
        let dir = std::env::temp_dir().join("fff-rg-tests").join(format!("{name}-{pid}-{id}"));

        if dir.exists() {
            let _ = fs::remove_dir_all(&dir);
        }
        fs::create_dir_all(&dir).unwrap();

        Command::new("git")
            .args(["init", "-q"])
            .current_dir(&dir)
            .output()
            .expect("git init failed");

        Self { dir }
    }

    pub fn create(&self, name: &str, contents: &str) {
        let path = self.dir.join(name);
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).unwrap();
        }
        fs::write(path, contents).unwrap();
    }

    pub fn command(&self) -> TestCommand {
        let bin = find_binary("fff-rg");
        let mut cmd = Command::new(&bin);
        cmd.current_dir(&self.dir);
        TestCommand { cmd, dir: self.dir.clone() }
    }
}

impl Drop for Dir {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.dir);
    }
}

pub struct TestCommand {
    cmd: Command,
    dir: PathBuf,
}

impl TestCommand {
    pub fn arg(&mut self, arg: &str) -> &mut Self {
        self.cmd.arg(arg);
        self
    }

    pub fn args(&mut self, args: &[&str]) -> &mut Self {
        self.cmd.args(args);
        self
    }

    pub fn stdout(&mut self) -> String {
        let output = self
            .cmd
            .output()
            .unwrap_or_else(|e| panic!("failed to run fff-rg: {e}\ndir: {}", self.dir.display()));
        String::from_utf8(output.stdout).unwrap()
    }

    pub fn exit_code(&mut self) -> i32 {
        let output = self.cmd.output().unwrap();
        output.status.code().unwrap_or(-1)
    }
}
