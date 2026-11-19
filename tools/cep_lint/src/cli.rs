// CEP:FILE: tools/cep_lint/src/cli.rs
// CEP:WHAT: Command line interface: option parsing, usage text, and the version banner.
// CEP:WHY: Section 50.1 defines the invocation contract and exit code three for usage errors.
// CEP:CLASS: CEP-2
// CEP:STATUS: complete
// CEP:FAILURE: Returns usage errors for unknown options; never panics on argument input.
// CEP:ASSUMES: Options are the documented flags with short forms.
// CEP:COST: cold; linear in arguments.
// CEP:EVIDENCE: CI invocation checks.

/// Parsed command line options.
#[derive(Debug, Clone)]
pub struct Options {
    /// Configuration path from --config, -c, or CEP_LINT_CONFIG.
    pub config: String,
    /// True when --self-test or -t was given.
    pub self_test: bool,
    /// True when --help or -h was given.
    pub help: bool,
    /// True when --version or -v was given.
    pub version: bool,
    /// True when the Bloodhound receipt rule is activated.
    pub verify_hunt: bool,
    /// True when the HUNTER receipt rule is activated.
    pub verify_hunter: bool,
    /// Input files and directories to scan.
    pub paths: Vec<String>,
}

/// Default configuration path per section 50.1.
pub const DEFAULT_CONFIG: &str = ".cep/cep_lint.json";

/// Exit code for command line usage errors per section 50.1.
pub const USAGE_EXIT: i32 = 3;

/// CEP:WHAT: Parses command line arguments into options or a usage error message.
/// CEP:WHY: Exit code three is reserved for command line usage errors.
/// CEP:STATUS: complete
/// CEP:FAILURE: Returns Err with a detail string for unknown options and missing values.
/// CEP:ASSUMES: The environment override CEP_LINT_CONFIG supplies the default configuration path.
/// CEP:COST: cold; linear in arguments.
/// CEP:EVIDENCE: CI invocation checks.
pub fn parse(args: &[String]) -> Result<Options, String> {
    let mut opts = Options {
        config: std::env::var("CEP_LINT_CONFIG").unwrap_or_else(|_| DEFAULT_CONFIG.to_string()),
        self_test: false,
        help: false,
        version: false,
        verify_hunt: false,
        verify_hunter: false,
        paths: Vec::new(),
    };
    assign_flags(args, &mut opts)?;
    Ok(opts)
}

/// CEP:WHAT: Assigns parsed flags into the options accumulator.
/// CEP:WHY: Splitting the option loop keeps the parser within statement limits.
/// CEP:STATUS: complete
/// CEP:FAILURE: Returns Err for unknown options and missing values.
/// CEP:ASSUMES: none
/// CEP:COST: cold; linear in arguments.
/// CEP:EVIDENCE: CI invocation checks.
fn assign_flags(args: &[String], opts: &mut Options) -> Result<(), String> {
    let mut i = 0;
    while i < args.len() {
        let a = args[i].as_str();
        match a {
            "--config" | "-c" => {
                i += 1;
                let Some(v) = args.get(i) else { return Err("--config requires a value".to_string()) };
                opts.config = v.clone();
            }
            "--self-test" | "-t" => opts.self_test = true,
            "--help" | "-h" => opts.help = true,
            "--version" | "-v" => opts.version = true,
            "--verify-hunt" => opts.verify_hunt = true,
            "--verify-hunter" => opts.verify_hunter = true,
            other if other.starts_with('-') => return Err(format!("unknown option {}", other)),
            other => opts.paths.push(other.to_string()),
        }
        i += 1;
    }
    Ok(())
}

/// CEP:WHAT: Returns the usage text for the help option.
/// CEP:WHY: Section 50.1 mandates a help output listing the documented options.
/// CEP:STATUS: complete
/// CEP:FAILURE: none
/// CEP:ASSUMES: none
/// CEP:COST: cold; builds one string.
/// CEP:EVIDENCE: CI invocation checks.
pub fn usage() -> String {
    let mut s = String::new();
    s.push_str("usage: cep_lint [options] <files or directories>\n");
    s.push_str("  --config, -c <path>   lint configuration JSON (default .cep/cep_lint.json, env CEP_LINT_CONFIG)\n");
    s.push_str("  --self-test, -t       run the self-test manifest and exit\n");
    s.push_str("  --help, -h            show this usage\n");
    s.push_str("  --version, -v         print tool and standard versions\n");
    s.push_str("  --verify-hunt         enforce the Bloodhound receipt rule on cycle-exact classes\n");
    s.push_str("  --verify-hunter       enforce the HUNTER receipt rule on cycle-exact classes\n");
    s
}
