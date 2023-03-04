static USAGE: &str = r#"
Create a new computed column, filter rows or compute aggregations by executing a
Luau script for every row of a CSV file.

The executed Luau has 3 ways to reference row columns (as strings):
  1. Directly by using column name (e.g. Amount), can be disabled with -g
  2. Indexing col variable by column name: col.Amount or col["Total Balance"]
  3. Indexing col variable by column 1-based index: col[1], col[2], etc.

Of course, if your input has no headers, then 3. will be the only available
option.

Some usage examples:

  Sum numeric columns 'a' and 'b' and call new column 'c'
  $ qsv luau map c "a + b"
  $ qsv luau map c "col.a + col['b']"
  $ qsv luau map c "col[1] + col[2]"

  There is some magic in the previous example as 'a' and 'b' are passed in
  as strings (not numbers), but Luau still manages to add them up.
  A more explicit way of doing it, is by using tonumber
  $ qsv luau map c "tonumber(a) + tonumber(b)"

  Add running total column for Amount
  $ qsv luau map Total -x "tot = (tot or 0) + Amount; return tot"

  Or use the --begin and --end options to compute the running & grand totals
  $ qsv luau map Total --begin "tot = 0; gtotal = 0" -x \
        "tot = tot + Amount; gtotal = gtotal + tot; return tot" --end "return gtotal"

  Add running total column for Amount when previous balance was 900
  $ qsv luau map Total -x "tot = (tot or 900) + Amount; return tot"

  Convert Amount to always-positive AbsAmount and Type (debit/credit) columns
  $ qsv luau map Type -x \
        "if tonumber(Amount) < 0 then return 'debit' else return 'credit' end" | \
    qsv luau map AbsAmount "math.abs(tonumber(Amount))"

  Map multiple new columns in one pass
  $ qsv luau map newcol1,newcol2,newcol3 "{cola + 1, colb + 2, colc + 3}"

  Filter some rows based on numerical filtering
  $ qsv luau filter "tonumber(a) > 45"
  $ qsv luau filter "tonumber(a) >= tonumber(b)"

  Typing long scripts on the command line gets tiresome rather quickly, so use the
  "file:" prefix to read non-trivial scripts from the filesystem.
  $ qsv luau map Type -B "file:init.luau" -x "file:debitcredit.luau" -E "file:end.luau"

The MAIN script is evaluated on a per row basis.
With "luau map", if the main-script is invalid for a row, "<ERROR>" is returned for that row.
With "luau filter", if the main-script is invalid for a row, that row is not filtered.

If any row has an invalid result, an exitcode of 1 is returned and an error count is logged.

There are also special variables - "_IDX" - a read-only variable that is zero during the BEGIN script and
set to the current row number during the main script; & "_ROWCOUNT" - a read-only variable which is zero
during the BEGIN & MAIN scripts, and set to the rowcount during the END script.

"_INDEX" - a read/write variable that enables random access of the CSV file. Setting it to a row number
will change the current row to that row number. It will only work, however, if the CSV has an index.

When using _INDEX, the MAIN script will keep looping and evaluate the row specified by _INDEX
until _INDEX is set to an invalid row number (e.g. negative number or to a value greater than rowcount).

Luau's standard library is relatively minimal (https://luau-lang.org/library).
That's why qsv preloads the LuaDate library as date manipulation is a common data-wrangling task.
See https://tieske.github.io/date/#date-id96473 for info on how to use the LuaDate library.

Additional libraries can be loaded from the LUAU_PATH using luau's "require" function.
See http://lua-users.org/wiki/LibrariesAndBindings for a list of other libraries.

With the judicious use of "require", the BEGIN script & the "_IDX"/"_ROWCOUNT" variables, one can create
variables/tables/arrays that can be used for complex aggregation operations in the END script.

TIP: When developing Luau scripts, be sure to take advantage of the "qsv_log" function.
It will log messages to the logfile at the specified log level. Be sure to set QSV_LOG_LEVEL to the
appropriate log level. The first parameter to qsv_log is the log level (info, warn, error, debug, trace).
You can add as many as 255 addl parameters which will be concatenated and logged as a single message.

