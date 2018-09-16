#[macro_use]
extern crate clap;
#[macro_use]
extern crate quicli;
use quicli::prelude::*;

mod path_set;

use std::io::prelude::*;
use std::os::unix::prelude::*;
use std::collections::HashSet;
use std::{env, process, io};
use std::ffi::OsStr;
use std::iter::once;
use std::process::Stdio;
use std::default::Default;

use path_set::{PathSetIter, iter};

struct Changes<'a, 'b> {
    to_remove: HashSet<String>,
    to_append: PathSetIter<clap::Values<'a>>,
    to_prepend: PathSetIter<clap::Values<'b>>,
}

main!({
    let matches = arg_matches();

    let var_name = matches.value_of("var-name").unwrap_or("PATH");

    let current_path = match env::var(var_name) {
        Ok(string) => string,
        Err(_)     => String::new(),
    };

    let changes = Changes {
        to_remove:  path_iter(&matches, "remove").collect(),
        to_append:  path_iter(&matches, "append"),
        to_prepend: path_iter(&matches, "prepend"),
    };

    if let Some(exec_matches) = matches.subcommand_matches("exec") {
        set_new_value_on_env(var_name, current_path, changes)?;
        call_child(exec_matches)?;
    } else {
        print_new_value(current_path, changes)?;
    }
});

fn set_new_value_on_env(var_name: &str, current_path: String, changes: Changes) -> Result<()> {
    let mut new_value = Vec::with_capacity(current_path.len());
    process_paths(&mut new_value, changes, &current_path)?;

    env::set_var(&var_name, OsStr::from_bytes(&new_value));
    Ok(())
}

fn print_new_value(current_path: String, changes: Changes) -> Result<()> {
    let mut stdout = io::stdout();
    process_paths(&mut stdout, changes, &current_path)?;

    write!(&mut stdout, "\n")?;
    Ok(())
}

fn arg_matches() -> clap::ArgMatches<'static> {
    use clap::{App, Arg, SubCommand, AppSettings};

    return App::new("env-control")
        .author("Renato Zannon <renato@rrsz.com.br>")
        .about("PATH-like string manipulation utility")
        .version(crate_version!())
        .arg(Arg::with_name("var-name")
             .help("the PATH-like environment variable to manipulate. Defaults to $PATH")
             .required(false)
             .index(1)
         )
        .args_from_usage("
            -a --append=[PATH]...  'Append this path to the variable'
            -p --prepend=[PATH]... 'Prepend this path to the variable'
            -r --remove=[PATH]...  'Remove this path from the variable'
        ")
        .subcommand(SubCommand::with_name("exec")
                    .about("Execute <cmd> with the modified var-name")
                    .arg_from_usage("<cmd> 'Command to execute'")
                    .arg_from_usage("[cmd-args]... 'Arguments to <cmd>'")
                    .setting(AppSettings::TrailingVarArg))
        .get_matches();
}

fn call_child(exec_matches: &clap::ArgMatches) -> Result<()> {
    let cmd_name = exec_matches.value_of("cmd").unwrap();
    let cmd_args = exec_matches.values_of("cmd-args").unwrap_or_else(Default::default);

    process::Command::new(cmd_name)
        .args(cmd_args)
        .stdin(Stdio::inherit())
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .spawn()
        .and_then(|mut child| child.wait())?;

    Ok(())
}

fn process_paths<W>(writer: &mut W, changes: Changes, current_path: &str) -> Result<()> where W: Write {
    let Changes { to_append, to_prepend, to_remove } = changes;

    let mut combined_paths = to_prepend
        .chain(iter(once(current_path)))
        .chain(to_append)
        .filter(move |path| {
            !(path.trim().is_empty() || to_remove.contains(path))
        });

    let mut printed_paths: HashSet<String> = HashSet::new();

    if let Some(first_path) = combined_paths.next() {
        write!(writer, "{}", first_path)?;
        printed_paths.insert(first_path);
    } else {
        return Ok(());
    }

    for path in combined_paths {
        if printed_paths.contains(&path) {
            continue
        }

        write!(writer, ":{}", path)?;
        printed_paths.insert(path);
    }

    Ok(())
}

fn path_iter<'a>(matches: &'a clap::ArgMatches, option: &str) -> PathSetIter<clap::Values<'a>> {
    return iter(matches.values_of(option).unwrap_or_else(Default::default));
}
