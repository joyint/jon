// Copyright (c) 2026 Joydev GmbH (joydev.com)
// SPDX-License-Identifier: MIT

//! Rendering for the PDA session instructions (JON-0007-9F). Same
//! technology as joy-core's AI tool templates (ADR-024): MiniJinja
//! templates plus structured YAML data, both embedded in the binary and
//! rendered straight to the consumer. The rendered session text is the
//! instruction prompt that drives the interview; jon-cli hands it to
//! the local agent, the platform seeds the "Jon PDA" chat with it.

use minijinja::{context, Environment};

use crate::error::JonError;

const SESSION_TMPL: &str = include_str!("../templates/pda/session.md");
const INTERVIEW_DATA: &str = include_str!("../data/pda/interview.yaml");

/// Hard cap for decision items produced by one PDA session. A handful
/// of real forks keeps the decision log readable; everything else is
/// prose in the architecture document (concept section 5.1).
pub const DECISION_CAP: u8 = 5;

/// Everything the session template needs to know about the project.
pub struct PdaContext {
    pub project_name: String,
    pub acronym: String,
    /// Project language code (ISO 639-1); all artifacts stay in it.
    pub language: String,
    /// Tool id running the session (claude|qwen|vibe|copilot).
    pub tool: String,
    pub vision_path: String,
    pub architecture_path: String,
    pub contributing_path: String,
}

/// Load the interview definition (stages and their questions).
pub fn load_interview() -> Result<serde_json::Value, JonError> {
    serde_yaml_ng::from_str(INTERVIEW_DATA).map_err(|e| JonError::Template(e.to_string()))
}

/// Render the full PDA session prompt for a project.
pub fn render_session(ctx: &PdaContext) -> Result<String, JonError> {
    let interview = load_interview()?;
    let mut env = Environment::new();
    env.add_template("pda-session", SESSION_TMPL)
        .map_err(|e| JonError::Template(e.to_string()))?;
    let tmpl = env
        .get_template("pda-session")
        .map_err(|e| JonError::Template(e.to_string()))?;
    let docs = context! {
        vision => ctx.vision_path,
        architecture => ctx.architecture_path,
        contributing => ctx.contributing_path,
    };
    let rendered = tmpl
        .render(context! {
            interview => interview,
            docs => docs,
            project_name => ctx.project_name,
            acronym => ctx.acronym,
            language => ctx.language,
            tool => ctx.tool,
            decision_cap => DECISION_CAP,
        })
        .map_err(|e| JonError::Template(e.to_string()))?;
    Ok(rendered)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ctx() -> PdaContext {
        PdaContext {
            project_name: "Acme Notes".into(),
            acronym: "AN".into(),
            language: "en".into(),
            tool: "claude".into(),
            vision_path: "VISION.md".into(),
            architecture_path: "ARCHITECTURE.md".into(),
            contributing_path: "CONTRIBUTING.md".into(),
        }
    }

    #[test]
    fn interview_has_the_three_doc_stages_in_order() {
        let interview = load_interview().unwrap();
        let stages = interview["stages"].as_array().unwrap();
        let keys: Vec<&str> = stages.iter().filter_map(|s| s["doc"].as_str()).collect();
        assert_eq!(keys, ["vision", "architecture", "contributing"]);
        for stage in stages {
            assert!(
                stage["questions"].as_array().is_some_and(|q| !q.is_empty()),
                "stage without questions"
            );
        }
    }

    #[test]
    fn session_renders_the_five_outputs_contract() {
        let rendered = render_session(&ctx()).unwrap();
        assert!(rendered.contains("Acme Notes"));
        // The three documents by their configured paths.
        assert!(rendered.contains("VISION.md"));
        assert!(rendered.contains("ARCHITECTURE.md"));
        assert!(rendered.contains("CONTRIBUTING.md"));
        // Decisions are capped, the closing creates exactly one task.
        assert!(rendered.contains(&format!("at most {DECISION_CAP} decision")));
        assert!(rendered.contains("exactly one task"));
        // Ask first, propose after: the standing rule must survive edits.
        assert!(rendered.contains("Ask first, propose after"));
        // No archetype or stack choice in the first engagement level.
        assert!(rendered.contains("archetype"));
    }

    #[test]
    fn session_renders_configured_doc_paths() {
        let mut c = ctx();
        c.vision_path = "docs/vision.md".into();
        let rendered = render_session(&c).unwrap();
        assert!(rendered.contains("docs/vision.md"));
    }
}
