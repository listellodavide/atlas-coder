//! `SquadOrchestrator` — the main pipeline state machine.
//!
//! Runs all four phases sequentially, using `OllamaAgent` for each agent and
//! `events` for logging. Agents communicate in-memory (text strings passed
//! directly as function arguments), never polling disk. The approved `plan.md`
//! is written to disk once (for observability) after Phase 2 completes.

use std::env;
use std::fs;
use std::path::PathBuf;

use crate::agent::OllamaAgent;
use crate::events;
use crate::roles;

/// Maximum Q&A rounds between A and B before giving up.
const MAX_REVIEW_ROUNDS: usize = 5;
/// Maximum fix rounds between C and D before giving up.
const MAX_FIX_ROUNDS: usize = 5;
/// Maximum planning turns for Agent A before giving up.
const MAX_PLANNING_TURNS: usize = 3;

/// Signal tokens agents embed in their responses so the orchestrator can
/// detect phase completion without an extra LLM call.
const PLAN_READY: &str = "PLAN_READY";
const PLAN_APPROVED: &str = "PLAN_APPROVED";
const QUESTIONS_PENDING: &str = "QUESTIONS_PENDING";
const CODE_READY: &str = "CODE_READY";
const FIXES_APPLIED: &str = "FIXES_APPLIED";
const ALL_PASS: &str = "ALL_PASS";
const ISSUES_FOUND: &str = "ISSUES_FOUND";

pub struct SquadOrchestrator {
    agent_a: OllamaAgent,
    agent_b: OllamaAgent,
    agent_c: OllamaAgent,
    agent_d: OllamaAgent,
    model: String,
    log_path: PathBuf,
}

impl SquadOrchestrator {
    /// Create a new orchestrator.
    ///
    /// `model` is the Ollama model string (may include `ollama/` prefix, e.g.
    /// `ollama/qwen2.5-coder:14b` or just `qwen2.5-coder:14b`).
    ///
    /// # Errors
    /// Returns a `Box<dyn std::error::Error>` if the log file cannot be initialised.
    pub fn new(model: &str, task: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let log_path = env::current_dir()
            .unwrap_or_else(|_| PathBuf::from("."))
            .join("dev-squad.log");

        events::write_header(&log_path, task, model);

        Ok(Self {
            agent_a: OllamaAgent::new(model, roles::ROLE_A),
            agent_b: OllamaAgent::new(model, roles::ROLE_B),
            agent_c: OllamaAgent::new(model, roles::ROLE_C),
            agent_d: OllamaAgent::new(model, roles::ROLE_D),
            model: model.to_string(),
            log_path,
        })
    }

    // ─── Phase 1: Planning ──────────────────────────────────────────────────

    /// Agent A writes the plan. Returns the plan text.
    async fn phase_planning(&mut self, task: &str) -> Result<String, Box<dyn std::error::Error>> {
        events::emit_system(
            &self.log_path,
            "planning",
            &format!("🗺  Phase 1 — Planning (Agent A, model: {})", self.model),
        );

        let initial_prompt = format!(
            "New task for the team:\n\n{task}\n\n\
             Research the task thoroughly, then write a complete build plan. \
             Cover every file to create or modify, full implementation steps, and testing steps. \
             When your plan is complete and ready for review, end your response with `{PLAN_READY}`."
        );

        let mut plan_text = String::new();

        for turn in 1..=MAX_PLANNING_TURNS {
            let prompt = if turn == 1 {
                initial_prompt.clone()
            } else {
                format!(
                    "Your plan is almost there but you did not include `{PLAN_READY}`. \
                     Please finalise the plan and end your response with `{PLAN_READY}` when ready."
                )
            };

            let response = self.agent_a.turn(&prompt).await?;
            events::emit(&self.log_path, "planning", "A", &response);

            plan_text = response.clone();

            if response.contains(PLAN_READY) {
                break;
            }

            if turn == MAX_PLANNING_TURNS {
                events::emit_system(
                    &self.log_path,
                    "planning",
                    &format!("⚠️  Agent A reached {MAX_PLANNING_TURNS} turns without {PLAN_READY}. Proceeding with last output."),
                );
            }
        }

        self.agent_a.reset();
        Ok(plan_text)
    }

    // ─── Phase 2: Plan Review ───────────────────────────────────────────────

