#[macro_use]
extern crate clap;

mod path_set;

use std::io::prelude::*;
use std::os::unix::prelude::*;
use std::collections::HashSet;
use std::{env, process, io, vec};
use std::ffi::OsStr;
use std::iter::once;
use std::process::Stdio;

use path_set::{PathSetIter, iter};

struct Changes<'a, 'b> {
    to_remove: HashSet<String>,
    to_append: PathSetIter<vec::IntoIter<&'a str>>,
    to_prepend: PathSetIter<vec::IntoIter<&'b str>>,
}

fn main() {
    use clap::{App, Arg, SubCommand, AppSettings};

    let matches = App::new("env-control")
        .author("Renato Zannon <renato@rrsz.com.br>")
        .about("PATH-like string manipulation utility")
        .version(&crate_version!())
        .arg_from_usage("[var-name] 'the PATH-like environment variable to manipulate. \
                         Defaults to $PATH'")
        .arg(Arg::from_usage("-a --append=[append-paths]... 'Append this path to the variable'")
             .value_name("PATH"))
        .arg(Arg::from_usage("-p --prepend=[prepend-paths]... 'Prepend this path to the variable'")
             .value_name("PATH"))
        .arg(Arg::from_usage("-r --remove=[remove-paths]... 'Remove this path from the variable'")
             .value_name("PATH"))
        .subcommand(SubCommand::with_name("exec")
                    .about("Execute <cmd> with the modified var-name")
                    .arg_from_usage("<cmd> 'Command to execute'")
                    .arg_from_usage("[cmd-args]... 'Arguments to <cmd>'")
                    .setting(AppSettings::TrailingVarArg))
        .get_matches();

    let var_name = matches.value_of("var-name").unwrap_or("PATH");

    let current_path = match env::var(&var_name) {
        Ok(string) => string,
        Err(_)     => "".to_owned(),
    };

    let changes = Changes {
        to_remove:  iter(matches.values_of("remove-paths").unwrap_or(vec![])).collect(),
        to_append:  iter(matches.values_of("append-paths").unwrap_or(vec![])),
        to_prepend: iter(matches.values_of("prepend-paths").unwrap_or(vec![])),
    };

    if let Some(ref exec_matches) = matches.subcommand_matches("exec") {
        let mut new_value = Vec::with_capacity(current_path.len());
        process_paths(&mut new_value, changes, &current_path).unwrap();

        env::set_var(&var_name, OsStr::from_bytes(&new_value));

        let cmd_name = exec_matches.value_of("cmd").unwrap();
        let cmd_args = exec_matches.values_of("cmd-args").unwrap_or(vec![]);

        process::Command::new(cmd_name)
            .args(&cmd_args)
            .stdin(Stdio::inherit())
            .stdout(Stdio::inherit())
            .stderr(Stdio::inherit())
            .spawn()
            .and_then(|mut child| child.wait())
            .unwrap();
    } else {
        let mut stdout = io::stdout();
        process_paths(&mut stdout, changes, &current_path).unwrap();
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
