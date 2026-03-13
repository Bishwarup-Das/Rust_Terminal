// main.rs — Entry point for the Rust Terminal Shell
//
// This file initializes the shell environment and starts the main REPL loop.
// All heavy lifting is delegated to the `shell` module.

mod commands;
mod shell;
mod utils;

fn main() {
    // Print a welcome banner when the shell starts
    utils::print_banner();

    // Create and run the shell instance
    let mut sh = shell::Shell::new();
    sh.run();
}
