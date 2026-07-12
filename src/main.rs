// SPDX-FileCopyrightText: 2026 Marcus Baw and Baw Medical Ltd
// SPDX-License-Identifier: AGPL-3.0-or-later

//! The standalone `clincalc` binary.
//!
//! A thin wrapper over [`clincalc::cli`]: all calculator behaviour lives in the
//! library so host CLIs (e.g. GitEHR's `gitehr calc`) can reuse it without
//! repetition. The binary adds the top-level command structure, shell
//! completions, version command, MCP entrypoint, and legacy shorthand handling.

use std::env;
use std::ffi::{OsStr, OsString};
use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result, anyhow};
use clap::{Args, CommandFactory, Parser, Subcommand, ValueHint};
use clap_complete::{Shell, generate};

use clincalc::cli::{CalcCommand, ListCommand, TagsCommand, VersionCommand};

/// Open, auditable clinical calculators.
#[derive(Debug, Parser)]
#[command(
    name = "clincalc",
    version,
    about = "Open, auditable clinical calculators",
    long_about = "Open, auditable clinical calculators. One registry-backed CLI shape drives every calculator: list the catalogue, inspect a calculator's template/schema/licence, then compute from JSON.",
    subcommand_required = false,
    arg_required_else_help = false,
    after_long_help = "Examples:\n  clincalc list\n  clincalc ls --tag cardiology\n  clincalc calc feverpain\n  clincalc calc feverpain --input examples/feverpain.json\n  clincalc tags\n  clincalc completions install\n\nCompatibility: `clincalc <name>` is shorthand for `clincalc calc <name>`."
)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Debug, Subcommand)]
enum Commands {
    /// List available calculators.
    #[command(visible_alias = "ls")]
    List(ListCommand),

    /// Inspect or run one calculator.
    Calc(CalcCommand),

    /// List every tag in the registry, with calculator counts.
    Tags(TagsCommand),

    /// Print version information.
    Version(VersionCommand),

    /// Generate or install shell completions.
    Completions(CompletionsArgs),

    /// Start the local stdio MCP server when compiled with `--features mcp`.
    Mcp,
}

#[derive(Debug, Args)]
struct CompletionsArgs {
    #[command(subcommand)]
    command: Option<CompletionCommand>,

    /// Shell to generate completions for.
    shell: Option<Shell>,

    /// Output directory. Prints to stdout when omitted.
    #[arg(long, short = 'd', value_hint = ValueHint::DirPath, value_parser = tilde_pathbuf)]
    dir: Option<PathBuf>,
}

#[derive(Debug, Subcommand)]
enum CompletionCommand {
    /// Install completions for the current user.
    Install {
        /// Shell to install completions for. Detected from $SHELL when omitted.
        #[arg(long)]
        shell: Option<Shell>,

        /// Completion directory to write to.
        #[arg(long, short = 'd', value_hint = ValueHint::DirPath, value_parser = tilde_pathbuf)]
        dir: Option<PathBuf>,
    },
}

fn main() -> anyhow::Result<()> {
    // Reset SIGPIPE on Unix so output pipes cleanly into `head`/`less` instead
    // of producing a "Broken pipe" panic when the reader closes early. The
    // house style requires this for CLIs whose stdout is meant to be piped.
    #[cfg(unix)]
    // SAFETY: setting the default disposition for SIGPIPE in a single-threaded
    // start-up context is the canonical pattern; no signal handlers run yet.
    unsafe {
        libc_signal_default_sigpipe();
    }

    if let Some(version) = legacy_version_request() {
        println!("{version}");
        return Ok(());
    }

    let cli = Cli::parse_from(normalized_args());
    match cli.command {
        None => clincalc::cli::run_list(ListCommand {
            tag: Vec::new(),
            tags: false,
            format: Default::default(),
        }),
        Some(Commands::List(cmd)) => clincalc::cli::run_list(cmd),
        Some(Commands::Calc(cmd)) => clincalc::cli::run(cmd),
        Some(Commands::Tags(cmd)) => clincalc::cli::run_tags(cmd),
        Some(Commands::Version(cmd)) => clincalc::cli::run_version(cmd),
        Some(Commands::Completions(args)) => run_completions(args),
        Some(Commands::Mcp) => run_mcp(),
    }
}

/// Preserve the historical calculator shorthand while exposing real top-level
/// subcommands to clap, help output, and completions.
fn normalized_args() -> Vec<OsString> {
    let mut args: Vec<OsString> = env::args_os().collect();
    let Some(first) = args.get(1).cloned() else {
        return args;
    };

    if first == OsStr::new("--tags") {
        args[1] = OsString::from("tags");
        return args;
    }

    if is_known_top_command(&first) || starts_with_dash(&first) {
        return args;
    }

    args.insert(1, OsString::from("calc"));
    args
}

fn starts_with_dash(arg: &OsStr) -> bool {
    arg.to_string_lossy().starts_with('-')
}

fn is_known_top_command(arg: &OsStr) -> bool {
    matches!(
        arg.to_string_lossy().as_ref(),
        "calc" | "list" | "ls" | "tags" | "version" | "completions" | "mcp" | "help"
    )
}

fn legacy_version_request() -> Option<String> {
    let mut args = env::args_os();
    let _program = args.next()?;
    let arg = args.next()?;
    if args.next().is_some() {
        return None;
    }
    matches!(arg.to_string_lossy().as_ref(), "-v" | "-version")
        .then(|| format!("clincalc {}", env!("CARGO_PKG_VERSION")))
}