For more detailed examples, see https://github.com/jqnatividad/qsv/blob/master/tests/test_luau.rs.

Usage:
    qsv luau map [options] -n <main-script> [<input>]
    qsv luau map [options] <new-columns> <main-script> [<input>]
    qsv luau filter [options] <main-script> [<input>]
    qsv luau map --help
    qsv luau filter --help
    qsv luau --help

Luau arguments:

    All <script> arguments/options can either be the Luau code, or if it starts with "file:",
    the filepath from which to load the script.

    Instead of using the --begin and --end options, you can also embed BEGIN and END scripts in the
    MAIN script by using the "BEGIN { ... }!" and "END { ... }!" syntax.

    The BEGIN script is embedded in the MAIN script by adding a BEGIN block at the top of the script.
    The BEGIN block can contain multiple statements.

    The END script is embedded in the MAIN script by adding an END block at the bottom of the script.
    The END block can contain multiple statements.

    <new-columns> is a comma-separated list of new computed columns to add to the CSV when using
    "luau map". Note that the new columns are added to the CSV after the existing columns.

Luau options:
    -x, --exec               exec[ute] Luau script, instead of the default eval[uate].
                             eval (default) expects just a single Luau expression,
                             while exec expects one or more statements, allowing
                             full-fledged Luau programs. This only applies to the main-script
                             argument, not the BEGIN & END scripts.
    -g, --no-globals         Don't create Luau global variables for each column, only col.
                             Useful when some column names mask standard Luau globals.
                             Note: access to Luau globals thru _G remains even without -g.
    -B, --begin <script>     Luau script/file to execute in the BEGINning, before processing the CSV
                             with the main-script.
                             The variables _IDX and _ROWCOUNT are set to zero before invoking
                             the BEGIN script.
                             Typically used to initialize global variables.
                             Takes precedence over an embedded BEGIN script.
    -E, --end <script>       Luau script/file to execute at the END, after processing the CSV with
                             the main-script.
                             Both _IDX and _ROWCOUNT variables are set to the rowcount before invoking
                             the END script. Typically used for aggregations.
                             The output of the END script is sent to stderr.
                             Takes precedence over an embedded END script.
    --luau-path <pattern>    The LUAU_PATH pattern to use from which the scripts 
                             can "require" lua/luau library files from.
                             See https://www.lua.org/pil/8.1.html
                             [default: ?;?.luau;?.lua]
    --max-errors <count>     The maximum number of errors to tolerate before aborting.
                             Set to zero to disable error limit.
                             [default: 100]

Common options:
    -h, --help             Display this message
    -o, --output <file>    Write output to <file> instead of stdout.
    -n, --no-headers       When set, the first row will not be interpreted
                           as headers.
    -d, --delimiter <arg>  The field delimiter for reading CSV data.
                           Must be a single character. (default: ,)
    -p, --progressbar      Show progress bars. Not valid for stdin.
                           Also not valid when "_INDEX" var is used
                           for random access.
"#;

use std::{env, fs};

#[cfg(any(feature = "full", feature = "lite"))]
use indicatif::{ProgressBar, ProgressDrawTarget};
use log::{debug, info, log_enabled};
use mlua::{Lua, LuaSerdeExt, Value};
use serde::Deserialize;
use tempfile;

use crate::{
    config::{Config, Delimiter},
    util, CliError, CliResult,
};

#[derive(Deserialize)]
struct Args {
    cmd_map:          bool,
    cmd_filter:       bool,
    arg_new_columns:  Option<String>,
    arg_main_script:  String,
    arg_input:        Option<String>,
    flag_exec:        bool,
    flag_no_globals:  bool,
    flag_begin:       Option<String>,
    flag_end:         Option<String>,
    flag_luau_path:   String,
    flag_output:      Option<String>,
    flag_no_headers:  bool,
    flag_delimiter:   Option<Delimiter>,
    flag_progressbar: bool,
    flag_max_errors:  usize,
}

