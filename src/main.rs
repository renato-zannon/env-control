use std::{
    collections::HashSet,
    env,
    io::{self, prelude::*},
    iter::once,
    process::{self, Stdio},
};

use path_set::{PathSetIter, iter};

mod path_set;

type AppResult<T> = Result<T, Box<dyn std::error::Error>>;

type OwnedPathIter = PathSetIter<std::vec::IntoIter<String>>;

struct Changes {
    to_remove: HashSet<String>,
    to_append: OwnedPathIter,
    to_prepend: OwnedPathIter,
}

fn main() -> AppResult<()> {
    let matches = arg_matches();

    let var_name = matches
        .get_one::<String>("var-name")
        .map(|s| s.as_str())
        .unwrap_or("PATH");

    let current_path = env::var(var_name).unwrap_or_default();

    if let Some(("exec", exec_matches)) = matches.subcommand() {
        let changes = collect_changes(&matches);
        let new_value = build_new_value(&current_path, changes)?;
        call_child(exec_matches, var_name, &new_value)?;
    } else {
        let changes = collect_changes(&matches);
        print_new_value(&current_path, changes)?;
    }

    Ok(())
}

fn collect_changes(matches: &clap::ArgMatches) -> Changes {
    Changes {
        to_remove: path_iter(matches, "remove").collect(),
        to_append: path_iter(matches, "append"),
        to_prepend: path_iter(matches, "prepend"),
    }
}

fn build_new_value(current_path: &str, changes: Changes) -> AppResult<String> {
    let mut buffer = Vec::with_capacity(current_path.len());
    process_paths(&mut buffer, changes, current_path)?;
    Ok(String::from_utf8(buffer)?)
}

fn print_new_value(current_path: &str, changes: Changes) -> AppResult<()> {
    let new_value = build_new_value(current_path, changes)?;
    let mut stdout = io::stdout();
    writeln!(&mut stdout, "{}", new_value)?;
    Ok(())
}

fn arg_matches() -> clap::ArgMatches {
    use clap::{Arg, ArgAction, Command};

    Command::new("env-control")
        .author("Renato Zannon <renato@rrsz.com.br>")
        .about("PATH-like string manipulation utility")
        .version(env!("CARGO_PKG_VERSION"))
        .arg(
            Arg::new("var-name")
                .help("the PATH-like environment variable to manipulate. Defaults to $PATH")
                .num_args(0..=1)
                .index(1)
                .value_name("VAR"),
        )
        .arg(
            Arg::new("append")
                .long("append")
                .short('a')
                .help("Append this path to the variable")
                .value_name("PATH")
                .num_args(1)
                .action(ArgAction::Append),
        )
        .arg(
            Arg::new("prepend")
                .long("prepend")
                .short('p')
                .help("Prepend this path to the variable")
                .value_name("PATH")
                .num_args(1)
                .action(ArgAction::Append),
        )
        .arg(
            Arg::new("remove")
                .long("remove")
                .short('r')
                .help("Remove this path from the variable")
                .value_name("PATH")
                .num_args(1)
                .action(ArgAction::Append),
        )
        .subcommand(
            Command::new("exec")
                .about("Execute <cmd> with the modified var-name")
                .arg(
                    Arg::new("cmd")
                        .help("Command to execute")
                        .required(true)
                        .index(1),
                )
                .arg(
                    Arg::new("cmd-args")
                        .help("Arguments to <cmd>")
                        .num_args(0..)
                        .index(2)
                        .trailing_var_arg(true)
                        .allow_hyphen_values(true),
                ),
        )
        .get_matches()
}

fn call_child(exec_matches: &clap::ArgMatches, var_name: &str, new_value: &str) -> AppResult<()> {
    let cmd_name = exec_matches
        .get_one::<String>("cmd")
        .expect("command name is required");

    let cmd_args = exec_matches
        .get_many::<String>("cmd-args")
        .unwrap_or_default();

    process::Command::new(cmd_name)
        .env(var_name, new_value)
        .args(cmd_args)
        .stdin(Stdio::inherit())
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .spawn()
        .and_then(|mut child| child.wait())?;

    Ok(())
}

fn process_paths<W>(writer: &mut W, changes: Changes, current_path: &str) -> AppResult<()>
where
    W: Write,
{
    let Changes {
        to_append,
        to_prepend,
        to_remove,
    } = changes;

    let mut combined_paths = to_prepend
        .chain(iter(once(current_path)))
        .chain(to_append)
        .filter(move |path| !(path.trim().is_empty() || to_remove.contains(path)));

    let mut printed_paths: HashSet<String> = HashSet::new();

    if let Some(first_path) = combined_paths.next() {
        write!(writer, "{}", first_path)?;
        printed_paths.insert(first_path);
    } else {
        return Ok(());
    }

    for path in combined_paths {
        if printed_paths.contains(&path) {
            continue;
        }

        write!(writer, ":{}", path)?;
        printed_paths.insert(path);
    }

    Ok(())
}

fn path_iter(matches: &clap::ArgMatches, option: &str) -> OwnedPathIter {
    let values: Vec<String> = matches
        .get_many::<String>(option)
        .unwrap_or_default()
        .map(|s| s.to_string())
        .collect();

    iter(values)
}
