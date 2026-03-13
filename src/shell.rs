// shell.rs — Core Shell Loop
//
// Implements the main Read-Eval-Print Loop (REPL) using `rustyline` for
// rich line editing, persistent command history, and Ctrl-C/Ctrl-D handling.

use colored::*;
use rustyline::error::ReadlineError;
use rustyline::{DefaultEditor};
use std::env;
use std::path::PathBuf;

use crate::commands;
use crate::utils;

/// Path to the history file stored in the user's home directory.
const HISTORY_FILE: &str = ".rust_terminal_history";

/// The Shell struct holds all mutable state for the shell session.
pub struct Shell {
    /// rustyline editor — handles input, history, and key bindings.
    editor: DefaultEditor,

    /// Tracks whether the shell should keep running.
    pub running: bool,

    /// Path to the history file.
    history_path: PathBuf,
}

impl Shell {
    /// Create a new Shell instance, initialising rustyline and loading history.
    pub fn new() -> Self {
        let mut editor = DefaultEditor::new().expect("Failed to initialise line editor");

        // Resolve the history file path: $HOME/.rust_terminal_history
        let history_path = dirs_home().join(HISTORY_FILE);

        // Load previous session's history (ignore errors if file doesn't exist)
        let _ = editor.load_history(&history_path);

        Shell {
            editor,
            running: true,
            history_path,
        }
    }

    /// Run the main REPL loop.
    pub fn run(&mut self) {
        while self.running {
            // Build the coloured prompt string: user@host:cwd$
            let prompt = build_prompt();

            // Read a line of input from the user
            match self.editor.readline(&prompt) {
                Ok(line) => {
                    let trimmed = line.trim().to_string();

                    // Skip blank lines
                    if trimmed.is_empty() {
                        continue;
                    }

                    // Record the command in history
                    let _ = self.editor.add_history_entry(&trimmed);

                    // Dispatch the command
                    self.execute(&trimmed);
                }

                // Ctrl-C pressed — print a hint rather than exiting immediately
                Err(ReadlineError::Interrupted) => {
                    println!("{}", "^C  (type 'exit' to quit)".dimmed());
                }

                // Ctrl-D pressed — treat as 'exit'
                Err(ReadlineError::Eof) => {
                    println!("{}", "exit".cyan());
                    self.running = false;
                }

                // Any other readline error is fatal
                Err(err) => {
                    eprintln!("{} {}", "readline error:".red(), err);
                    self.running = false;
                }
            }
        }

        // Persist history before quitting
        let _ = self.editor.save_history(&self.history_path);
        println!("{}", "Goodbye!".cyan().bold());
    }

    /// Parse and execute a raw input line.
    ///
    /// Supports:
    ///   * Pipe chains:  `cmd1 | cmd2 | cmd3`
    ///   * Built-in commands handled directly in Rust
    ///   * External process execution via `std::process::Command`
    fn execute(&mut self, input: &str) {
        // ── Pipe detection ────────────────────────────────────────────────────
        // If the input contains a `|` we hand the entire string off to the
        // pipe handler which forks a chain of processes.
        if input.contains('|') {
            if let Err(e) = commands::execute_pipeline(input) {
                eprintln!("{} {}", "pipe error:".red(), e);
            }
            return;
        }

        // ── Tokenise ──────────────────────────────────────────────────────────
        let parts: Vec<&str> = input.split_whitespace().collect();
        if parts.is_empty() {
            return;
        }

        let cmd = parts[0];
        let args = &parts[1..];

        // ── Built-in commands ─────────────────────────────────────────────────
        match cmd {
            "exit" | "quit" => {
                self.running = false;
            }

            "cd" => {
                commands::builtin_cd(args);
            }

            "pwd" => {
                commands::builtin_pwd();
            }

            "help" => {
                commands::builtin_help();
            }

            "history" => {
                // Print the in-memory history via the rustyline editor
                for (i, entry) in self.editor.history().iter().enumerate() {
                    println!("{:>4}  {}", i + 1, entry);
                }
            }

            // ── External commands ─────────────────────────────────────────────
            _ => {
                if let Err(e) = commands::execute_external(cmd, args) {
                    eprintln!("{}", utils::format_error(cmd, &e.to_string()));
                }
            }
        }
    }
}

// ── Helpers ──────────────────────────────────────────────────────────────────

/// Build a coloured shell prompt: `user@host:cwd$ `
fn build_prompt() -> String {
    let user = whoami::username();
    let host = whoami::fallible::hostname().unwrap_or_else(|_| "localhost".into());

    // Get current working directory; fall back gracefully
    let cwd = env::current_dir()
        .map(|p| utils::shorten_home(p))
        .unwrap_or_else(|_| "?".to_string());

    // Compose: green user@host, blue cwd, white $
    format!(
        "{}{}{}{}{}",
        format!("{}@{}", user, host).green().bold(),
        ":".white(),
        cwd.blue().bold(),
        "$ ".white().bold(),
        ""  // rustyline appends cursor after this
    )
}

/// Return the user's home directory as a `PathBuf`.
fn dirs_home() -> PathBuf {
    env::var("HOME")
        .map(PathBuf::from)
        .unwrap_or_else(|_| PathBuf::from("."))
}
