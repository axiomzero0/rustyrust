// CEP:FILE: tools/cep_lint/src/main.rs
// CEP:WHAT: Binary entry point with the exit code mapping of section 50.1.
// CEP:WHY: The tool is the repository's mechanical enforcement boundary.
// CEP:CLASS: CEP-2
// CEP:STATUS: complete
// CEP:FAILURE: Exits with code two on bootstrap failures and code three on usage errors.
// CEP:ASSUMES: The process working directory is the repository root for live runs.
// CEP:COST: cold; one engine run per invocation.
// CEP:EVIDENCE: CI self-lint, self-test, and doc-sync jobs.

use cep_lint::cli;
use cep_lint::config;
use cep_lint::engine::{self, Flags};
use cep_lint::manifest;
use std::path::Path;
use std::process::ExitCode;

/// Exit code for bootstrap failures per section 50.1.
const BOOTSTRAP_EXIT: i32 = 2;

/// Exit code for the clean report case.
const CLEAN_EXIT: i32 = 0;

/// CEP:WHAT: Runs the lint engine or the self-test and maps the result to an exit code.
/// CEP:WHY: CI gates depend on the exact exit code contract of section 50.1.
/// CEP:ID: CS-a06
/// CEP:STATUS: complete
/// CEP:FAILURE: Returns usage and bootstrap exit codes; findings map through configuration data.
/// CEP:ASSUMES: none
/// CEP:COST: cold; one pass over the requested paths.
/// CEP:EVIDENCE: doc file docs/functions/CS/CS-a06.md; CI jobs.
fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let opts = match cli::parse(&args) {
        Ok(o) => o,
        Err(detail) => {
            eprintln!("CEP-LINT-USAGE: {}", detail);
            eprint!("{}", cli::usage());
            return code(cli::USAGE_EXIT);
        }
    };
    if opts.help {
        print!("{}", cli::usage());
        return code(CLEAN_EXIT);
    }
    let cfg_path = Path::new(&opts.config).to_path_buf();
    if opts.version {
        return version_banner(&cfg_path);
    }
    dispatch(&opts, &cfg_path)
}

/// CEP:WHAT: Routes non-help options to their invocation modes.
/// CEP:WHY: Splitting the router keeps main and each mode within statement limits.
/// CEP:STATUS: complete
/// CEP:FAILURE: none
/// CEP:ASSUMES: none
/// CEP:COST: cold; constant time.
/// CEP:EVIDENCE: CI invocation checks.
fn dispatch(opts: &cli::Options, cfg_path: &Path) -> ExitCode {
    if opts.self_test {
        return self_test(cfg_path);
    }
    if opts.paths.is_empty() {
        eprintln!("CEP-LINT-USAGE: no input paths");
        eprint!("{}", cli::usage());
        return code(cli::USAGE_EXIT);
    }
    lint(cfg_path, opts)
}

/// CEP:WHAT: Runs the self-test path and maps the outcome to an exit code.
/// CEP:WHY: The self-verification gate is a first-class invocation mode.
/// CEP:STATUS: complete
/// CEP:FAILURE: Bootstrap failures exit two; scenario failures exit one.
/// CEP:ASSUMES: The summary marks total failure counts in its first line.
/// CEP:COST: cold; one manifest run.
/// CEP:EVIDENCE: CI self-test job.
fn self_test(cfg_path: &Path) -> ExitCode {
    match manifest::run(cfg_path) {
        Ok(text) => {
            print!("{}", text);
            code(if text.contains("failed") { 1 } else { CLEAN_EXIT })
        }
        Err(e) => {
            eprintln!("{}", e.format());
            code(BOOTSTRAP_EXIT)
        }
    }
}

/// CEP:WHAT: Runs the lint path over the requested paths and prints the report.
/// CEP:WHY: This is the default invocation of the tool.
/// CEP:STATUS: complete
/// CEP:FAILURE: Bootstrap failures exit two; findings map through the fail-on list.
/// CEP:ASSUMES: Input paths are present; the usage path rejects empty input earlier.
/// CEP:COST: cold; one engine pass.
/// CEP:EVIDENCE: CI self-lint job.
fn lint(cfg_path: &Path, opts: &cli::Options) -> ExitCode {
    let cfg = match config::load(cfg_path) {
        Ok(c) => c,
        Err(e) => {
            eprintln!("{}", e.format());
            return code(BOOTSTRAP_EXIT);
        }
    };
    let cwd = std::env::current_dir().unwrap_or_else(|_| Path::new(".").to_path_buf());
    let flags = Flags { verify_hunt: opts.verify_hunt, verify_hunter: opts.verify_hunter };
    let findings = match engine::run(&cfg, &opts.paths, &cwd, None, false, flags) {
        Ok(f) => f,
        Err(e) => {
            eprintln!("{}", e.format());
            return code(BOOTSTRAP_EXIT);
        }
    };
    print!("{}", engine::report(&findings));
    code(engine::exit_code(&cfg, &findings))
}

/// CEP:WHAT: Prints the tool and standard versions per section 50.1.
/// CEP:WHY: The banner binds the tool binary to the standard revision it enforces.
/// CEP:STATUS: complete
/// CEP:FAILURE: A configuration failure yields exit code two with the bootstrap message.
/// CEP:ASSUMES: The standard version is configuration data, not compiled in.
/// CEP:COST: cold; one configuration load.
/// CEP:EVIDENCE: CI version banner check.
fn version_banner(cfg_path: &Path) -> ExitCode {
    let tool = env!("CARGO_PKG_VERSION");
    match config::load(cfg_path) {
        Ok(cfg) => {
            println!("cep_lint {}", tool);
            println!("standard: {} {}", cfg.standard_name, cfg.standard_version);
            code(CLEAN_EXIT)
        }
        Err(e) => {
            println!("cep_lint {}", tool);
            eprintln!("{}", e.format());
            code(BOOTSTRAP_EXIT)
        }
    }
}

/// CEP:WHAT: Converts an integer status into an ExitCode.
/// CEP:WHY: The engine works with configured integer codes.
/// CEP:STATUS: complete
/// CEP:FAILURE: none
/// CEP:ASSUMES: Exit codes are within the u8 range of process semantics.
/// CEP:COST: cold; constant time.
/// CEP:EVIDENCE: cep_lint self-lint.
fn code(value: i32) -> ExitCode {
    ExitCode::from(value as u8)
}
