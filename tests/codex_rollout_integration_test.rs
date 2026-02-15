use aura::{AgentEvent, AgentType};
use std::ffi::OsString;
use std::path::Path;
use std::sync::OnceLock;
use std::time::Duration;
use tempfile::TempDir;
use tokio::sync::Mutex;
use tokio::time::timeout;

static ENV_LOCK: OnceLock<Mutex<()>> = OnceLock::new();

fn env_lock() -> &'static Mutex<()> {
    ENV_LOCK.get_or_init(|| Mutex::new(()))
}

fn write_jsonl(path: &Path, lines: &[serde_json::Value]) {
    let mut out = String::new();
    for line in lines {
        out.push_str(&serde_json::to_string(line).unwrap());
        out.push('\n');
    }
    std::fs::write(path, out).unwrap();
}

struct EnvVarGuard {
    key: &'static str,
    original: Option<OsString>,
}

impl EnvVarGuard {
    fn set(key: &'static str, value: &Path) -> Self {
        let original = std::env::var_os(key);

        // `set_var` / `remove_var` are `unsafe` on Rust 2024 because mutating the
        // process environment can be undefined behavior if other threads call
        // into `getenv` concurrently. This test serializes environment mutation
        // with a global mutex and uses a single-threaded runtime.
        unsafe {
            std::env::set_var(key, value);
        }

        Self { key, original }
    }
}

impl Drop for EnvVarGuard {
    fn drop(&mut self) {
        match &self.original {
            Some(value) => unsafe {
                std::env::set_var(self.key, value);
            },
            None => unsafe {
                std::env::remove_var(self.key);
            },
        }
    }
}

async fn recv_until(
    rx: &mut aura::agents::codex::CodexEventRx,
    deadline: Duration,
    mut pred: impl FnMut(&AgentEvent) -> bool,
) -> Vec<AgentEvent> {
    let mut out = Vec::new();
    let start = tokio::time::Instant::now();

    loop {
        let Some(remaining) = deadline.checked_sub(start.elapsed()) else {
            return out;
        };
        if remaining.is_zero() {
            return out;
        }

        match timeout(remaining, rx.recv()).await {
            Ok(Some(ev)) => {
                let done = pred(&ev);
                out.push(ev);
                if done {
                    return out;
                }
            }
            Ok(None) => return out,
            Err(_) => return out,
        }
    }
}

#[tokio::test(flavor = "current_thread")]
async fn codex_bootstrap_emits_activity_from_existing_rollout() {
    let _guard = env_lock().lock().await;

    let tmp = TempDir::new().unwrap();
    let _env = EnvVarGuard::set("CODEX_HOME", tmp.path());

    let rollout_dir = tmp.path().join("sessions/2026/02/15");
    std::fs::create_dir_all(&rollout_dir).unwrap();

    let session_id = "019c5edf-c355-7071-b480-eb61d3aabee5";
    let rollout_path = rollout_dir.join(format!(
        "rollout-2026-02-15T10-17-28-{session_id}.jsonl"
    ));

    write_jsonl(
        &rollout_path,
        &[
            serde_json::json!({
                "timestamp":"2026-02-15T01:17:31.178Z",
                "type":"session_meta",
                "payload":{ "id": session_id, "cwd":"/tmp/project" }
            }),
            serde_json::json!({
                "timestamp":"2026-02-15T01:17:31.180Z",
                "type":"event_msg",
                "payload":{ "type":"task_started" }
            }),
            serde_json::json!({
                "timestamp":"2026-02-15T01:17:31.181Z",
                "type":"response_item",
                "payload":{ "type":"message", "role":"user", "content":[{"type":"text","text":"hello"}] }
            }),
        ],
    );

    let stream = aura::agents::codex::spawn();
    let mut rx = stream.subscribe();

    let events = recv_until(&mut rx, Duration::from_secs(2), |ev| {
        matches!(
            ev,
            AgentEvent::Activity { session_id: sid, .. } if sid == session_id
        )
    })
    .await;

    let saw_started = events.iter().any(|ev| match ev {
        AgentEvent::SessionStarted {
            session_id: sid,
            cwd,
            agent,
        } => sid == session_id && cwd == "/tmp/project" && agent == &AgentType::Codex,
        _ => false,
    });
    let saw_activity = events.iter().any(|ev| match ev {
        AgentEvent::Activity { session_id: sid, .. } => sid == session_id,
        _ => false,
    });

    assert!(saw_started, "expected SessionStarted for codex session");
    assert!(saw_activity, "expected Activity emitted from rollout bootstrap");
}
