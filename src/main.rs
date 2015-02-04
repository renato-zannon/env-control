#![feature(plugin, io, core, env)]

extern crate "rustc-serialize" as rustc_serialize;
extern crate docopt;
#[plugin] #[no_link] extern crate docopt_macros;

use std::collections::HashSet;
use std::old_io::{process, IoError, stdio};
use std::old_io::process::StdioContainer;
use std::env;

docopt!(Config derive Debug Clone, "
Usage: env-control <var-name> [-a PATH | -p PATH | -r PATH]...
       env-control exec <var-name> [-a PATH | -p PATH | -r PATH]... <cmd> [<cmd-args>]...

Options:
    -a, --append PATH   Append this path to the variable
    -p, --prepend PATH  Prepend this path to the variable
    -r, --remove PATH   Remove this path from the variable
    <var-name>          The PATH-like environment variable to manipulate
", flag_remove: HashSet<String>);

struct Changes {
    to_remove: HashSet<String>,
    to_append: Vec<String>,
    to_prepend: Vec<String>,
}

fn main() {
    let cfg: Config = Config::docopt().decode().unwrap_or_else(|e| e.exit());

    let current_path = match env::var_string(&cfg.arg_var_name[]) {
        Ok(string) => string,
        Err(_)     => "".to_string(),
    };

    let changes = Changes {
        to_remove: cfg.flag_remove,
        to_append: cfg.flag_append,
        to_prepend: cfg.flag_prepend,
    };

    if cfg.cmd_exec {
        let mut buffer = Vec::with_capacity(current_path.len());
        process_paths(&mut buffer, changes, &current_path[]).unwrap();

        process::Command::new(&cfg.arg_cmd[])
            .args(&cfg.arg_cmd_args[])
            .env(cfg.arg_var_name, buffer)
            .stdin(StdioContainer::InheritFd(0))
            .stdout(StdioContainer::InheritFd(1))
            .stderr(StdioContainer::InheritFd(2))
            .spawn()
            .unwrap();
    } else {
        let mut stdout = stdio::stdout();
        process_paths(&mut stdout, changes, &current_path[]).unwrap();
        write!(&mut stdout, "\n").unwrap();
    }
}

fn process_paths<W>(writer: &mut W, changes: Changes, current_path: &str) -> Result<(), IoError> where W: Writer {
    let Changes { to_append, to_prepend, to_remove } = changes;

    let mut combined_paths = to_prepend.into_iter()
        .chain(current_path.split(':').map(|slice| slice.to_string()))
        .chain(to_append.into_iter())
        .filter(move |path| {
            !(path.trim().is_empty() || to_remove.contains(path))
        });

    let mut printed_paths: HashSet<String> = HashSet::new();

    if let Some(first_path) = combined_paths.next() {
        try!(write!(writer, "{}", first_path));
        printed_paths.insert(first_path);
    } else {
        return Ok(());
    }

    for path in combined_paths {
        if printed_paths.contains(&path) {
            continue
        }

        try!(write!(writer, ":{}", path));
        printed_paths.insert(path);
    }

    Ok(())
}
