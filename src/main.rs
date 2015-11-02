extern crate rustc_serialize;
extern crate docopt;

mod path_set;

use std::io::prelude::*;
use std::collections::HashSet;
use std::{env, process, io, vec};
use std::iter::once;
use std::process::Stdio;

use docopt::Docopt;

use path_set::{PathSetIter, iter};

const USAGE: &'static str = "
Usage: env-control <var-name> [-a PATH | -p PATH | -r PATH]...
       env-control exec <var-name> [-a PATH | -p PATH | -r PATH]... <cmd> [<cmd-args>]...

Options:
    -a, --append PATH   Append this path to the variable
    -p, --prepend PATH  Prepend this path to the variable
    -r, --remove PATH   Remove this path from the variable
    <var-name>          The PATH-like environment variable to manipulate
";

struct Changes {
    to_remove: HashSet<String>,
    to_append: PathSetIter<vec::IntoIter<String>>,
    to_prepend: PathSetIter<vec::IntoIter<String>>,
}

#[derive(Debug, Clone, RustcDecodable)]
struct Config {
    arg_cmd: String,
    arg_var_name: String,
    cmd_exec: bool,
    flag_remove: Vec<String>,
    arg_cmd_args: Vec<String>,
    flag_prepend: Vec<String>,
    flag_append: Vec<String>,
}

fn main() {
    let cfg: Config = Docopt::new(USAGE)
        .and_then(|docopt| docopt.decode())
        .unwrap_or_else(|e| e.exit());

    let current_path = match env::var(&cfg.arg_var_name[..]) {
        Ok(string) => string,
        Err(_)     => "".to_owned(),
    };

    let changes = Changes {
        to_remove: iter(cfg.flag_remove).collect(),
        to_append: iter(cfg.flag_append),
        to_prepend: iter(cfg.flag_prepend),
    };

    if cfg.cmd_exec {
        let mut buffer = Vec::with_capacity(current_path.len());
        process_paths(&mut buffer, changes, &current_path[..]).unwrap();

        env::set_var(&cfg.arg_var_name, &String::from_utf8(buffer).unwrap());

        process::Command::new(&cfg.arg_cmd[..])
            .args(&cfg.arg_cmd_args[..])
            .stdin(Stdio::inherit())
            .stdout(Stdio::inherit())
            .stderr(Stdio::inherit())
            .spawn()
            .unwrap();
    } else {
        let mut stdout = io::stdout();
        process_paths(&mut stdout, changes, &current_path[..]).unwrap();
        write!(&mut stdout, "\n").unwrap();
    }
}

fn process_paths<W>(writer: &mut W, changes: Changes, current_path: &str) -> Result<(), io::Error> where W: Write {
    let Changes { to_append, to_prepend, to_remove } = changes;

    let mut combined_paths = to_prepend
        .chain(iter(once(current_path)))
        .chain(to_append)
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