    /// Agent B reviews the plan. If B has questions, A answers; loop until
    /// B sends `PLAN_APPROVED`. Returns the final approved plan text.
    async fn phase_review(
        &mut self,
        plan_text: String,
    ) -> Result<String, Box<dyn std::error::Error>> {
        events::emit_system(
            &self.log_path,
            "review",
            "🔍  Phase 2 — Plan Review (Agents A ↔ B)",
        );

        let mut current_plan = plan_text;

        // First: Agent B reads the plan
        let b_initial = format!(
            "Here is the plan from Agent A:\n\n{current_plan}\n\n\
             Review the entire plan. If you have any gaps, questions, or concerns, \
             list them and end with `{QUESTIONS_PENDING}`. \
             If the plan is complete and correct, approve it and end with `{PLAN_APPROVED}`."
        );

        let mut b_response = self.agent_b.turn(&b_initial).await?;
        events::emit(&self.log_path, "review", "B", &b_response);

        for round in 0..MAX_REVIEW_ROUNDS {
            if b_response.contains(PLAN_APPROVED) {
                events::emit_system(&self.log_path, "review", "✅  Plan approved by Agent B.");
                break;
            }

            // B has questions — send them to A
            events::emit_system(
                &self.log_path,
                "review",
                &format!("💬  Review round {}/{MAX_REVIEW_ROUNDS}: B has questions, routing to A.", round + 1),
            );

            let a_prompt = format!(
                "Agent B (Plan Reviewer) has questions about your plan:\n\n{b_response}\n\n\
                 Answer each question with verified reasoning. Update and restate the complete \
                 plan incorporating your answers. End with `{PLAN_READY}` when done."
            );

            let a_response = self.agent_a.turn(&a_prompt).await?;
            events::emit(&self.log_path, "review", "A", &a_response);
            current_plan = a_response.clone();

            let b_follow_up = format!(
                "Agent A has updated the plan:\n\n{a_response}\n\n\
                 Re-review. Do you have any remaining concerns? \
                 If yes, list them and end with `{QUESTIONS_PENDING}`. \
                 If the plan is now complete, approve it and end with `{PLAN_APPROVED}`."
            );

            b_response = self.agent_b.turn(&b_follow_up).await?;
            events::emit(&self.log_path, "review", "B", &b_response);

            if b_response.contains(PLAN_APPROVED) {
                events::emit_system(&self.log_path, "review", "✅  Plan approved by Agent B.");
                break;
            }

            if round + 1 == MAX_REVIEW_ROUNDS {
                events::emit_system(
                    &self.log_path,
                    "review",
                    &format!("⚠️  Max review rounds ({MAX_REVIEW_ROUNDS}) reached. Proceeding with last plan."),
                );
            }
        }

        // Write the approved plan to disk (observability)
        let plan_path = env::current_dir()
            .unwrap_or_else(|_| PathBuf::from("."))
            .join("plan.md");
        if let Ok(clean_plan) = self.strip_signal_tokens(&current_plan) {
            let _ = fs::write(&plan_path, &clean_plan);
            events::emit_system(
                &self.log_path,
                "review",
                &format!("📄  Approved plan written to {}", plan_path.display()),
            );
        }

        self.agent_a.reset();
        self.agent_b.reset();
        Ok(current_plan)
    }

    // ─── Phase 3: Coding ────────────────────────────────────────────────────

    /// Agent C implements the approved plan. Returns C's implementation summary.
    async fn phase_coding(
        &mut self,
        plan_text: &str,
    ) -> Result<String, Box<dyn std::error::Error>> {
        events::emit_system(
            &self.log_path,
            "coding",
            "⌨️   Phase 3 — Coding (Agent C)",
        );

        let prompt = format!(
            "Here is your approved plan:\n\n{plan_text}\n\n\
             Implement every item exactly as specified. Show the complete code for every file \
             you create or modify. When your implementation is complete, end with `{CODE_READY}`."
        );

        let mut response = self.agent_c.turn(&prompt).await?;
        events::emit(&self.log_path, "coding", "C", &response);

        if !response.contains(CODE_READY) {
            let follow_up = format!(
                "Please complete the implementation and end your response with `{CODE_READY}` \
                 to signal you are done."
            );
            let follow_response = self.agent_c.turn(&follow_up).await?;
            events::emit(&self.log_path, "coding", "C", &follow_response);
            response = follow_response;
        }

        self.agent_c.reset();
        Ok(response)
    }

    // ─── Phase 4: Testing ───────────────────────────────────────────────────

