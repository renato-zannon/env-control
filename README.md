# env-control

Silly utility for manipulating `PATH`-like strings

## Building

Run `cargo build --release` on the root directory, and copy the `target/release/env-control` executable to somewhere
on your default `$PATH` (like `/usr/local/bin`, for example).

## Example of usage

On your `.bashrc`, use this to prepend or append atoms to your `$PATH` or `$LD_LIBRARY_PATH`:

```bash
# Prepend ~/bin to your PATH, and append ~/.rbenv/bin
export PATH=$(env-control PATH -p ~/bin -a ~/.rbenv/bin)

# Remove /usr/local/lib from your $LD_LIBRARY_PATH, and append ~/.local/lib
export LD_LIBRARY_PATH=$(env-control LD_LIBRARY_PATH -r /usr/local/lib -a ~/.local/lib)
```

## Motivation

I'm tired of manipulating the `PATH` using shell script. And Rust is cool.
