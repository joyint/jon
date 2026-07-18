// Copyright (c) 2026 Joydev GmbH (joydev.com)
// SPDX-License-Identifier: MIT

use std::io::Write as _;
use std::path::Path;

use clap::{Args, Parser, Subcommand};
use jon_core::bootstrap::{self, BootstrapOptions, RepoState};
use jon_core::joy_core::ai_setup;

/// Jon - Natural language interface and product development assistant
#[derive(Parser)]
#[command(name = "jon", version, about)]
struct Cli {
    #[command(subcommand)]
    command: Option<Command>,

    /// A natural language query (tier 0 pattern router)
    query: Option<String>,
}

#[derive(Subcommand)]
enum Command {
    /// Bootstrap a Joy project from an idea: the guided PDA session
    Init(InitArgs),
}

#[derive(Args, Default)]
struct InitArgs {
    /// AI tool that runs the session locally (claude|qwen|vibe|copilot)
    #[arg(long)]
    tool: Option<String>,

    /// Project name (defaults to the directory name)
    #[arg(long)]
    name: Option<String>,

    /// Project language code (ISO 639-1, e.g. en, de)
    #[arg(long)]
    language: Option<String>,

    /// Founder e-mail override (defaults to git user.email)
    #[arg(long)]
    user: Option<String>,

    /// Accept defaults instead of prompting
    #[arg(short = 'y', long)]
    yes: bool,

    /// Prepare the project and print the session prompt, do not launch the agent
    #[arg(long)]
    no_launch: bool,
}

pub fn run() -> anyhow::Result<()> {
    let cli = Cli::parse();

    match (cli.command, cli.query) {
        (Some(Command::Init(args)), _) => init(args),
        (None, Some(query)) => {
            println!("Jon received: {query}");
            println!("Tier 0 pattern router not yet implemented.");
            Ok(())
        }
        (None, None) => {
            println!("Jon v{}", env!("CARGO_PKG_VERSION"));
            println!("Natural language interface for Joy and Jyn.");
            println!();
            println!("Usage: jon \"what's my next task?\"");
            println!("       jon init    bootstrap a project from an idea");
            Ok(())
        }
    }
}

fn init(args: InitArgs) -> anyhow::Result<()> {
    let root = std::env::current_dir()?;

    if bootstrap::inspect(&root) == RepoState::JoyProject {
        let project_path = jon_core::joy_core::store::joy_dir(&root)
            .join(jon_core::joy_core::store::PROJECT_FILE);
        let name = jon_core::joy_core::store::read_project(&project_path)
            .map(|p| p.name)
            .unwrap_or_else(|_| "unnamed".into());
        println!("This repository already is a Joy project ({name}).");
        println!("The PDA session bootstraps empty projects; continue with this one instead:");
        println!("  joy ai init     set up or refresh AI tooling");
        println!("  joy ls          see the backlog");
        println!("Or add it in the Joyint app with '+ Add'.");
        return Ok(());
    }

    let tool = resolve_tool(args.tool.as_deref(), args.yes)?;
    let default_name = root
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("project")
        .to_string();
    let name = match args.name {
        Some(n) => n,
        None if args.yes => default_name,
        None => prompt_line("Project name", &default_name)?,
    };
    let language = match args.language {
        Some(l) => l,
        None if args.yes => "en".into(),
        None => prompt_line("Project language (ISO 639-1)", "en")?,
    };

    println!(
        "Bootstrapping \"{name}\" for a PDA session with {}...",
        tool_label(&tool)
    );
    let mut report = |line: String| println!("  + {line}");
    let outcome = bootstrap::bootstrap(
        &BootstrapOptions {
            root: root.clone(),
            name: Some(name),
            user: args.user.clone(),
            language: Some(language),
            tool: tool.clone(),
        },
        &mut report,
    )?;
    if outcome.initialized {
        println!("  + .joy store initialized");
    }
    for doc in &outcome.docs_seeded {
        println!("  + {doc} scaffolded");
    }
    println!();

    if args.no_launch {
        println!("--- PDA session prompt ---");
        println!("{}", outcome.session_prompt);
        println!("--- end of session prompt ---");
        println!("Start your tool in this directory with the prompt above.");
        return Ok(());
    }
    launch(&root, &tool, &outcome.session_prompt)
}

fn tool_label(id: &str) -> &str {
    ai_setup::TOOLS
        .iter()
        .find(|(_, eid, _, _)| *eid == id)
        .map(|(label, _, _, _)| *label)
        .unwrap_or(id)
}

fn resolve_tool(flag: Option<&str>, yes: bool) -> anyhow::Result<String> {
    if let Some(id) = flag {
        if !ai_setup::TOOLS.iter().any(|(_, eid, _, _)| *eid == id) {
            let known: Vec<&str> = ai_setup::TOOLS.iter().map(|(_, id, _, _)| *id).collect();
            anyhow::bail!("unknown tool: {id} (known: {})", known.join(", "));
        }
        return Ok(id.to_string());
    }

    let installed: Vec<&str> = ai_setup::TOOLS
        .iter()
        .filter(|(_, _, detect, _)| detect())
        .map(|(_, id, _, _)| *id)
        .collect();
    let default = installed.first().copied().unwrap_or("claude");
    if yes {
        return Ok(default.to_string());
    }

    println!("Which AI tool runs the session?");
    for (label, id, detect, _) in ai_setup::TOOLS {
        let marker = if detect() { " (installed)" } else { "" };
        println!("  {id:<8} {label}{marker}");
    }
    prompt_line("Tool", default)
}

fn prompt_line(question: &str, default: &str) -> anyhow::Result<String> {
    print!("{question} [{default}]: ");
    std::io::stdout().flush()?;
    let mut buf = String::new();
    std::io::stdin().read_line(&mut buf)?;
    let answer = buf.trim();
    Ok(if answer.is_empty() {
        default.to_string()
    } else {
        answer.to_string()
    })
}

/// Hand the session to the local agent. Claude Code and Qwen Code take
/// an initial prompt on the command line and stay interactive; for the
/// others the prompt is printed to paste into the tool.
fn launch(root: &Path, tool: &str, prompt: &str) -> anyhow::Result<()> {
    let command: Option<(&str, Vec<&str>)> = match tool {
        "claude" => Some(("claude", vec![prompt])),
        "qwen" => Some(("qwen", vec!["-i", prompt])),
        _ => None,
    };

    let Some((program, args)) = command else {
        println!("--- PDA session prompt ---");
        println!("{prompt}");
        println!("--- end of session prompt ---");
        println!(
            "Start {} in this directory and paste the prompt above.",
            tool_label(tool)
        );
        return Ok(());
    };

    println!("Launching {} for the PDA session...", tool_label(tool));
    let status = std::process::Command::new(program)
        .args(&args)
        .current_dir(root)
        .status();
    match status {
        Ok(s) if s.success() => Ok(()),
        Ok(s) => anyhow::bail!("{program} exited with {s}"),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
            anyhow::bail!(
                "{program} is not installed or not on PATH. \
                 Re-run with --no-launch to get the session prompt."
            )
        }
        Err(e) => Err(e.into()),
    }
}
