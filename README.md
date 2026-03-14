# 🦀 Rust Terminal Shell

A Linux-like terminal shell implemented in pure Rust. It behaves similarly to a basic Unix shell — complete with a coloured prompt, command history, built-in commands, external process execution, and basic pipe support.

---

## ✨ Features

| Feature           | Details                                                 |
| ----------------- | ------------------------------------------------------- |
| Coloured prompt   | `user@host:~/cwd$` in green/blue                        |
| Built-in commands | `cd`, `pwd`, `history`, `help`, `exit`                  |
| External commands | Runs any binary found in `$PATH`                        |
| Pipe support      | `cmd1 \| cmd2 \| cmd3` chains                           |
| Command history   | Persistent across sessions (`~/.rust_terminal_history`) |
| Graceful errors   | Unknown commands show a friendly message                |
| Ctrl-C / Ctrl-D   | Ctrl-C prints a hint; Ctrl-D exits cleanly              |

---

## 🔧 Installing Rust

If you don't already have Rust installed, use the official installer **rustup**:

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

Follow the on-screen instructions, then reload your shell:

```bash
source "$HOME/.cargo/env"
```

Verify the installation:

```bash
rustc --version   # e.g. rustc 1.78.0 (...)
cargo --version   # e.g. cargo 1.78.0 (...)
```

> The project targets the **stable** toolchain. No nightly features are used.

---

## 🏗️ Building the Project

Clone or extract the project, then build it:

```bash
cd rust_terminal

# Debug build (faster compile, includes debug symbols)
cargo build

# Release build (optimised binary)
cargo build --release
```

The compiled binary will be at:

| Build   | Path                           |
| ------- | ------------------------------ |
| Debug   | `target/debug/rust_terminal`   |
| Release | `target/release/rust_terminal` |

---

## ▶️ Running the Terminal

```bash
# Run directly with Cargo (debug)
cargo run

# Or run the binary directly after building
./target/debug/rust_terminal

# Optimised release binary
./target/release/rust_terminal
```

You'll be greeted with a banner and the interactive prompt:

```
  ██████╗ ██╗   ██╗███████╗████████╗    ███████╗██╗  ██╗
  ██╔══██╗██║   ██║██╔════╝╚══██╔══╝    ██╔════╝██║  ██║
  ██████╔╝██║   ██║███████╗   ██║       ███████╗███████║
  ██╔══██╗██║   ██║╚════██║   ██║       ╚════██║██╔══██║
  ██║  ██║╚██████╔╝███████║   ██║       ███████║██║  ██║
  ╚═╝  ╚═╝ ╚═════╝ ╚══════╝   ╚═╝       ╚══════╝╚═╝  ╚═╝
  ...

  Rust Terminal Shell  v0.1.0
  Type 'help' for available commands, 'exit' to quit.

alice@hostname:~$
```

---

## 🗂️ Project Structure

```
rust_terminal/
├── Cargo.toml          # Package manifest & dependencies
├── README.md           # This file
└── src/
    ├── main.rs         # Entry point — initialises and runs the shell
    ├── shell.rs        # REPL loop, prompt building, rustyline integration
    ├── commands.rs     # Built-ins (cd, pwd, help) + external/pipe execution
    └── utils.rs        # Banner, path helpers, error formatting, unit tests
```

### Module responsibilities

**`main.rs`**  
Prints the welcome banner, constructs a `Shell` instance, and calls `Shell::run()`.

**`shell.rs`**

- Owns the `rustyline::DefaultEditor` for line-editing and history.
- Builds the coloured prompt string on every iteration.
- Dispatches input to built-ins or `commands::execute_external` / `commands::execute_pipeline`.

**`commands.rs`**

- `builtin_cd` — handles `cd`, `cd -`, `cd ~`, `cd <path>`.
- `builtin_pwd` — prints `env::current_dir()`.
- `builtin_help` — renders a formatted help table.
- `execute_external` — spawns a child process with inherited stdio.
- `execute_pipeline` — forks a chain of processes, wiring stdout→stdin.

**`utils.rs`**

- `print_banner` — ASCII art welcome screen.
- `shorten_home` — replaces `/home/user/…` with `~/…` in the prompt.
- `format_error` — consistent `rust_terminal: cmd: message` format.
- Unit tests for the above helpers.

---

## 📦 Dependencies

| Crate                                             | Version | Purpose                                    |
| ------------------------------------------------- | ------- | ------------------------------------------ |
| [`rustyline`](https://crates.io/crates/rustyline) | 14.x    | Readline-like input, history, key bindings |
| [`colored`](https://crates.io/crates/colored)     | 2.x     | ANSI colour output                         |
| [`whoami`](https://crates.io/crates/whoami)       | 1.x     | Current username & hostname for the prompt |

---

## 🛠️ Built-in Commands Reference

| Command         | Description                                            |
| --------------- | ------------------------------------------------------ |
| `cd [dir]`      | Change directory. No arg → `$HOME`; `-` → previous dir |
| `pwd`           | Print current working directory                        |
| `history`       | List session command history                           |
| `help`          | Show this help table inside the shell                  |
| `exit` / `quit` | Exit the shell                                         |

---

## 🔁 Pipe Examples

```bash
ls -la | grep .rs
ls | sort | uniq
cat /etc/passwd | grep root | wc -l
ps aux | grep cargo
```

---

## 🧪 Running Tests

```bash
cargo test
```

Unit tests for `utils.rs` are included and run with the standard Cargo test harness.

---

## 📝 Notes

- Command history is saved to `~/.rust_terminal_history` and loaded automatically on startup.
- The shell runs on any Linux or macOS system with a standard `$PATH`.
- Windows is not a target (Unix process model is used directly).
