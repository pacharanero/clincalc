// SPDX-FileCopyrightText: 2026 Marcus Baw and Baw Medical Ltd
// SPDX-License-Identifier: AGPL-3.0-or-later

//! The standalone `clincalc` binary.
//!
//! A thin wrapper over [`clincalc::cli`]: all behaviour lives in the library so
//! host CLIs (e.g. GitEHR's `gitehr calc`) can reuse it without repetition.

use std::env;
use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result, anyhow};
use clap::{Args, CommandFactory, Parser, Subcommand};
use clap_complete::{Shell, generate};

use clincalc::cli::CalcCommand;

/// Open clinical calculators - scoring at the command line.
#[derive(Debug, Parser)]
#[command(
    name = "clincalc",
    version,
    about,
    long_about = None,
    // Surface the `completions` side-channel in `--help`. It is dispatched
    // before the main clap parser in `main()` (because `name` is a
    // positional that would otherwise swallow the word "completions"), so
    // it is not a clap subcommand and would not appear here otherwise.
    after_long_help = "Shell completions:\n  \
        clincalc completions install                  Install for the current shell\n  \
        clincalc completions <bash|zsh|fish|...>      Print to stdout\n  \
        clincalc completions --dir <DIR> <SHELL>      Write to a specific dir\n\n\
See `clincalc completions --help` and `docs/cli-reference.md` for the full surface."
)]
struct Cli {
    #[command(flatten)]
    command: CalcCommand,
}

#[derive(Debug, Parser)]
#[command(name = "clincalc completions")]
struct CompletionsCli {
    #[command(flatten)]
    args: CompletionsArgs,
}

#[derive(Debug, Subcommand)]
enum TopCommand {
    /// Generate or install shell completions.
    Completions(CompletionsArgs),
}

#[derive(Debug, Args)]
struct CompletionsArgs {
    #[command(subcommand)]
    command: Option<CompletionCommand>,

    /// Shell to generate completions for.
    shell: Option<Shell>,

    /// Output directory. Prints to stdout when omitted.
    #[arg(long, short = 'd')]
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
        #[arg(long, short = 'd')]
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

    let mut args = env::args_os();
    let program = args.next();
    match args.next().as_deref() {
        Some(arg) if arg == std::ffi::OsStr::new("completions") => {
            let parse_args = program.into_iter().chain(args);
            return run_completions(CompletionsCli::parse_from(parse_args).args);
        }
        Some(arg) if arg == std::ffi::OsStr::new("mcp") => return run_mcp(),
        _ => {}
    }

    let cli = Cli::parse();
    clincalc::cli::run(cli.command)
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
    let mut cmd = TopCommand::augment_subcommands(Cli::command());
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
