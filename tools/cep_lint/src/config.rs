// CEP:FILE: tools/cep_lint/src/config.rs
// CEP:WHAT: Configuration loading, include merging, and schema validation with bootstrap error codes.
// CEP:WHY: Section 50.4 defines eight bootstrap failures; all of them originate here.
// CEP:CLASS: CEP-2
// CEP:STATUS: complete
// CEP:FAILURE: Returns BootstrapError with codes one through eight per section 50.4.
// CEP:ASSUMES: Configuration JSON fits the documented schema; unknown keys are ignored.
// CEP:COST: cold; linear in configuration size.
// CEP:EVIDENCE: unit test cep_lint::config::tests::bootstrap_codes; engine self-lint.

use crate::json::{self, Value};
use crate::model::{BootstrapError, Config, Rule};
use std::fs;
use std::path::{Path, PathBuf};

/// Maximum include depth before a cycle or excessive depth is reported.
const MAX_INCLUDE_DEPTH: usize = 8;

/// Maximum valid severity value.
const MAX_SEVERITY: u8 = 3;

/// Default exit code for the clean case.
const DEFAULT_EXIT_CLEAN: i64 = 0;

/// Default exit code for the findings case.
const DEFAULT_EXIT_FINDINGS: i64 = 1;

/// Default exit code for the bootstrap case.
const DEFAULT_EXIT_BOOTSTRAP: i64 = 2;

/// Default exit code for the usage case.
const DEFAULT_EXIT_USAGE: i64 = 3;

/// Bootstrap code for file input failures.
const BOOT_IO: u8 = 1;

/// Bootstrap code for JSON parse failures.
const BOOT_JSON: u8 = 2;

/// Bootstrap code for include cycles and excessive depth.
const BOOT_INCLUDE: u8 = 3;

/// Bootstrap code for missing configuration keys.
const BOOT_MISSING_KEY: u8 = 4;

/// Bootstrap code for wrong parameter types.
const BOOT_WRONG_TYPE: u8 = 5;

/// Bootstrap code for unknown check primitives.
const BOOT_UNKNOWN_CHECK: u8 = 6;

/// Bootstrap code for severity values outside the zero to three range.
const BOOT_SEVERITY: u8 = 7;

/// Bootstrap code for invalid regular expressions.
const BOOT_REGEX: u8 = 8;

/// Parameter keys whose values are regular expressions and must compile.
const REGEX_PARAM_KEYS: [&str; 10] = [
    "pattern", "suppression_pattern", "name_pattern", "guard_pattern", "gap_pattern",
    "heading_pattern", "version_pattern", "id_pattern", "exempt_contexts", "exempt_paths",
];

/// Parameter keys whose values must be string lists.
const STRING_LIST_KEYS: [&str; 15] = [
    "required_fields", "allowed_classes", "allowed_values", "statuses", "suppress_statuses",
    "tokens", "applies_to_classes", "exempt_contexts", "exempt_paths", "patterns",
    "file_globs", "exclude_paths", "junk_names", "include", "line_limits_values",
];

/// Parameter keys whose values must be integers.
const INT_KEYS: [&str; 7] = [
    "max_start_line", "min_body_statements", "max_id_statements", "default_limit",
    "min_fuzz_executions", "min_diff_inputs", "name_group",
];

/// Parameter keys whose values must be strings.
const STRING_KEYS: [&str; 8] = [
    "class_field", "status_field", "suppress_status_field", "doc_path",
    "registry_path", "docs_prefix", "receipt_path", "public_key_path",
];

/// Check primitive names known to the engine.
const KNOWN_CHECKS: [&str; 15] = [
    "comment_block_schema", "function_policy", "comment_regex", "string_literal_regex",
    "flat_code_regex", "numeric_literal_policy", "macro_prefix_policy", "banned_token_policy",
    "doc_rule_coverage", "file_topology_policy", "rosetta_policy", "secret_probe_policy",
    "unsafe_doc_policy", "commit_hunt_manifest", "hunter_manifest",
];

