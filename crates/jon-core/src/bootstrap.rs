// Copyright (c) 2026 Joydev GmbH (joydev.com)
// SPDX-License-Identifier: MIT

//! Non-interactive bootstrap engine for the PDA (JON-0006-34): bring a
//! repository from "nothing but an idea" to a Joy project with the
//! three document scaffolds, a configured AI tool, and the rendered
//! session prompt. jon-cli wraps this with prompts; the platform calls
//! it directly on a server-side checkout.
//!
//! Member registration for the tool is deliberately NOT part of the
//! bootstrap: a fresh project has no attestation chain yet. The
//! founder's identity and the tool member follow through `joy auth
//! init` / `joy ai init` (or the platform) once the session has
//! produced something worth governing.

use std::fs;
use std::path::{Path, PathBuf};

use joy_core::ai_setup;
use joy_core::vcs::Vcs;

use crate::error::JonError;
use crate::pda;

const VISION_SCAFFOLD: &str = include_str!("../data/docs/VISION.md");
const ARCHITECTURE_SCAFFOLD: &str = include_str!("../data/docs/ARCHITECTURE.md");
const CONTRIBUTING_SCAFFOLD: &str = include_str!("../data/docs/CONTRIBUTING.md");

/// What a directory is, from Jon's point of view.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RepoState {
    /// No git repository at this root.
    NoGit,
    /// A git repository without a `.joy/` store.
    GitOnly,
    /// An initialized Joy project.
    JoyProject,
}

/// Inspect a directory root. The `.joy` check is on the root itself,
/// not on the ancestors: `jon init` in a subdirectory of a Joy project
/// is answered by the caller walking up via
/// `joy_core::store::find_project_root`, not silently here.
pub fn inspect(root: &Path) -> RepoState {
    if joy_core::store::is_initialized(root) {
        RepoState::JoyProject
    } else if joy_core::vcs::default_vcs().is_repo(root) {
        RepoState::GitOnly
    } else {
        RepoState::NoGit
    }
}

pub struct BootstrapOptions {
    pub root: PathBuf,
    /// Project name; defaults to the directory name.
    pub name: Option<String>,
    /// Founder e-mail override; defaults to git user.email.
    pub user: Option<String>,
    /// Project language code (ISO 639-1); defaults to "en".
    pub language: Option<String>,
    /// Tool id that runs the session (claude|qwen|vibe|copilot).
    pub tool: String,
}

pub struct BootstrapOutcome {
    /// The `.joy` store was created by this call.
    pub initialized: bool,
    /// Doc paths (relative) seeded with a scaffold by this call.
    pub docs_seeded: Vec<String>,
    /// The tool's instruction files were written or refreshed.
    pub tool_configured: bool,
    /// The rendered PDA session prompt for this project.
    pub session_prompt: String,
}

