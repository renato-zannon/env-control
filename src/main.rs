#![feature(plugin)]

extern crate "rustc-serialize" as rustc_serialize;
extern crate docopt;
#[plugin] #[no_link] extern crate docopt_macros;

use std::collections::HashSet;

docopt!(Config derive Debug, "
Usage: env-control <string> [-a PATH | -p PATH | -r PATH]...

Options:
    -a, --append PATH   Append this path to the variable
    -p, --prepend PATH  Prepend this path to the variable
    -r, --remove PATH   Remove this path from the variable
    <string>            The PATH-like string to manipulate
", flag_remove: HashSet<String>);

fn main() {
    let cfg: Config = Config::docopt().decode().unwrap_or_else(|e| e.exit());
    let to_append    = cfg.flag_append;
    let to_prepend   = cfg.flag_prepend;
    let to_remove    = cfg.flag_remove;
    let current_path = cfg.arg_string;

    let mut combined_paths = to_prepend.into_iter()
        .chain(current_path.split(':').map(|slice| slice.to_string()))
        .chain(to_append.into_iter());

    let mut printed_paths: HashSet<String> = HashSet::new();

    match combined_paths.next() {
        Some(first_path) => {
            if !first_path.trim().is_empty() {
                print!("{}", first_path);
                printed_paths.insert(first_path);
            }
        },

        None => return,
    }

    for path in combined_paths {
        if path.trim().is_empty() || printed_paths.contains(&path) || to_remove.contains(&path) {
            continue
        }

        print!(":{}", path);
        printed_paths.insert(path);
    }

    print!("\n");
}
