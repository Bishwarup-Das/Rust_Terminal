// utils.rs — Utility / Helper Functions
//
// Miscellaneous helpers used across the shell:
//   * `print_banner`   — startup splash screen
//   * `shorten_home`   — replace the home prefix with `~`
//   * `format_error`   — consistent error message formatting

use colored::*;
use std::env;
use std::path::PathBuf;

// ── Banner ────────────────────────────────────────────────────────────────────

/// Print a welcome banner when the shell starts.
pub fn print_banner() {
    println!();
    println!("{}", r#"  ██████╗ ██╗   ██╗███████╗████████╗    ███████╗██╗  ██╗"#.cyan());
    println!("{}", r#"  ██╔══██╗██║   ██║██╔════╝╚══██╔══╝    ██╔════╝██║  ██║"#.cyan());
    println!("{}", r#"  ██████╔╝██║   ██║███████╗   ██║       ███████╗███████║"#.cyan());
    println!("{}", r#"  ██╔══██╗██║   ██║╚════██║   ██║       ╚════██║██╔══██║"#.cyan());
    println!("{}", r#"  ██║  ██║╚██████╔╝███████║   ██║       ███████║██║  ██║"#.cyan());
    println!("{}", r#"  ╚═╝  ╚═╝ ╚═════╝ ╚══════╝   ╚═╝       ╚══════╝╚═╝  ╚═╝"#.cyan());
    println!();
    println!(
        "  {}  {}",
        "Rust Terminal Shell".bold().white(),
        "v0.1.0".dimmed()
    );
    println!(
        "  {}",
        "Type 'help' for available commands, 'exit' to quit.".dimmed()
    );
    println!();
}

// ── Path helpers ──────────────────────────────────────────────────────────────

/// Replace the user's home directory prefix in `path` with `~`.
///
/// Example: `/home/alice/projects` → `~/projects`
pub fn shorten_home(path: PathBuf) -> String {
    let home = match env::var("HOME") {
        Ok(h) => h,
        Err(_) => return path.display().to_string(),
    };

    let path_str = path.display().to_string();

    if path_str.starts_with(&home) {
        // Replace only the leading home prefix
        format!("~{}", &path_str[home.len()..])
    } else {
        path_str
    }
}

// ── Error formatting ──────────────────────────────────────────────────────────

/// Format a shell error message consistently.
///
/// Output example:  `rust_terminal: ls: command not found`
pub fn format_error(cmd: &str, message: &str) -> String {
    format!(
        "{}: {}: {}",
        "rust_terminal".red().bold(),
        cmd.yellow(),
        message
    )
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn shorten_home_replaces_prefix() {
        // Temporarily override HOME for the test
        let fake_home = "/tmp/testuser";
        env::set_var("HOME", fake_home);

        let path = PathBuf::from("/tmp/testuser/projects/foo");
        let result = shorten_home(path);
        assert_eq!(result, "~/projects/foo");
    }

    #[test]
    fn shorten_home_leaves_other_paths() {
        env::set_var("HOME", "/home/alice");

        let path = PathBuf::from("/etc/passwd");
        let result = shorten_home(path);
        assert_eq!(result, "/etc/passwd");
    }

    #[test]
    fn format_error_contains_cmd_and_message() {
        // Strip ANSI codes for comparison — just check raw strings contain the words
        let msg = format_error("foobar", "not found");
        // The colored crate adds escape codes, so use contains()
        assert!(msg.contains("foobar"));
        assert!(msg.contains("not found"));
        assert!(msg.contains("rust_terminal"));
    }
}