/// Bootstrap a PDA-ready project. Idempotent: an already-initialized
/// store, already-seeded docs, and an already-configured tool are left
/// alone, and only the session prompt is re-rendered.
pub fn bootstrap(
    opts: &BootstrapOptions,
    report: ai_setup::Report,
) -> Result<BootstrapOutcome, JonError> {
    let root = &opts.root;

    if !ai_setup::TOOLS.iter().any(|(_, id, _, _)| *id == opts.tool) {
        return Err(JonError::Other(format!("unknown tool: {}", opts.tool)));
    }

    let mut initialized = false;
    if !joy_core::store::is_initialized(root) {
        joy_core::init::init(joy_core::init::InitOptions {
            root: root.clone(),
            name: opts.name.clone(),
            acronym: None,
            user: opts.user.clone(),
            language: opts.language.clone(),
        })?;
        initialized = true;
    }

    let project_path = joy_core::store::joy_dir(root).join(joy_core::store::PROJECT_FILE);
    let project = joy_core::store::read_project(&project_path)?;

    let vision_path = project.docs.vision_or_default().to_string();
    let architecture_path = project.docs.architecture_or_default().to_string();
    let contributing_path = project.docs.contributing_or_default().to_string();

    let mut docs_seeded = Vec::new();
    for (rel, scaffold) in [
        (&vision_path, VISION_SCAFFOLD),
        (&architecture_path, ARCHITECTURE_SCAFFOLD),
        (&contributing_path, CONTRIBUTING_SCAFFOLD),
    ] {
        let path = root.join(rel);
        if !path.is_file() {
            if let Some(parent) = path.parent() {
                fs::create_dir_all(parent).map_err(|e| JonError::Other(e.to_string()))?;
            }
            fs::write(&path, scaffold).map_err(|e| JonError::Other(e.to_string()))?;
            joy_core::git_ops::auto_git_add(root, &[rel]);
            docs_seeded.push(rel.clone());
        }
    }

    let tool_configured = ai_setup::configure_tool(root, &opts.tool, report)?;
    let configured: Vec<&str> = ai_setup::TOOLS
        .iter()
        .filter(|(_, id, _, _)| ai_setup::is_tool_configured(root, id))
        .map(|(_, id, _, _)| *id)
        .collect();
    ai_setup::update_gitignore(root, &configured)?;

    let session_prompt = pda::render_session(&pda::PdaContext {
        project_name: project.name.clone(),
        acronym: project.acronym.clone().unwrap_or_default(),
        language: project.language.clone(),
        tool: opts.tool.clone(),
        vision_path,
        architecture_path,
        contributing_path,
    })?;

    Ok(BootstrapOutcome {
        initialized,
        docs_seeded,
        tool_configured,
        session_prompt,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    fn opts(root: &Path) -> BootstrapOptions {
        BootstrapOptions {
            root: root.to_path_buf(),
            name: Some("Acme Notes".into()),
            user: Some("founder@example.com".into()),
            language: None,
            tool: "claude".into(),
        }
    }

    #[test]
    fn inspect_distinguishes_the_three_states() {
        let dir = tempdir().unwrap();
        assert_eq!(inspect(dir.path()), RepoState::NoGit);

        let mut report = |_line: String| {};
        bootstrap(&opts(dir.path()), &mut report).unwrap();
        assert_eq!(inspect(dir.path()), RepoState::JoyProject);
    }

    #[test]
    fn bootstrap_produces_a_pda_ready_project() {
        let dir = tempdir().unwrap();
        let mut lines = Vec::new();
        let mut report = |line: String| lines.push(line);
        let outcome = bootstrap(&opts(dir.path()), &mut report).unwrap();

        assert!(outcome.initialized);
        assert!(dir.path().join(".joy/project.yaml").is_file());
        // The three docs are seeded at their default paths.
        assert_eq!(outcome.docs_seeded.len(), 3);
        for doc in ["VISION.md", "ARCHITECTURE.md", "CONTRIBUTING.md"] {
            assert!(dir.path().join(doc).is_file(), "missing {doc}");
        }
        // The chosen tool is configured.
        assert!(outcome.tool_configured);
        assert!(dir.path().join(".claude/skills/joy/SKILL.md").is_file());
        // The session prompt is rendered for this project.
        assert!(outcome.session_prompt.contains("Acme Notes"));
    }

    #[test]
    fn bootstrap_is_idempotent_and_keeps_existing_docs() {
        let dir = tempdir().unwrap();
        let mut report = |_line: String| {};
        bootstrap(&opts(dir.path()), &mut report).unwrap();

        std::fs::write(dir.path().join("VISION.md"), "# My vision\n").unwrap();
        let second = bootstrap(&opts(dir.path()), &mut report).unwrap();

        assert!(!second.initialized);
        assert!(second.docs_seeded.is_empty());
        let vision = std::fs::read_to_string(dir.path().join("VISION.md")).unwrap();
        assert_eq!(vision, "# My vision\n");
    }

    #[test]
    fn bootstrap_rejects_unknown_tools() {
        let dir = tempdir().unwrap();
        let mut report = |_line: String| {};
        let mut o = opts(dir.path());
        o.tool = "cursor".into();
        assert!(bootstrap(&o, &mut report).is_err());
    }
}
