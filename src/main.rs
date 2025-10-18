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

struct Changes<'a, 'b> {
    to_remove: HashSet<String>,
    to_append: PathSetIter<clap::Values<'a>>,
    to_prepend: PathSetIter<clap::Values<'b>>,
}

fn main() -> AppResult<()> {
    let matches = arg_matches();

    let var_name = matches.value_of("var-name").unwrap_or("PATH");

    let current_path = env::var(var_name).unwrap_or_default();

    if let Some(exec_matches) = matches.subcommand_matches("exec") {
        let changes = collect_changes(&matches);
        let new_value = build_new_value(&current_path, changes)?;
        call_child(exec_matches, var_name, &new_value)?;
    } else {
        let changes = collect_changes(&matches);
        print_new_value(&current_path, changes)?;
    }

    Ok(())
}

fn collect_changes<'a>(matches: &'a clap::ArgMatches) -> Changes<'a, 'a> {
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

fn arg_matches() -> clap::ArgMatches<'static> {
    use clap::{App, AppSettings, Arg, SubCommand, crate_version};

    App::new("env-control")
        .author("Renato Zannon <renato@rrsz.com.br>")
        .about("PATH-like string manipulation utility")
        .version(crate_version!())
        .arg(
            Arg::with_name("var-name")
                .help("the PATH-like environment variable to manipulate. Defaults to $PATH")
                .required(false)
                .index(1),
        )
        .args_from_usage(
            "
            -a --append=[PATH]...  'Append this path to the variable'
            -p --prepend=[PATH]... 'Prepend this path to the variable'
            -r --remove=[PATH]...  'Remove this path from the variable'
        ",
        )
        .subcommand(
            SubCommand::with_name("exec")
                .about("Execute <cmd> with the modified var-name")
                .arg_from_usage("<cmd> 'Command to execute'")
                .arg_from_usage("[cmd-args]... 'Arguments to <cmd>'")
                .setting(AppSettings::TrailingVarArg),
        )
        .get_matches()
}

fn call_child(
    exec_matches: &clap::ArgMatches,
    var_name: &str,
    new_value: &str,
) -> AppResult<()> {
    let cmd_name = exec_matches.value_of("cmd").unwrap();
    let cmd_args = exec_matches
        .values_of("cmd-args")
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

fn path_iter<'a>(matches: &'a clap::ArgMatches, option: &str) -> PathSetIter<clap::Values<'a>> {
    iter(matches.values_of(option).unwrap_or_default())
}
