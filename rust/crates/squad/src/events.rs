//! Event logging for the squad pipeline.
//!
//! Every agent turn is:
//! 1. Printed to stdout with a colored prefix.
//! 2. Appended to `dev-squad.log` in the current working directory.
//!
//! The log format is plain-text, human-readable. Each entry:
//!   ```
//!   [2026-04-05T09:30:00Z] [PHASE] [AGENT]
//!   <message text>
//!   ---
//!   ```

use std::fmt::Write as FmtWrite;
use std::fs::OpenOptions;
use std::io::Write;
use std::path::PathBuf;
use std::time::SystemTime;

/// ANSI colors for each agent.
fn agent_color(agent: &str) -> &'static str {
    match agent {
        "A" => "\x1b[36m", // cyan — Planner
        "B" => "\x1b[35m", // magenta — Reviewer
        "C" => "\x1b[32m", // green — Coder
        "D" => "\x1b[33m", // yellow — Tester
        "S" => "\x1b[34m", // blue — Supervisor
        _ => "\x1b[0m",
    }
}

const RESET: &str = "\x1b[0m";
const DIM: &str = "\x1b[2m";

fn now_iso() -> String {
    // SystemTime to a rough ISO-8601 string without external deps.
    let secs = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();
    let s = secs % 60;
    let m = (secs / 60) % 60;
    let h = (secs / 3600) % 24;
    // Day/month/year approximation (good enough for log timestamps)
    let days = secs / 86400;
    let year = 1970 + days / 365;
    let day_of_year = days % 365;
    let month = day_of_year / 30 + 1;
    let day = day_of_year % 30 + 1;
    format!("{year:04}-{month:02}-{day:02}T{h:02}:{m:02}:{s:02}Z")
}

/// Append a structured entry to `dev-squad.log`.
fn append_to_log(log_path: &PathBuf, phase: &str, agent: &str, text: &str) {
    let mut file = match OpenOptions::new().create(true).append(true).open(log_path) {
        Ok(f) => f,
        Err(e) => {
            eprintln!("⚠️  Could not write to dev-squad.log: {e}");
            return;
        }
    };
    let entry = format!(
        "[{}] [{}] [Agent {}]\n{}\n---\n\n",
        now_iso(),
        phase,
        agent,
        text
    );
    let _ = file.write_all(entry.as_bytes());
}

/// Print a squad event to the terminal and append it to `dev-squad.log`.
///
/// `log_path` is the resolved path to `dev-squad.log` (computed once in the orchestrator).
pub fn emit(log_path: &PathBuf, phase: &str, agent: &str, text: &str) {
    let color = agent_color(agent);
    let phase_dim = format!("{DIM}[{phase}]{RESET}");

    // Pretty-print each line with the agent label prefix
    let mut display = String::new();
    for line in text.lines() {
        let _ = writeln!(display, "{color}[Agent {agent}]{RESET} {phase_dim} {line}");
    }
    if text.is_empty() {
        let _ = writeln!(
            display,
            "{color}[Agent {agent}]{RESET} {phase_dim} (empty response)"
        );
    }
    print!("{display}");

    append_to_log(log_path, phase, agent, text);
}

/// Print a supervisor/system message (no agent label, dim style).
pub fn emit_system(log_path: &PathBuf, phase: &str, text: &str) {
    println!("{DIM}[{phase}] ⚙ {text}{RESET}");
    append_to_log(log_path, phase, "system", text);
}

/// Write the squad header (task + model) to the log at run start.
pub fn write_header(log_path: &PathBuf, task: &str, model: &str) {
    let header = format!(
        "=== Dev Squad Run ===\nTask  : {task}\nModel : {model}\nStarted: {}\n\n",
        now_iso()
    );
    if let Ok(mut file) = OpenOptions::new().create(true).append(true).open(log_path) {
        let _ = file.write_all(header.as_bytes());
    }
    println!("\n{DIM}📋 Log → {}{RESET}", log_path.display());
}

/// Write a completion summary to the log.
pub fn write_footer(log_path: &PathBuf, success: bool, task: &str) {
    let status = if success {
        "✅ COMPLETED"
    } else {
        "❌ FAILED"
    };
    let footer = format!(
        "\n=== {status} ===\nTask: {task}\nFinished: {}\n",
        now_iso()
    );
    if let Ok(mut file) = OpenOptions::new().create(true).append(true).open(log_path) {
        let _ = file.write_all(footer.as_bytes());
    }
    if success {
        println!(
            "\n\x1b[32m{status}\x1b[0m — Dev Squad finished. Log at: {}",
            log_path.display()
        );
    } else {
        println!(
            "\n\x1b[31m{status}\x1b[0m — Dev Squad stopped. Log at: {}",
            log_path.display()
        );
    }
}