/// CEP:WHAT: Loads the configuration from a path with includes and validates the schema.
/// CEP:WHY: The engine runs entirely from configuration data; loading is the trust boundary.
/// CEP:ID: CS-a03
/// CEP:STATUS: complete
/// CEP:FAILURE: Bootstrap errors one through eight per section 50.4.
/// CEP:ASSUMES: Includes are relative to the including file; include depth is bounded.
/// CEP:COST: cold; linear in configuration size.
/// CEP:EVIDENCE: doc file docs/functions/CS/CS-a03.md; unit test bootstrap_codes.
pub fn load(path: &Path) -> Result<Config, BootstrapError> {
    let mut stack: Vec<PathBuf> = Vec::new();
    let mut rules: Vec<Rule> = Vec::new();
    let root = load_inner(path, &mut stack, &mut rules, 0)?;
    for r in rules.iter() {
        validate_rule(r)?;
    }
    build_config(&root, rules)
}

/// CEP:WHAT: Builds the final configuration from the merged root value.
/// CEP:WHY: Splitting construction keeps the loader within statement limits.
/// CEP:STATUS: complete
/// CEP:FAILURE: Bootstrap error when the standard version key is missing.
/// CEP:ASSUMES: Exit code defaults follow section 50.1.
/// CEP:COST: cold; linear in configuration size.
/// CEP:EVIDENCE: unit test cep_lint::config::tests::bootstrap_codes.
fn build_config(root: &Value, rules: Vec<Rule>) -> Result<Config, BootstrapError> {
    let fail_on = root.get("fail_on")
        .and_then(|v| v.as_arr())
        .map(|items| items.iter().filter_map(|v| v.as_i64().map(|n| n as u8)).collect())
        .unwrap_or_else(|| vec![0, 1]);
    let exits = root.get("exit_codes");
    let cfg = Config {
        standard_name: root.get("standard").and_then(|s| s.get("name")).and_then(|v| v.as_str()).unwrap_or("CEP&CC").to_string(),
        standard_version: root.get("standard").and_then(|s| s.get("version")).and_then(|v| v.as_str()).unwrap_or("").to_string(),
        file_globs: str_list(root.get("file_globs")),
        exclude_paths: str_list(root.get("exclude_paths")),
        fail_on,
        exit_clean: exits.and_then(|e| e.get("clean")).and_then(|v| v.as_i64()).unwrap_or(DEFAULT_EXIT_CLEAN) as i32,
        exit_findings: exits.and_then(|e| e.get("findings")).and_then(|v| v.as_i64()).unwrap_or(DEFAULT_EXIT_FINDINGS) as i32,
        exit_bootstrap: exits.and_then(|e| e.get("bootstrap")).and_then(|v| v.as_i64()).unwrap_or(DEFAULT_EXIT_BOOTSTRAP) as i32,
        exit_usage: exits.and_then(|e| e.get("usage")).and_then(|v| v.as_i64()).unwrap_or(DEFAULT_EXIT_USAGE) as i32,
        rules,
    };
    if cfg.standard_version.is_empty() {
        return Err(BootstrapError::new(BOOT_MISSING_KEY, "standard.version is missing"));
    }
    Ok(cfg)
}

/// CEP:WHAT: Recursive include-aware loader that appends rules in include order.
/// CEP:WHY: Layered configurations let repositories share rule fragments without duplication.
/// CEP:STATUS: complete
/// CEP:FAILURE: Bootstrap errors one through three for unreadable files, bad JSON, and cycles.
/// CEP:ASSUMES: Includes are resolved relative to the including file.
/// CEP:COST: cold; linear in configuration size times include depth.
/// CEP:EVIDENCE: unit test cep_lint::config::tests::bootstrap_codes.
fn load_inner(path: &Path, stack: &mut Vec<PathBuf>, rules: &mut Vec<Rule>, depth: usize) -> Result<Value, BootstrapError> {
    if depth > MAX_INCLUDE_DEPTH {
        return Err(BootstrapError::new(BOOT_INCLUDE, format!("include depth beyond {} at {}", MAX_INCLUDE_DEPTH, path.display())));
    }
    let canon = path.to_path_buf();
    if stack.contains(&canon) {
        return Err(BootstrapError::new(BOOT_INCLUDE, format!("include cycle at {}", path.display())));
    }
    stack.push(canon);
    let text = fs::read_to_string(path)
        .map_err(|e| BootstrapError::new(BOOT_IO, format!("config {} unreadable: {}", path.display(), e)))?;
    let value = json::parse(&text)
        .map_err(|e| BootstrapError::new(BOOT_JSON, format!("config {}: {} at line {} column {}", path.display(), e.detail, e.line, e.col)))?;
    load_includes(path, &value, stack, rules, depth)?;
    if let Some(arr) = value.get("rules").and_then(|v| v.as_arr()).map(|a| a.to_vec()) {
        for item in arr {
            rules.push(parse_rule(&item)?);
        }
    }
    stack.pop();
    Ok(value)
}

