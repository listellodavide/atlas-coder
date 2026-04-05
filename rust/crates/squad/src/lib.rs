//! # Squad — 5-Agent Dev Pipeline via Ollama
//!
//! Replicates the `the-dev-squad` TypeScript orchestrator in Rust.
//! Agents communicate in-memory; all activity is logged to `dev-squad.log`
//! in the current working directory.
//!
//! Pipeline phases:
//!   Phase 1 — Planning  : Agent A writes a `plan.md`
//!   Phase 2 — Review    : Agent B audits; A ↔ B loop until B approves
//!   Phase 3 — Coding    : Agent C implements the approved plan
//!   Phase 4 — Testing   : Agent D reviews; C ↔ D loop until D passes
//!
//! Usage (invoked from the CLI via `/squad <task>`):
//!   ```no_run
//!   # use squad::SquadOrchestrator;
//!   # tokio_test::block_on(async {
//!   let mut orc = SquadOrchestrator::new("ollama/qwen2.5-coder:14b", "Add a Fibonacci function").unwrap();
//!   orc.run("Add a Fibonacci function").await.unwrap();
//!   # });
//!   ```

#![allow(clippy::module_name_repetitions)]

pub mod agent;
pub mod events;
pub mod orchestrator;
pub mod roles;

pub use orchestrator::SquadOrchestrator;