    /// Agent D reviews and tests C's code. If D finds issues, C fixes; loop
    /// until D sends `ALL_PASS`. Returns the final result summary.
    async fn phase_testing(
        &mut self,
        plan_text: &str,
        code_text: &str,
    ) -> Result<String, Box<dyn std::error::Error>> {
        events::emit_system(
            &self.log_path,
            "testing",
            "🧪  Phase 4 — Testing (Agents C ↔ D)",
        );

        let d_initial = format!(
            "Here is the approved plan:\n\n{plan_text}\n\n\
             Here is Agent C's implementation:\n\n{code_text}\n\n\
             Review the code against the plan and verify correctness. \
             If everything is correct, end with `{ALL_PASS}`. \
             If you find issues, describe each one specifically and end with `{ISSUES_FOUND}`."
        );

        let mut d_response = self.agent_d.turn(&d_initial).await?;
        events::emit(&self.log_path, "testing", "D", &d_response);

        let mut current_code = code_text.to_string();

        for round in 0..MAX_FIX_ROUNDS {
            if d_response.contains(ALL_PASS) {
                events::emit_system(&self.log_path, "testing", "✅  All tests passed. Agent D approved.");
                break;
            }

            events::emit_system(
                &self.log_path,
                "testing",
                &format!("🔧  Fix round {}/{MAX_FIX_ROUNDS}: D found issues, routing to C.", round + 1),
            );

            let c_prompt = format!(
                "Agent D (Code Reviewer + Tester) found issues:\n\n{d_response}\n\n\
                 Fix every issue described above. Show the corrected code for each file. \
                 End with `{FIXES_APPLIED}` when all fixes are done."
            );

            let c_response = self.agent_c.turn(&c_prompt).await?;
            events::emit(&self.log_path, "testing", "C", &c_response);
            current_code = c_response.clone();

            let d_follow = format!(
                "Agent C has applied fixes:\n\n{c_response}\n\n\
                 Re-review. If everything now passes, end with `{ALL_PASS}`. \
                 If there are still issues, describe them and end with `{ISSUES_FOUND}`."
            );

            d_response = self.agent_d.turn(&d_follow).await?;
            events::emit(&self.log_path, "testing", "D", &d_response);

            if d_response.contains(ALL_PASS) {
                events::emit_system(&self.log_path, "testing", "✅  All tests passed.");
                break;
            }

            if round + 1 == MAX_FIX_ROUNDS {
                events::emit_system(
                    &self.log_path,
                    "testing",
                    &format!("⚠️  Max fix rounds ({MAX_FIX_ROUNDS}) reached."),
                );
            }
        }

        self.agent_c.reset();
        self.agent_d.reset();
        Ok(current_code)
    }

    // ─── Public entry-point ──────────────────────────────────────────────────

    /// Run the full 4-phase pipeline for `task`.
    ///
    /// All agent communication is in-memory. The approved plan is written to
    /// `./plan.md` for observability. All activity is logged to `dev-squad.log`.
    ///
    /// # Errors
    /// Propagates any Ollama API errors.
    pub async fn run(&mut self, task: &str) -> Result<(), Box<dyn std::error::Error>> {
        // ── Phase 1: Planning
        let plan_text = match self.phase_planning(task).await {
            Ok(p) => p,
            Err(e) => {
                events::emit_system(&self.log_path, "planning", &format!("❌  Phase 1 failed: {e}"));
                events::write_footer(&self.log_path, false, task);
                return Err(e);
            }
        };

        // ── Phase 2: Review
        let approved_plan = match self.phase_review(plan_text).await {
            Ok(p) => p,
            Err(e) => {
                events::emit_system(&self.log_path, "review", &format!("❌  Phase 2 failed: {e}"));
                events::write_footer(&self.log_path, false, task);
                return Err(e);
            }
        };

        // ── Phase 3: Coding
        let code_text = match self.phase_coding(&approved_plan).await {
            Ok(c) => c,
            Err(e) => {
                events::emit_system(&self.log_path, "coding", &format!("❌  Phase 3 failed: {e}"));
                events::write_footer(&self.log_path, false, task);
                return Err(e);
            }
        };

        // ── Phase 4: Testing
        match self.phase_testing(&approved_plan, &code_text).await {
            Ok(_) => {}
            Err(e) => {
                events::emit_system(&self.log_path, "testing", &format!("❌  Phase 4 failed: {e}"));
                events::write_footer(&self.log_path, false, task);
                return Err(e);
            }
        }

        events::write_footer(&self.log_path, true, task);
        Ok(())
    }

    // ─── Helpers ────────────────────────────────────────────────────────────

    /// Remove internal signal tokens from text before writing to disk.
    fn strip_signal_tokens(&self, text: &str) -> Result<String, Box<dyn std::error::Error>> {
        let tokens = [
            PLAN_READY,
            PLAN_APPROVED,
            QUESTIONS_PENDING,
            CODE_READY,
            FIXES_APPLIED,
            ALL_PASS,
            ISSUES_FOUND,
        ];
        let mut result = text.to_string();
        for token in tokens {
            result = result.replace(token, "");
        }
        Ok(result.trim().to_string())
    }
}