#[cfg(feature = "mcp")]
fn run_mcp() -> Result<()> {
    tokio::runtime::Runtime::new()?.block_on(clincalc::mcp::serve_stdio())
}

#[cfg(not(feature = "mcp"))]
fn run_mcp() -> Result<()> {
    Err(anyhow!(
        "MCP support was not compiled into this clincalc binary.\nReinstall with MCP enabled, for example: cargo install clincalc --features mcp"
    ))
}

fn run_completions(args: CompletionsArgs) -> Result<()> {
    let mut cmd = Cli::command();
    cmd.set_bin_name("clincalc");
    match args.command {
        Some(CompletionCommand::Install { shell, dir }) => {
            let shell = shell.or_else(detect_shell).ok_or_else(|| {
                anyhow!("could not detect shell; pass --shell bash|zsh|fish|powershell|elvish")
            })?;
            let dir = dir
                .map(Ok)
                .unwrap_or_else(|| default_completion_dir(shell))?;
            write_completion(shell, &mut cmd, &dir)?;
            print_install_note(shell, &dir);
        }
        None => {
            let shell = args
                .shell
                .ok_or_else(|| anyhow!("missing shell; try `clincalc completions install`"))?;
            if let Some(dir) = args.dir {
                write_completion(shell, &mut cmd, &dir)?;
            } else {
                generate(shell, &mut cmd, "clincalc", &mut std::io::stdout());
            }
        }
    }
    Ok(())
}

fn write_completion(shell: Shell, cmd: &mut clap::Command, dir: &Path) -> Result<PathBuf> {
    fs::create_dir_all(dir)
        .with_context(|| format!("creating completion directory {}", dir.display()))?;
    let path = dir.join(completion_filename(shell));
    let mut file = fs::File::create(&path)
        .with_context(|| format!("creating completion file {}", path.display()))?;
    generate(shell, cmd, "clincalc", &mut file);
    println!("Completion script written to: {}", path.display());
    Ok(path)
}

fn completion_filename(shell: Shell) -> &'static str {
    match shell {
        Shell::Bash => "clincalc",
        Shell::Zsh => "_clincalc",
        Shell::Fish => "clincalc.fish",
        Shell::PowerShell => "clincalc.ps1",
        Shell::Elvish => "clincalc.elv",
        _ => "clincalc.completion",
    }
}

fn detect_shell() -> Option<Shell> {
    let shell = env::var("SHELL").ok()?;
    let name = Path::new(&shell).file_name()?.to_string_lossy();
    match name.as_ref() {
        "bash" => Some(Shell::Bash),
        "zsh" => Some(Shell::Zsh),
        "fish" => Some(Shell::Fish),
        "elvish" => Some(Shell::Elvish),
        _ => None,
    }
}

fn default_completion_dir(shell: Shell) -> Result<PathBuf> {
    let home = home_dir().ok_or_else(|| anyhow!("could not determine home directory"))?;
    Ok(match shell {
        Shell::Bash => env::var_os("XDG_DATA_HOME")
            .map(PathBuf::from)
            .unwrap_or_else(|| home.join(".local/share"))
            .join("bash-completion/completions"),
        Shell::Zsh => home.join(".zfunc"),
        Shell::Fish => env::var_os("XDG_CONFIG_HOME")
            .map(PathBuf::from)
            .unwrap_or_else(|| home.join(".config"))
            .join("fish/completions"),
        Shell::PowerShell => home.join(".config/powershell/completions"),
        Shell::Elvish => home.join(".elvish/lib"),
        _ => home.join(".local/share/clincalc/completions"),
    })
}

fn tilde_pathbuf(src: &str) -> Result<PathBuf, String> {
    Ok(if src == "~" {
        home_dir().ok_or_else(|| "could not determine home directory".to_string())?
    } else if let Some(rest) = src.strip_prefix("~/") {
        home_dir()
            .ok_or_else(|| "could not determine home directory".to_string())?
            .join(rest)
    } else {
        PathBuf::from(src)
    })
}

fn home_dir() -> Option<PathBuf> {
    env::var_os("HOME")
        .or_else(|| env::var_os("USERPROFILE"))
        .map(PathBuf::from)
}

fn print_install_note(shell: Shell, dir: &Path) {
    match shell {
        Shell::Zsh => {
            println!("Add this before `compinit` in ~/.zshrc if it is not already there:");
            println!("  fpath=({} $fpath)", dir.display());
            println!("Then restart zsh or run `autoload -Uz compinit && compinit`.");
        }
        Shell::PowerShell => {
            println!("Add this to your PowerShell profile if it is not already there:");
            println!("  . {}/clincalc.ps1", dir.display());
        }
        _ => println!("Restart your shell to load the updated completions."),
    }
}

#[cfg(unix)]
unsafe fn libc_signal_default_sigpipe() {
    // Avoid pulling in the `libc` crate just for this: use the raw syscall via
    // the std-exposed signal number constants. `signal(SIGPIPE, SIG_DFL)`.
    unsafe extern "C" {
        fn signal(signum: i32, handler: usize) -> usize;
    }
    const SIGPIPE: i32 = 13;
    const SIG_DFL: usize = 0;
    unsafe {
        signal(SIGPIPE, SIG_DFL);
    }
}
