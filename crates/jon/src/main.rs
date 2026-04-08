// Jon - Natural language interface for Joy and Jot
//
// Currently delegates to jon-cli (CLI mode). Future feature flags will add:
// - TUI mode (terminal UI with panels and interactive navigation)
// - Desktop app mode (native window via Tauri)

fn main() -> anyhow::Result<()> {
    jon_cli::run()
}
