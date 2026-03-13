// commands.rs — Built-in Commands & External Process Execution
//
// Implements:
//   * `cd`, `pwd`, `help`   — pure Rust built-ins
//   * `execute_external`    — spawn any external binary with arguments
//   * `execute_pipeline`    — chain multiple commands with Unix pipes

use colored::*;
use std::env;
use std::io;
use std::process::{Command, Stdio};


// ── Built-in: cd ─────────────────────────────────────────────────────────────

/// Change the current working directory.
///
/// * No argument  → go to $HOME
/// * `cd -`       → go to the previous directory (stored in $OLDPWD)
/// * `cd <path>`  → change to the specified path
pub fn builtin_cd(args: &[&str]) {
    let new_dir: std::path::PathBuf = match args.first() {
        // No argument → $HOME
        None => {
            let home = env::var("HOME").unwrap_or_else(|_| "/".to_string());
            home.into()
        }

        // `cd -` → previous directory
        Some(&"-") => {
            match env::var("OLDPWD") {
                Ok(prev) => {
                    println!("{}", prev);  // bash behaviour: print the path
                    prev.into()
                }
                Err(_) => {
                    eprintln!("{}", "cd: OLDPWD not set".red());
                    return;
                }
            }
        }

        // Explicit path
        Some(path) => (*path).into(),
    };

    // Save current dir as OLDPWD before changing
    if let Ok(cwd) = env::current_dir() {
        env::set_var("OLDPWD", cwd);
    }

    // Perform the directory change
    if let Err(e) = env::set_current_dir(&new_dir) {
        eprintln!(
            "{}: {}: {}",
            "cd".red(),
            new_dir.display(),
            e.to_string().yellow()
        );
    }
}

// ── Built-in: pwd ─────────────────────────────────────────────────────────────

/// Print the current working directory.
pub fn builtin_pwd() {
    match env::current_dir() {
        Ok(path) => println!("{}", path.display()),
        Err(e) => eprintln!("{}: {}", "pwd".red(), e),
    }
}

// ── Built-in: help ────────────────────────────────────────────────────────────

/// Display a help message listing all built-in commands.
pub fn builtin_help() {
    println!();
    println!("{}", "╔══════════════════════════════════════╗".cyan());
    println!("{}", "║       Rust Terminal — Help           ║".cyan().bold());
    println!("{}", "╚══════════════════════════════════════╝".cyan());
    println!();
    println!("{}", "Built-in commands:".yellow().bold());
    println!();

    let builtins: &[(&str, &str)] = &[
        ("cd [dir]",   "Change directory (no arg → $HOME, - → previous)"),
        ("pwd",        "Print the current working directory"),
        ("history",    "Show command history for this session"),
        ("help",       "Show this help message"),
        ("exit / quit","Exit the shell"),
    ];

    for (name, desc) in builtins {
        println!("  {:<18} {}", name.green().bold(), desc);
    }

    println!();
    println!("{}", "External commands:".yellow().bold());
    println!("  Any binary found in $PATH is executed directly.");
    println!("  Example: ls -la, cat file.txt, grep pattern file");
    println!();
    println!("{}", "Piping:".yellow().bold());
    println!("  Chain commands with |");
    println!("  Example: ls -la | grep .rs | wc -l");
    println!();
}

// ── External command execution ────────────────────────────────────────────────

/// Spawn an external process, wait for it to finish, and return any error.
///
/// stdin/stdout/stderr are all inherited from the parent so the child's
/// output appears directly in the terminal.
pub fn execute_external(cmd: &str, args: &[&str]) -> io::Result<()> {
    let status = Command::new(cmd)
        .args(args)
        .stdin(Stdio::inherit())
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .spawn()
        .map_err(|e| {
            // Provide a friendlier error when the binary doesn't exist
            if e.kind() == io::ErrorKind::NotFound {
                io::Error::new(
                    io::ErrorKind::NotFound,
                    format!("command not found: {}", cmd),
                )
            } else {
                e
            }
        })?
        .wait()?;

    // If the child exited with a non-zero code, surface it (but don't abort)
    if !status.success() {
        if let Some(code) = status.code() {
            // Only print the exit-code hint for non-trivial failures
            if code != 1 {
                eprintln!("{} {}", "exit code:".dimmed(), code.to_string().yellow());
            }
        }
    }

    Ok(())
}

// ── Pipeline execution ────────────────────────────────────────────────────────

/// Execute a pipe-separated command string, e.g. `ls -la | grep .rs | wc -l`.
///
/// Approach:
///   1. Split on `|` to get individual command strings.
///   2. Parse each into (binary, args[]).
///   3. Spawn each process, connecting stdout → stdin down the chain.
///   4. Wait for all processes in order.
pub fn execute_pipeline(input: &str) -> io::Result<()> {
    // Split the pipeline into individual command strings
    let stages: Vec<&str> = input.split('|').map(str::trim).collect();

    if stages.is_empty() {
        return Ok(());
    }

    // Validate that every stage is non-empty
    for stage in &stages {
        if stage.is_empty() {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "empty command in pipeline",
            ));
        }
    }

    let mut children: Vec<std::process::Child> = Vec::with_capacity(stages.len());
    let mut previous_stdout: Option<std::process::ChildStdout> = None;

    for (i, stage) in stages.iter().enumerate() {
        // Tokenise the stage
        let parts: Vec<&str> = stage.split_whitespace().collect();
        if parts.is_empty() {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "empty stage in pipeline",
            ));
        }

        let cmd  = parts[0];
        let args = &parts[1..];
        let is_last = i == stages.len() - 1;

        // Determine stdin: inherit (first stage) or pipe from previous stage
        let stdin_cfg = match previous_stdout.take() {
            Some(stdout) => Stdio::from(stdout),
            None         => Stdio::inherit(),
        };

        // Determine stdout: pipe (not last) or inherit (last stage)
        let stdout_cfg = if is_last {
            Stdio::inherit()
        } else {
            Stdio::piped()
        };

        let mut child = Command::new(cmd)
            .args(args)
            .stdin(stdin_cfg)
            .stdout(stdout_cfg)
            .stderr(Stdio::inherit())
            .spawn()
            .map_err(|e| {
                if e.kind() == io::ErrorKind::NotFound {
                    io::Error::new(
                        io::ErrorKind::NotFound,
                        format!("command not found: {}", cmd),
                    )
                } else {
                    e
                }
            })?;

        // If this stage has a piped stdout, take it for the next stage
        if !is_last {
            previous_stdout = child.stdout.take();
        }

        children.push(child);
    }

    // Wait for all children in order, collecting the last exit status
    for mut child in children {
        let status = child.wait()?;
        if !status.success() {
            if let Some(code) = status.code() {
                if code != 1 {
                    eprintln!("{} {}", "exit code:".dimmed(), code.to_string().yellow());
                }
            }
        }
    }

    Ok(())
}