impl From<mlua::Error> for CliError {
    fn from(err: mlua::Error) -> CliError {
        CliError::Other(err.to_string())
    }
}

pub fn run(argv: &[&str]) -> CliResult<()> {
    let args: Args = util::get_args(USAGE, argv)?;
    let rconfig = Config::new(&args.arg_input)
        .delimiter(args.flag_delimiter)
        .no_headers(args.flag_no_headers);

    let mut luau_script = if let Some(script_filepath) = args.arg_main_script.strip_prefix("file:")
    {
        match fs::read_to_string(script_filepath) {
            Ok(file_contents) => file_contents,
            Err(e) => return fail_clierror!("Cannot load Luau file: {e}"),
        }
    } else {
        args.arg_main_script.clone()
    };

    let mut index_file_used = luau_script.contains("_INDEX");

    // check if the main script has BEGIN and END blocks
    // and if so, extract them and remove them from the main script
    let begin_re = regex::Regex::new(r"(?ms)BEGIN \{(?P<begin_block>.*?)\}!").unwrap();
    let end_re = regex::Regex::new(r"(?ms)END \{(?P<end_block>.*?)\}!").unwrap();
    let mut embedded_begin_script = String::new();
    let mut embedded_end_script = String::new();
    let mut main_script = luau_script.clone();
    if let Some(caps) = begin_re.captures(&luau_script) {
        embedded_begin_script = caps["begin_block"].to_string();
        let begin_block_replace = format!("BEGIN {{{embedded_begin_script}}}!");
        debug!("begin_block_replace: {begin_block_replace:?}");
        main_script = main_script.replace(&begin_block_replace, "");
    }
    if let Some(caps) = end_re.captures(&main_script) {
        embedded_end_script = caps["end_block"].to_string();
        let end_block_replace = format!("END {{{embedded_end_script}}}!");
        main_script = main_script.replace(&end_block_replace, "");
    }
    luau_script = main_script;

    let mut main_script = if args.flag_exec {
        String::new()
    } else {
        String::from("return ")
    };

    main_script.push_str(luau_script.trim());
    debug!("MAIN script: {main_script:?}");

    // setup LUAU_PATH; create a temporary directory and add it to LUAU_PATH and copy date.lua
    // there so qsv luau users can just `local date = require "date"` in their scripts,
    // as well as load other lua/luau libraries as needed.
    let luadate_library = include_bytes!("../../resources/luau/vendor/luadate/date.lua");
    let Ok(temp_dir) = tempfile::tempdir() else {
        return fail!("Cannot create temporary directory to copy luadate library to.");
    };
    let luadate_path = temp_dir.path().join("date.lua");
    fs::write(luadate_path.clone(), luadate_library)?;
    let mut luau_path = args.flag_luau_path.clone();
    luau_path.push_str(&format!(";{}", luadate_path.as_os_str().to_string_lossy()));
    env::set_var("LUAU_PATH", luau_path.clone());
    info!(r#"set LUAU_PATH to "{luau_path}""#);

    // check if a BEGIN script was specified
    let begin_script = if let Some(ref begin) = args.flag_begin {
        if let Some(begin_filepath) = begin.strip_prefix("file:") {
            match fs::read_to_string(begin_filepath) {
                Ok(begin) => {
                    // check if the BEGIN script uses _INDEX
                    index_file_used = index_file_used || begin.contains("_INDEX");
                    begin
                }
                Err(e) => return fail_clierror!("Cannot load Luau BEGIN script file: {e}"),
            }
        } else {
            begin.to_string()
        }
    } else {
        embedded_begin_script.trim().to_string()
    };
    debug!("BEGIN script: {begin_script:?}");

    // check if an END script was specified
    let end_script = if let Some(ref end) = args.flag_end {
        if let Some(end_filepath) = end.strip_prefix("file:") {
            match fs::read_to_string(end_filepath) {
                Ok(end) => {
                    // check if the END script uses _INDEX
                    index_file_used = index_file_used || end.contains("_INDEX");
                    end
                }
                Err(e) => return fail_clierror!("Cannot load Luau END script file: {e}"),
            }
        } else {
            end.to_string()
        }
    } else {
        embedded_end_script.trim().to_string()
    };
    debug!("END script: {end_script:?}");

    // -------- setup Luau environment --------
    let luau = Lua::new();
    let luau_compiler = if log_enabled!(log::Level::Debug) || log_enabled!(log::Level::Trace) {
        // debugging is on, set more debugging friendly compiler settings
        // so we can see more error details in the logfile
        mlua::Compiler::new()
            .set_optimization_level(0)
            .set_debug_level(2)
            .set_coverage_level(2)
    } else {
        // use more performant compiler settings
        mlua::Compiler::new()
            .set_optimization_level(2)
            .set_debug_level(1)
            .set_coverage_level(0)
    };
    // set default Luau compiler
    luau.set_compiler(luau_compiler.clone());

    let globals = luau.globals();

    setup_helpers(&luau)?;

    if index_file_used {
        with_index(
            &rconfig,
            &args,
            &luau,
            &luau_compiler,
            &globals,
            &begin_script,
            &main_script,
            &end_script,
            args.flag_max_errors,
        )?;
    } else {
        no_index(
            &rconfig,
            &args,
            &luau,
            &luau_compiler,
            &globals,
            &begin_script,
            &main_script,
            &end_script,
            args.flag_max_errors,
        )?;
    }

    Ok(())
}

fn setup_helpers(luau: &Lua) -> Result<(), CliError> {
    // this is a helper function that can be called from Luau scripts
    // to send log messages to the logfile
    // the first parameter is the log level, and the following parameters are concatenated
    let qsv_log = luau.create_function(|luau, mut args: mlua::MultiValue| {
        let mut log_msg = String::with_capacity(20);
        let mut idx = 0_u8;
        let mut log_level = String::new();
        while let Some(val) = args.pop_front() {
            let val = luau.from_value::<serde_json::Value>(val)?;
            let val_str = &serde_json::to_string_pretty(&val).unwrap_or_default();
            if idx == 0 {
                log_level = val_str.trim_matches('"').to_lowercase();
            } else {
                log_msg.push_str(val_str.trim_matches('"'));
                if idx == u8::MAX {
                    break;
                }
            }
            idx += 1;
        }
        match log_level.as_str() {
            "info" => log::info!("{log_msg}"),
            "warn" => log::warn!("{log_msg}"),
            "error" => log::error!("{log_msg}"),
            "debug" => log::debug!("{log_msg}"),
            "trace" => log::trace!("{log_msg}"),
            _ => {
                log::info!("unknown log level: {log_level} msg: {log_msg}");
            }
        }
        Ok(())
    })?;
    luau.globals().set("qsv_log", qsv_log)?;

    // this is a helper function that can be called from Luau scripts
    // to coalesce - return the first non-null value in a list
    let qsv_coalesce = luau.create_function(|luau, mut args: mlua::MultiValue| {
        while let Some(val) = args.pop_front() {
            let val = luau.from_value::<serde_json::Value>(val)?;
            let val_str = &serde_json::to_string(&val).unwrap_or_default();
            if !val_str.is_empty() {
                return Ok(val_str.to_string());
            }
        }
        Ok(String::new())
    })?;
    luau.globals().set("qsv_coalesce", qsv_coalesce)?;
    Ok(())
}

fn no_index(
    rconfig: &Config,
    args: &Args,
    luau: &Lua,
    luau_compiler: &mlua::Compiler,
    globals: &mlua::Table,
    begin_script: &str,
    main_script: &str,
    end_script: &str,
    max_errors: usize,
) -> Result<(), CliError> {
    globals.set("cols", "{}")?;

    // we initialize the special vars _IDX and _ROWCOUNT
    globals.set("_IDX", 0)?;
    globals.set("_ROWCOUNT", 0)?;
    if !begin_script.is_empty() {
        info!("Compiling and executing BEGIN script.");
        let begin_bytecode = luau_compiler.compile(begin_script);
        if let Err(e) = luau
            .load(&begin_bytecode)
            .set_mode(mlua::ChunkMode::Binary)
            .exec()
        {
            return fail_clierror!("BEGIN error: Failed to execute \"{begin_script}\".\n{e}");
        }
        info!("BEGIN executed.");
    }
    let mut rdr = rconfig.reader()?;
    let mut wtr = Config::new(&args.flag_output).writer()?;
    let mut headers = rdr.headers()?.clone();
    let mut new_column_count = 0_u8;
    if !rconfig.no_headers {
        if !args.cmd_filter {
            let new_columns = args
                .arg_new_columns
                .as_ref()
                .ok_or("Specify new column names")?;

            let new_columns_vec: Vec<&str> = new_columns.split(',').collect();
            for new_column in new_columns_vec {
                new_column_count += 1;
                headers.push_field(new_column.trim());
            }
        }

        wtr.write_record(&headers)?;
    }
    #[cfg(any(feature = "full", feature = "lite"))]
    let show_progress =
        (args.flag_progressbar || std::env::var("QSV_PROGRESSBAR").is_ok()) && !rconfig.is_stdin();
    #[cfg(any(feature = "full", feature = "lite"))]
    let progress = ProgressBar::with_draw_target(None, ProgressDrawTarget::stderr_with_hz(5));
    #[cfg(any(feature = "full", feature = "lite"))]
    if show_progress {
        util::prep_progress(&progress, util::count_rows(rconfig)?);
    } else {
        progress.set_draw_target(ProgressDrawTarget::hidden());
    }

    let error_result: Value = luau.load("return \"<ERROR>\";").eval()?;
    let mut error_flag;
    let main_bytecode = luau_compiler.compile(main_script);
    let mut record = csv::StringRecord::new();
    let mut idx = 0_u64;
    let mut error_count = 0_usize;

    // main loop
    while rdr.read_record(&mut record)? {
        #[cfg(any(feature = "full", feature = "lite"))]
        if show_progress {
            progress.inc(1);
        }

        idx += 1;
        globals.set("_IDX", idx)?;

        // Updating col
        {
            let col =
                luau.create_table_with_capacity(record.len().try_into().unwrap_or_default(), 1)?;

            for (i, v) in record.iter().enumerate() {
                col.set(i + 1, v)?;
            }
            if !rconfig.no_headers {
                for (h, v) in headers.iter().zip(record.iter()) {
                    col.set(h, v)?;
                }
            }
            globals.set("col", col)?;
        }

        // Updating global
        if !args.flag_no_globals && !rconfig.no_headers {
            for (h, v) in headers.iter().zip(record.iter()) {
                globals.set(h, v)?;
            }
        }

        error_flag = false;
        let computed_value: Value = match luau
            .load(&main_bytecode)
            .set_mode(mlua::ChunkMode::Binary)
            .eval()
        {
            Ok(computed) => computed,
            Err(e) => {
                error_flag = true;
                error_count += 1;
                log::error!("_IDX: {idx} error({error_count}): {e:?}");
                error_result.clone()
            }
        };

        if max_errors > 0 && error_count > max_errors {
            info!("Maximum number of errors ({max_errors}) reached. Aborting MAIN script.");
            break;
        }

        if args.cmd_map {
            match computed_value {
                Value::String(string) => {
                    record.push_field(&string.to_string_lossy());
                }
                Value::Number(number) => {
                    let mut buffer = ryu::Buffer::new();
                    record.push_field(buffer.format(number));
                }
                Value::Integer(number) => {
                    let mut buffer = itoa::Buffer::new();
                    record.push_field(buffer.format(number));
                }
                Value::Boolean(boolean) => {
                    record.push_field(if boolean { "true" } else { "false" });
                }
                Value::Nil => {
                    record.push_field("");
                }
                Value::Table(table) => {
                    let mut columns_inserted = 0_u8;
                    for pair in table.pairs::<mlua::Value, mlua::Value>() {
                        // we don't care about the key, just the value
                        let (_k, v) = pair?;
                        match v {
                            Value::Integer(intval) => {
                                let mut buffer = itoa::Buffer::new();
                                record.push_field(buffer.format(intval));
                            }
                            Value::String(strval) => {
                                record.push_field(&strval.to_string_lossy());
                            }
                            Value::Number(number) => {
                                let mut buffer = ryu::Buffer::new();
                                record.push_field(buffer.format(number));
                            }
                            Value::Boolean(boolean) => {
                                record.push_field(if boolean { "true" } else { "false" });
                            }
                            Value::Nil => {
                                record.push_field("");
                            }
                            _ => {
                                return fail_clierror!(
                                    "Unexpected value type returned by provided Luau expression. \
                                     {v:?}"
                                );
                            }
                        }
                        columns_inserted += 1;
                        if new_column_count > 0 && columns_inserted >= new_column_count {
                            // we ignore table values more than the number of new columns defined
                            break;
                        }
                    }
                    // on the other hand, if there are less table values than expected
                    // we fill it up with empty fields
                    while new_column_count > 0 && columns_inserted < new_column_count {
                        record.push_field("");
                        columns_inserted += 1;
                    }
                }
                _ => {
                    return fail_clierror!(
                        "Unexpected value type returned by provided Luau expression. \
                         {computed_value:?}"
                    );
                }
            }

            wtr.write_record(&record)?;
        } else if args.cmd_filter {
            let must_keep_row = if error_flag {
                true
            } else {
                match computed_value {
                    Value::String(strval) => !strval.to_string_lossy().is_empty(),
                    Value::Boolean(boolean) => boolean,
                    Value::Nil => false,
                    Value::Integer(intval) => intval != 0,
                    // we compare to f64::EPSILON as float comparison to zero
                    // unlike int, where we can say intval != 0, we cannot do fltval !=0
                    // https://doc.rust-lang.org/std/primitive.f64.html#associatedconstant.EPSILON
                    Value::Number(fltval) => (fltval).abs() > f64::EPSILON,
                    _ => true,
                }
            };

            if must_keep_row {
                wtr.write_record(&record)?;
            }
        }
    }

    if !end_script.is_empty() {
        // at the END, set a convenience variable named _ROWCOUNT;
        // true, _ROWCOUNT is equal to _IDX at this point, but this
        // should make for more readable END scripts.
        // Also, _ROWCOUNT is zero during the main script, and only set
        // to _IDX during the END script.
        globals.set("_ROWCOUNT", idx)?;

        info!("Compiling and executing END script. _ROWCOUNT: {idx}");
        let end_bytecode = luau_compiler.compile(end_script);
        let end_value: Value = match luau
            .load(&end_bytecode)
            .set_mode(mlua::ChunkMode::Binary)
            .eval()
        {
            Ok(computed) => computed,
            Err(e) => {
                log::error!("END error: Cannot evaluate \"{end_script}\".\n{e}");
                log::error!("END globals: {globals:?}");
                error_result.clone()
            }
        };
        let end_string = match end_value {
            Value::String(string) => string.to_string_lossy().to_string(),
            Value::Number(number) => number.to_string(),
            Value::Integer(number) => number.to_string(),
            Value::Boolean(boolean) => (if boolean { "true" } else { "false" }).to_string(),
            Value::Nil => String::new(),
            _ => {
                return fail_clierror!(
                    "Unexpected END value type returned by provided Luau expression. {end_value:?}"
                );
            }
        };
        winfo!("{end_string}");
    }
    wtr.flush()?;
    #[cfg(any(feature = "full", feature = "lite"))]
    if show_progress {
        util::finish_progress(&progress);
    }
    if error_count > 0 {
        return fail_clierror!("Luau errors encountered: {error_count}");
    };
    Ok(())
}

// this function is largely similar to no_index
// the primary difference being that we use an Indexed File rdr in the main loop
// differences pointed out in comments below
fn with_index(
    rconfig: &Config,
    args: &Args,
    luau: &Lua,
    luau_compiler: &mlua::Compiler,
    globals: &mlua::Table,
    begin_script: &str,
    main_script: &str,
    end_script: &str,
    max_errors: usize,
) -> Result<(), CliError> {
    let Some(mut idx_file) = rconfig.indexed()? else { return fail!("Index required but no index file found") };

    globals.set("cols", "{}")?;

    let row_count = util::count_rows(rconfig).unwrap_or_default();

    let mut wtr = Config::new(&args.flag_output).writer()?;
    let mut headers = idx_file.headers()?.clone();
    let mut new_column_count = 0_u8;
    if !rconfig.no_headers {
        if !args.cmd_filter {
            let new_columns = args
                .arg_new_columns
                .as_ref()
                .ok_or("Specify new column names")?;

            let new_columns_vec: Vec<&str> = new_columns.split(',').collect();
            for new_column in new_columns_vec {
                new_column_count += 1;
                headers.push_field(new_column.trim());
            }
        }

        wtr.write_record(&headers)?;
    }

    // unlike no_index, we actually know the row_count at the BEGINning
    globals.set("_IDX", 0)?;
    globals.set("_INDEX", 0)?;
    globals.set("_ROWCOUNT", row_count)?;
    globals.set("_LASTROW", row_count - 1)?;
    if !begin_script.is_empty() {
        info!("Compiling and executing BEGIN script.");
        let begin_bytecode = luau_compiler.compile(begin_script);
        if let Err(e) = luau
            .load(&begin_bytecode)
            .set_mode(mlua::ChunkMode::Binary)
            .exec()
        {
            return fail_clierror!("BEGIN error: Failed to execute \"{begin_script}\".\n{e}");
        }
        info!("BEGIN executed.");
    }

    // unlike no_index, setting "_INDEX" allows us to change the current record
    // for the NEXT read
    let mut pos = globals.get::<_, isize>("_INDEX").unwrap_or_default();
    let mut curr_record = if pos > 0 && pos <= row_count as isize {
        pos as u64
    } else {
        0_u64
    };
    debug!("BEGIN current record: {curr_record}");
    idx_file.seek(curr_record)?;

    let error_result: Value = luau.load("return \"<ERROR>\";").eval()?;
    let mut error_flag;

    let main_bytecode = luau_compiler.compile(main_script);
    let mut record = csv::StringRecord::new();
    let mut error_count = 0_usize;

    // main loop - here we an indexed file reader, seeking to the next
    // record to read by looking at _INDEX special var
    while idx_file.read_record(&mut record)? {
        globals.set("_IDX", curr_record)?;

        {
            let col =
                luau.create_table_with_capacity(record.len().try_into().unwrap_or_default(), 1)?;

            for (i, v) in record.iter().enumerate() {
                col.set(i + 1, v)?;
            }
            if !rconfig.no_headers {
                for (h, v) in headers.iter().zip(record.iter()) {
                    col.set(h, v)?;
                }
            }
            globals.set("col", col)?;
        }

        if !args.flag_no_globals && !rconfig.no_headers {
            for (h, v) in headers.iter().zip(record.iter()) {
                globals.set(h, v)?;
            }
        }

        error_flag = false;
        let computed_value: Value = match luau
            .load(&main_bytecode)
            .set_mode(mlua::ChunkMode::Binary)
            .eval()
        {
            Ok(computed) => computed,
            Err(e) => {
                error_flag = true;
                error_count += 1;
                log::error!("_IDX: {curr_record} error({error_count}): {e:?}");
                error_result.clone()
            }
        };

        if max_errors > 0 && error_count > max_errors {
            info!("Maximum number of errors ({max_errors}) reached. Aborting MAIN script.");
            break;
        }

        if args.cmd_map {
            match computed_value {
                Value::String(string) => {
                    record.push_field(&string.to_string_lossy());
                }
                Value::Number(number) => {
                    let mut buffer = ryu::Buffer::new();
                    record.push_field(buffer.format(number));
                }
                Value::Integer(number) => {
                    let mut buffer = itoa::Buffer::new();
                    record.push_field(buffer.format(number));
                }
                Value::Boolean(boolean) => {
                    record.push_field(if boolean { "true" } else { "false" });
                }
                Value::Nil => {
                    record.push_field("");
                }
                Value::Table(table) => {
                    let mut columns_inserted = 0_u8;
                    for pair in table.pairs::<mlua::Value, mlua::Value>() {
                        // we don't care about the key, just the value
                        let (_k, v) = pair?;
                        match v {
                            Value::Integer(intval) => {
                                let mut buffer = itoa::Buffer::new();
                                record.push_field(buffer.format(intval));
                            }
                            Value::String(strval) => {
                                record.push_field(&strval.to_string_lossy());
                            }
                            Value::Number(number) => {
                                let mut buffer = ryu::Buffer::new();
                                record.push_field(buffer.format(number));
                            }
                            Value::Boolean(boolean) => {
                                record.push_field(if boolean { "true" } else { "false" });
                            }
                            Value::Nil => {
                                record.push_field("");
                            }
                            _ => {
                                return fail_clierror!(
                                    "Unexpected value type returned by provided Luau expression. \
                                     {v:?}"
                                );
                            }
                        }
                        columns_inserted += 1;
                        if new_column_count > 0 && columns_inserted >= new_column_count {
                            // we ignore table values more than the number of new columns defined
                            break;
                        }
                    }
                    // on the other hand, if there are less table values than expected
                    // we fill it up with empty fields
                    while new_column_count > 0 && columns_inserted < new_column_count {
                        record.push_field("");
                        columns_inserted += 1;
                    }
                }
                _ => {
                    return fail_clierror!(
                        "Unexpected value type returned by provided Luau expression. \
                         {computed_value:?}"
                    );
                }
            }

            wtr.write_record(&record)?;
        } else if args.cmd_filter {
            let must_keep_row = if error_flag {
                true
            } else {
                match computed_value {
                    Value::String(strval) => !strval.to_string_lossy().is_empty(),
                    Value::Boolean(boolean) => boolean,
                    Value::Nil => false,
                    Value::Integer(intval) => intval != 0,
                    Value::Number(fltval) => (fltval).abs() > f64::EPSILON,
                    _ => true,
                }
            };

            if must_keep_row {
                wtr.write_record(&record)?;
            }
        }

        pos = globals.get::<_, isize>("_INDEX").unwrap_or_default();
        if pos < 0 || pos as u64 > row_count {
            break;
        }
        let next_record = if pos > 0 && pos <= row_count as isize {
            pos as u64
        } else if pos < 0 && pos.unsigned_abs() as u64 <= row_count {
            row_count + 1 - pos.unsigned_abs() as u64
        } else {
            0_u64
        };
        if idx_file.seek(next_record).is_err() {
            break;
        }
        curr_record = next_record;
    } // main loop

    if !end_script.is_empty() {
        info!("Compiling and executing END script. _ROWCOUNT: {row_count}");
        let end_bytecode = luau_compiler.compile(end_script);
        let end_value: Value = match luau
            .load(&end_bytecode)
            .set_mode(mlua::ChunkMode::Binary)
            .eval()
        {
            Ok(computed) => computed,
            Err(e) => {
                log::error!("END error: Cannot evaluate \"{end_script}\".\n{e}");
                log::error!("END globals: {globals:?}");
                error_result.clone()
            }
        };
        let end_string = match end_value {
            Value::String(string) => string.to_string_lossy().to_string(),
            Value::Number(number) => number.to_string(),
            Value::Integer(number) => number.to_string(),
            Value::Boolean(boolean) => (if boolean { "true" } else { "false" }).to_string(),
            Value::Nil => String::new(),
            _ => {
                return fail_clierror!(
                    "Unexpected END value type returned by provided Luau expression. {end_value:?}"
                );
            }
        };
        winfo!("{end_string}");
    }
    wtr.flush()?;
    if error_count > 0 {
        return fail_clierror!("Luau errors encountered: {error_count}");
    };
    Ok(())
}