/// CEP:WHAT: Resolves and merges include files declared by one configuration value.
/// CEP:WHY: Include handling is isolated from rule parsing.
/// CEP:STATUS: complete
/// CEP:FAILURE: Propagates bootstrap errors from nested loads.
/// CEP:ASSUMES: Include entries are strings.
/// CEP:COST: cold; linear in include count.
/// CEP:EVIDENCE: cep_lint self-lint.
fn load_includes(path: &Path, value: &Value, stack: &mut Vec<PathBuf>, rules: &mut Vec<Rule>, depth: usize) -> Result<(), BootstrapError> {
    if let Some(includes) = value.get("include").and_then(|v| v.as_arr()).map(|a| a.to_vec()) {
        for inc in includes {
            let inc_path = path.parent().unwrap_or(Path::new(".")).join(inc.as_str().unwrap_or(""));
            let sub = load_inner(&inc_path, stack, rules, depth + 1)?;
            merge_defaults(stack, &sub);
        }
    }
    Ok(())
}

/// CEP:WHAT: Placeholder merge step retained for include-key extension.
/// CEP:WHY: Key merging is currently unused because rules append rather than override.
/// CEP:STATUS: stub
/// CEP:OWNER: axiomzero0
/// CEP:TICKET: RR-101
/// CEP:FAILURE: none
/// CEP:ASSUMES: none
/// CEP:COST: cold; constant time.
/// CEP:EVIDENCE: cep_lint self-lint.
fn merge_defaults(_stack: &mut Vec<PathBuf>, _sub: &Value) {}

/// CEP:WHAT: Parses one rule object into the Rule model.
/// CEP:WHY: Missing required keys are bootstrap code four failures.
/// CEP:STATUS: complete
/// CEP:FAILURE: Bootstrap error for missing id, check, severity, or message keys.
/// CEP:ASSUMES: none
/// CEP:COST: cold; linear in rule fields.
/// CEP:EVIDENCE: cep_lint self-lint.
fn parse_rule(item: &Value) -> Result<Rule, BootstrapError> {
    let id = item.get("id").and_then(|v| v.as_str())
        .ok_or_else(|| BootstrapError::new(BOOT_MISSING_KEY, "rule.id is missing"))?.to_string();
    let check = item.get("check").and_then(|v| v.as_str())
        .ok_or_else(|| BootstrapError::new(BOOT_MISSING_KEY, format!("rule {}.check is missing", id)))?.to_string();
    let severity = item.get("severity").and_then(|v| v.as_i64())
        .ok_or_else(|| BootstrapError::new(BOOT_MISSING_KEY, format!("rule {}.severity is missing", id)))? as u8;
    let message = item.get("message").and_then(|v| v.as_str())
        .ok_or_else(|| BootstrapError::new(BOOT_MISSING_KEY, format!("rule {}.message is missing", id)))?.to_string();
    let mode = item.get("mode").and_then(|v| v.as_str()).map(|s| s.to_string());
    let params = item.get("params").cloned().unwrap_or(Value::Obj(Vec::new()));
    Ok(Rule { id, check, mode, severity, message, params })
}

/// CEP:WHAT: Validates severity range, check primitive name, and regex compilability for one rule.
/// CEP:WHY: Eager validation keeps check execution free of pattern failures.
/// CEP:STATUS: complete
/// CEP:FAILURE: Bootstrap errors six, seven, and eight.
/// CEP:ASSUMES: none
/// CEP:COST: cold; linear in regex parameters.
/// CEP:EVIDENCE: cep_lint self-lint.
fn validate_rule(rule: &Rule) -> Result<(), BootstrapError> {
    if rule.severity > MAX_SEVERITY {
        return Err(BootstrapError::new(BOOT_SEVERITY, format!("rule {} severity {} is out of range", rule.id, rule.severity)));
    }
    if !KNOWN_CHECKS.contains(&rule.check.as_str()) {
        return Err(BootstrapError::new(BOOT_UNKNOWN_CHECK, format!("rule {} uses unknown check {}", rule.id, rule.check)));
    }
    validate_params(rule)
}

