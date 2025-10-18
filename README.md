# env-control

Silly little utility for manipulating `PATH`-like strings

## Features
- Prepend, append, or remove entries on any PATH-style variable with a single command
- Split colon-delimited values automatically so `-p ~/.local/bin:~/bin` just works
- Drop empty segments and keep only the first copy of each entry to avoid duplicates
- Includes the `exec` subcommand to run one-off commands with the tweaked variable
- Fits nicely into shell scripts or dotfiles without more hand-written plumbing

## Installation

```bash
cargo install --path . --locked --profile release
```

You'll want a recent stable Rust toolchain (tested on 1.90). Cargo installs the binary into `~/.cargo/bin/` unless you've told it otherwise.

Prefer managing binaries yourself? Build locally and copy it wherever you keep the rest of your tools:

```bash
cargo build --release
install -Dm755 target/release/env-control ~/.local/bin/env-control
```

## Usage

```
env-control [VAR] [OPTIONS]
env-control [VAR] [OPTIONS] exec <cmd> [-- <args>...]
```

- `VAR` defaults to `PATH`. Pass another variable name (like `LD_LIBRARY_PATH`) if you want to modify something else

### Flags

- `-a, --append <PATH>` Add entries to the end of the variable
- `-p, --prepend <PATH>` Add entries to the beginning of the variable
- `-r, --remove <PATH>` Remove matching entries from the variable
- Repeat any flag as needed. Values containing `:` are split into individual entries automatically

### Subcommands

- `exec <cmd> [-- <args>...]` Apply the changes, then run a command with the updated variable

## Examples

Prepend `~/bin`, append `~/.rbenv/bin`, and print the resulting `PATH`:

```bash
env-control PATH -p ~/bin -a ~/.rbenv/bin
```

Tidy up `LD_LIBRARY_PATH`, then run `env` to inspect the result:

```bash
env-control LD_LIBRARY_PATH -r /usr/local/lib -a ~/.local/lib exec env
```

Add multiple colon-separated values in one go while dropping `/usr/bin`:

```bash
env-control -p ~/.cargo/bin:/opt/bin -r /usr/bin
```

## Notes

- Empty segments are ignored; later duplicates are skipped
- If the variable didn't exist, `env-control` starts from an empty string and builds from there
