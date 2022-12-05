extern crate crossbeam_channel as channel;
use std::{env, io, time::Instant};

use docopt::Docopt;
use serde::Deserialize;

use crate::clitypes::{CliError, CliResult, QsvExitCode};

#[cfg(feature = "mimalloc")]
#[global_allocator]
static GLOBAL: mimalloc::MiMalloc = mimalloc::MiMalloc;

macro_rules! command_list {
    () => {
        "
    count       Count records
    dedup       Remove redundant rows
    excel       Exports an Excel sheet to a CSV
    exclude     Excludes the records in one CSV from another
    frequency   Show frequency tables
    headers     Show header names
    help        Show this usage message
    index       Create CSV index for faster access
    input       Read CSVs w/ special quoting, skipping, trimming & transcoding rules
    luau        Execute Luau script on CSV data
    pseudo      Pseudonymise the values of a column
    rename      Rename the columns of CSV data efficiently
    replace     Replace patterns in CSV data
    safenames   Modify a CSV's header names to db-safe names
    sample      Randomly sample CSV data
    search      Search CSV data with a regex
    searchset   Search CSV data with a regex set
    select      Select, re-order, duplicate or drop columns
    slice       Slice records from CSV
    sniff       Quickly sniff CSV metadata
    sort        Sort CSV data in alphabetical, numerical, reverse or random order
    sortcheck   Check if a CSV is sorted
    stats       Infer data types and compute descriptive statistics
    validate    Validate CSV data for RFC4180-compliance or with JSON Schema

    NOTE: qsvdp ignores the --progressbar option for all commands.

    sponsored by datHere - Data Infrastructure Engineering
"
    };
}
mod clitypes;
mod cmd;
mod config;
mod index;
mod select;
mod util;

static USAGE: &str = concat!(
    "
Usage:
    qsvdp <command> [<args>...]
    qsvdp [options]

Options:
    --list               List all commands available.
    --envlist            List all qsv-relevant environment variables.
    -u, --update         Check for the latest qsv release.
    -h, --help           Display this message
    <command> -h         Display the command help message
    -v, --version        Print version info, mem allocator, features installed, 
                         max_jobs, num_cpus then exit

* sponsored by datHere - Data Infrastructure Engineering
"
);
#[derive(Deserialize)]
struct Args {
    arg_command:  Option<Command>,
    flag_list:    bool,
    flag_envlist: bool,
    flag_update:  bool,
}

fn main() -> QsvExitCode {
    let now = Instant::now();
    let qsv_args = util::init_logger();

    let args: Args = Docopt::new(USAGE)
        .and_then(|d| {
            d.options_first(true)
                .version(Some(util::version()))
                .deserialize()
        })
        .unwrap_or_else(|e| e.exit());
    if args.flag_list {
        wout!(concat!("Installed commands:", command_list!()));
        util::log_end(qsv_args, now);
        return QsvExitCode::Good;
    } else if args.flag_envlist {
        util::show_env_vars();
        util::log_end(qsv_args, now);
        return QsvExitCode::Good;
    }
    if args.flag_update {
        let update_checked = util::qsv_check_for_update(false);
        util::log_end(qsv_args, now);
        if update_checked.is_ok() {
            return QsvExitCode::Good;
        }
        return QsvExitCode::Bad;
    }
    match args.arg_command {
        None => {
            werr!(concat!(
                "qsvdp is a suite of CSV command line utilities optimized for Datapusher+.

Please choose one of the following commands:",
                command_list!()
            ));
            _ = util::qsv_check_for_update(true);
            util::log_end(qsv_args, now);
            QsvExitCode::Good
        }
        Some(cmd) => match cmd.run() {
            Ok(()) => {
                util::log_end(qsv_args, now);
                QsvExitCode::Good
            }
            Err(CliError::Flag(err)) => {
                werr!("{err}");
                util::log_end(qsv_args, now);
                QsvExitCode::IncorrectUsage
            }
            Err(CliError::Csv(err)) => {
                werr!("{err}");
                util::log_end(qsv_args, now);
                QsvExitCode::Bad
            }
            Err(CliError::Io(ref err)) if err.kind() == io::ErrorKind::BrokenPipe => {
                werr!("Broken pipe: {err}");
                util::log_end(qsv_args, now);
                QsvExitCode::Abort
            }
            Err(CliError::Io(err)) => {
                werr!("{err}");
                util::log_end(qsv_args, now);
                QsvExitCode::Bad
            }
            Err(CliError::NoMatch()) => {
                util::log_end(qsv_args, now);
                QsvExitCode::Bad
            }
            Err(CliError::Other(msg)) => {
                werr!("{msg}");
                util::log_end(qsv_args, now);
                QsvExitCode::Bad
            }
        },
    }
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "lowercase")]
enum Command {
    Count,
    Dedup,
    Excel,
    Exclude,
    Frequency,
    Headers,
    Help,
    Index,
    Input,
    Luau,
    Pseudo,
    Rename,
    Replace,
    Safenames,
    Sample,
    Search,
    SearchSet,
    Select,
    Slice,
    Sniff,
    Sort,
    SortCheck,
    Stats,
    Validate,
}

impl Command {
    fn run(self) -> CliResult<()> {
        let argv: Vec<_> = env::args().collect();
        let argv: Vec<_> = argv.iter().map(|s| &**s).collect();
        let argv = &*argv;

        if !argv[1].chars().all(char::is_lowercase) {
            return Err(CliError::Other(format!(
                "qsvdp expects commands in lowercase. Did you mean '{}'?",
                argv[1].to_lowercase()
            )));
        }
        match self {
            Command::Count => cmd::count::run(argv),
            Command::Dedup => cmd::dedup::run(argv),
            Command::Excel => cmd::excel::run(argv),
            Command::Exclude => cmd::exclude::run(argv),
            Command::Frequency => cmd::frequency::run(argv),
            Command::Headers => cmd::headers::run(argv),
            Command::Help => {
                wout!("{USAGE}");
                _ = util::qsv_check_for_update(true);
                Ok(())
            }
            Command::Index => cmd::index::run(argv),
            Command::Input => cmd::input::run(argv),
            Command::Luau => cmd::luau::run(argv),
            Command::Pseudo => cmd::pseudo::run(argv),
            Command::Rename => cmd::rename::run(argv),
            Command::Replace => cmd::replace::run(argv),
            Command::Safenames => cmd::safenames::run(argv),
            Command::Sample => cmd::sample::run(argv),
            Command::Search => cmd::search::run(argv),
            Command::SearchSet => cmd::searchset::run(argv),
            Command::Select => cmd::select::run(argv),
            Command::Slice => cmd::slice::run(argv),
            Command::Sniff => cmd::sniff::run(argv),
            Command::Sort => cmd::sort::run(argv),
            Command::SortCheck => cmd::sortcheck::run(argv),
            Command::Stats => cmd::stats::run(argv),
            Command::Validate => cmd::validate::run(argv),
        }
    }
}