/// CEP:WHAT: Compiles every regex-shaped parameter of one rule.
/// CEP:WHY: Regex parameters are the only failure-capable configuration inputs at check time.
/// CEP:STATUS: complete
/// CEP:FAILURE: Bootstrap error eight with the parameter key named.
/// CEP:ASSUMES: Regex parameter keys are listed in REGEX_PARAM_KEYS.
/// CEP:COST: cold; linear in parameters.
/// CEP:EVIDENCE: cep_lint self-lint.
fn validate_params(rule: &Rule) -> Result<(), BootstrapError> {
    if let Value::Obj(pairs) = &rule.params {
        for (k, v) in pairs.iter() {
            validate_param(rule.id.as_str(), k.as_str(), v)?;
            if !REGEX_PARAM_KEYS.contains(&k.as_str()) {
                continue;
            }
            compile_all(v).map_err(|e| BootstrapError::new(BOOT_REGEX, format!("rule {} param {}: {}", rule.id, k, e)))?;
        }
    }
    Ok(())
}

/// CEP:WHAT: Validates the declared type of one parameter against its key family.
/// CEP:WHY: Wrong parameter types are bootstrap code five failures.
/// CEP:STATUS: complete
/// CEP:FAILURE: Bootstrap error five naming the key and the expected type.
/// CEP:ASSUMES: Key families are enumerated in the module constants.
/// CEP:COST: cold; constant per parameter.
/// CEP:EVIDENCE: cep_lint self-lint.
fn validate_param(rule_id: &str, key: &str, v: &Value) -> Result<(), BootstrapError> {
    let expected = if STRING_LIST_KEYS.contains(&key) {
        "string list"
    } else if INT_KEYS.contains(&key) {
        "integer"
    } else if STRING_KEYS.contains(&key) {
        "string"
    } else {
        return Ok(());
    };
    let ok = match expected {
        "string list" => v.as_arr().map(|a| a.iter().all(|i| matches!(i, Value::Str(_) | Value::Num(_)))).unwrap_or(false),
        "integer" => v.as_i64().is_some(),
        _ => v.as_str().is_some(),
    };
    if !ok {
        return Err(BootstrapError::new(BOOT_WRONG_TYPE, format!("rule {} param {} must be a {}", rule_id, key, expected)));
    }
    Ok(())
}

/// CEP:WHAT: Compiles one or many regex parameter values.
/// CEP:WHY: List and scalar regex parameters share one validation path.
/// CEP:STATUS: complete
/// CEP:FAILURE: Returns the regex error text for the first invalid pattern.
/// CEP:ASSUMES: none
/// CEP:COST: cold; linear in pattern count.
/// CEP:EVIDENCE: cep_lint self-lint.
fn compile_all(v: &Value) -> Result<(), String> {
    match v {
        Value::Str(s) => regex::Regex::new(s).map(|_| ()).map_err(|e| e.to_string()),
        Value::Arr(items) => {
            for it in items {
                compile_all(it)?;
            }
            Ok(())
        }
        _ => Ok(()),
    }
}

/// CEP:WHAT: Reads a string list value.
/// CEP:WHY: Glob and exclusion lists are string arrays.
/// CEP:STATUS: complete
/// CEP:FAILURE: none
/// CEP:ASSUMES: Non-string items are ignored deterministically.
/// CEP:COST: cold; linear in list length.
/// CEP:EVIDENCE: cep_lint self-lint.
fn str_list(v: Option<&Value>) -> Vec<String> {
    v.and_then(|x| x.as_arr())
        .map(|items| items.iter().filter_map(|i| i.as_str().map(|s| s.to_string())).collect())
        .unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use super::*;

/// CEP:WHAT: Verifies bootstrap error codes for unreadable files and malformed JSON.
/// CEP:WHY: Section 50.4 codes one and two are contract behavior.
/// CEP:STATUS: complete
/// CEP:FAILURE: Fails the test binary on regression.
/// CEP:ASSUMES: none
/// CEP:COST: cold; unit test only.
/// CEP:EVIDENCE: cargo test -p cep_lint.
    #[test]
    fn bootstrap_codes() {
        let e1 = load(Path::new("/nonexistent/cep_lint.json")).unwrap_err();
        assert_eq!(e1.code, BOOT_IO);
        let dir = std::env::temp_dir().join("cep_lint_cfg_test");
        fs::create_dir_all(&dir).unwrap();
        let bad = dir.join("bad.json");
        fs::write(&bad, "{\"a\": }").unwrap();
        let e2 = load(&bad).unwrap_err();
        assert_eq!(e2.code, BOOT_JSON);
        assert!(e2.detail.contains("line"));
    }
}
