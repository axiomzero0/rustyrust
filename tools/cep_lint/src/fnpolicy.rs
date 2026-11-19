// CEP:FILE: tools/cep_lint/src/fnpolicy.rs
// CEP:WHAT: Function policy checks: comment blocks, naming, stub bodies, constant returns, and documentation identifiers.
// CEP:WHY: Rules CEP-LINT-FUNC-BLOCK, CEP-LINT-STUB-BODY, CEP-LINT-CONSTANT-RETURN, CEP-LINT-NAMING-FUNC, and CEP-LINT-DOC-ID share the same function extractor.
// CEP:CLASS: CEP-2
// CEP:STATUS: complete
// CEP:FAILURE: none; checks are total over scanned files and produce findings instead of panicking.
// CEP:ASSUMES: Function extraction follows the lexical model documented in scan.rs; extractor limits are documented in section 50.3.
// CEP:COST: cold; linear over functions per file.
// CEP:EVIDENCE: self-test scenario set for the five function_policy rules in tools/cep_lint/tests/manifest.json.


use crate::model::{substitute, Finding, Rule};
use crate::scan::{CommentBlock, Function, SourceFile};

/// Fallback minimum body statements for the block rule.
const DEFAULT_MIN_STATEMENTS: i64 = 2;

/// Fallback statement threshold for the documentation identifier rule.
const DEFAULT_MAX_ID_STATEMENTS: i64 = 10;

/// CEP:WHAT: Reads a string parameter from rule params with a fallback value.
/// CEP:WHY: Parameters are configuration data; fallbacks keep check code total.
/// CEP:STATUS: complete
/// CEP:FAILURE: none
/// CEP:ASSUMES: none
/// CEP:COST: cold; constant time.
/// CEP:EVIDENCE: cep_lint self-lint.
fn param_str(rule: &Rule, key: &str, fallback: &str) -> String {
    rule.params.get(key).and_then(|v| v.as_str()).unwrap_or(fallback).to_string()
}

/// CEP:WHAT: Reads an integer parameter from rule params with a fallback value.
/// CEP:WHY: Thresholds are configuration data with documented defaults.
/// CEP:STATUS: complete
/// CEP:FAILURE: none
/// CEP:ASSUMES: none
/// CEP:COST: cold; constant time.
/// CEP:EVIDENCE: cep_lint self-lint.
fn param_int(rule: &Rule, key: &str, fallback: i64) -> i64 {
    rule.params.get(key).and_then(|v| v.as_i64()).unwrap_or(fallback)
}

/// CEP:WHAT: Reads a string list parameter from rule params.
/// CEP:WHY: Field and status lists are configuration data.
/// CEP:STATUS: complete
/// CEP:FAILURE: none
/// CEP:ASSUMES: Non-string items are ignored deterministically.
/// CEP:COST: cold; linear in list length.
/// CEP:EVIDENCE: cep_lint self-lint.
fn param_list(rule: &Rule, key: &str) -> Vec<String> {
    rule.params.get(key)
        .and_then(|v| v.as_arr())
        .map(|items| items.iter().filter_map(|v| v.as_str().map(|s| s.to_string())).collect())
        .unwrap_or_default()
}

/// CEP:WHAT: Builds a finding by substituting the detail slot of the rule message.
/// CEP:WHY: Message templates live in configuration; checks supply one detail string.
/// CEP:STATUS: complete
/// CEP:FAILURE: none
/// CEP:ASSUMES: none
/// CEP:COST: cold; linear in template length.
/// CEP:EVIDENCE: cep_lint self-lint.
fn mk(rule: &Rule, file: &SourceFile, line: usize, detail: String) -> Finding {
    let msg = substitute(&rule.message, &[("detail", &detail)]);
    Finding { severity: rule.severity, rule: rule.id.clone(), file: file.path.clone(), line, message: msg }
}

/// CEP:WHAT: Returns the effective comment block of a function, rejecting the file header block.
/// CEP:WHY: Section 50.3 forbids file header blocks from serving as function blocks.
/// CEP:STATUS: complete
/// CEP:FAILURE: none
/// CEP:ASSUMES: Header identity is the start line; blocks are contiguous by construction.
/// CEP:COST: cold; constant time.
/// CEP:EVIDENCE: cep_lint self-lint.
fn effective_block(file: &SourceFile, f: &Function) -> Option<CommentBlock> {
    let header_start = file.header_block.as_ref().map(|b| b.start_line);
    match (f.block.clone(), header_start) {
        (Some(b), Some(hs)) if b.start_line == hs => None,
        (Some(b), _) => Some(b),
        (None, _) => None,
    }
}

/// CEP:WHAT: Checks that a function block exists and contains the required fields.
/// CEP:WHY: Sections 10.3 through 10.9 mandate complete blocks on nontrivial functions.
/// CEP:STATUS: complete
/// CEP:FAILURE: none
/// CEP:ASSUMES: Statement counting is the lexical semicolon model of scan.rs.
/// CEP:COST: cold; linear in functions times required fields.
/// CEP:EVIDENCE: self-test scenarios for CEP-LINT-FUNC-BLOCK.
pub fn check_block(rule: &Rule, file: &SourceFile) -> Vec<Finding> {
    let min_statements = param_int(rule, "min_body_statements", DEFAULT_MIN_STATEMENTS);
    let required = param_list(rule, "required_fields");
    let mut out = Vec::new();
    for f in file.functions.iter() {
        if (f.statement_count as i64) < min_statements {
            continue;
        }
        match effective_block(file, f) {
            None => out.push(mk(rule, file, f.decl_line, format!("function {} lacks a CEP comment block", f.name))),
            Some(b) => {
                if let Some(d) = missing_fields(&b, &required) {
                    out.push(mk(rule, file, f.decl_line, format!("function {} block missing fields: {}", f.name, d)));
                }
            }
        }
    }
    out
}

/// CEP:WHAT: Joins the names of required fields missing from one block.
/// CEP:WHY: One finding per function keeps counts deterministic and readable.
/// CEP:STATUS: complete
/// CEP:FAILURE: none
/// CEP:ASSUMES: none
/// CEP:COST: cold; linear in required fields.
/// CEP:EVIDENCE: cep_lint self-lint.
fn missing_fields(block: &CommentBlock, required: &[String]) -> Option<String> {
    let missing: Vec<&String> = required.iter().filter(|r| !block.fields.iter().any(|(k, _)| k == *r)).collect();
    if missing.is_empty() {
        None
    } else {
        Some(missing.iter().map(|s| s.as_str()).collect::<Vec<&str>>().join(", "))
    }
}

/// CEP:WHAT: Checks that function names are snake_case.
/// CEP:WHY: Section 33.13 mandates snake_case function names.
/// CEP:STATUS: complete
/// CEP:FAILURE: none
/// CEP:ASSUMES: The name pattern is validated at configuration load.
/// CEP:COST: cold; linear in functions.
/// CEP:EVIDENCE: self-test scenarios for CEP-LINT-NAMING-FUNC.
pub fn check_naming(rule: &Rule, file: &SourceFile) -> Vec<Finding> {
    let name_pattern = param_str(rule, "name_pattern", "^[a-z_][a-z0-9_]*$");
    let re = regex::Regex::new(&name_pattern).expect("naming regex validated at load");
    let mut out = Vec::new();
    for f in file.functions.iter() {
        if !re.is_match(&f.name) {
            out.push(mk(rule, file, f.decl_line, format!("function name {} is not snake_case", f.name)));
        }
    }
    out
}

/// CEP:WHAT: Checks that empty function bodies declare a sanctioned stub status.
/// CEP:WHY: Section 19 bans silent empty bodies outside declared stubs.
/// CEP:STATUS: complete
/// CEP:FAILURE: none
/// CEP:ASSUMES: none
/// CEP:COST: cold; linear in functions.
/// CEP:EVIDENCE: self-test scenarios for CEP-LINT-STUB-BODY.
pub fn check_stub_body(rule: &Rule, file: &SourceFile) -> Vec<Finding> {
    let status_field = param_str(rule, "suppress_status_field", "CEP:STATUS");
    let statuses = param_list(rule, "suppress_statuses");
    let mut out = Vec::new();
    for f in file.functions.iter() {
        if !f.body_is_empty {
            continue;
        }
        let suppressed = effective_block(file, f)
            .and_then(|b| b.fields.iter().find(|(k, _)| k == &status_field).map(|(_, v)| v.clone()))
            .map(|v| statuses.iter().any(|s| s == &v))
            .unwrap_or(false);
        if !suppressed {
            out.push(mk(rule, file, f.decl_line, format!("function {} has an empty body without a declared stub status", f.name)));
        }
    }
    out
}

/// CEP:WHAT: Checks that function bodies are not a single constant return.
/// CEP:WHY: Section 14.5 requires results computed from inputs.
/// CEP:STATUS: complete
/// CEP:FAILURE: none
/// CEP:ASSUMES: The constant-return shape is detected by scan.rs over the stripped body.
/// CEP:COST: cold; linear in functions.
/// CEP:EVIDENCE: self-test scenarios for CEP-LINT-CONSTANT-RETURN.
pub fn check_constant_return(rule: &Rule, file: &SourceFile) -> Vec<Finding> {
    let mut out = Vec::new();
    for f in file.functions.iter() {
        if f.constant_return {
            out.push(mk(rule, file, f.decl_line, format!("function {} body is a single constant return", f.name)));
        }
    }
    out
}

/// CEP:WHAT: Checks that long functions declare a documentation identifier.
/// CEP:WHY: Section 68.6.1 mandates identifiers for functions beyond the statement threshold.
/// CEP:STATUS: complete
/// CEP:FAILURE: none
/// CEP:ASSUMES: The identifier pattern is validated at configuration load.
/// CEP:COST: cold; linear in functions.
/// CEP:EVIDENCE: self-test scenarios for CEP-LINT-DOC-ID.
pub fn check_doc_id(rule: &Rule, file: &SourceFile) -> Vec<Finding> {
    let max_statements = param_int(rule, "max_id_statements", DEFAULT_MAX_ID_STATEMENTS);
    let id_pattern = param_str(rule, "id_pattern", "^[A-Z]+-[a-z0-9]+$");
    let re = regex::Regex::new(&id_pattern).expect("id regex validated at load");
    let mut out = Vec::new();
    for f in file.functions.iter() {
        if (f.statement_count as i64) <= max_statements {
            continue;
        }
        let id = effective_block(file, f)
            .and_then(|b| b.fields.iter().find(|(k, _)| k == "CEP:ID").map(|(_, v)| v.clone()));
        match id {
            None => out.push(mk(rule, file, f.decl_line, format!("function {} has {} statements but no CEP:ID", f.name, f.statement_count))),
            Some(v) if !re.is_match(&v) => out.push(mk(rule, file, f.decl_line, format!("function {} has malformed CEP:ID value {}", f.name, v))),
            Some(_) => {}
        }
    }
    out
}

/// CEP:WHAT: Dispatches a function_policy rule by its mode.
/// CEP:WHY: Mode selection keeps one check name per rule catalog entry.
/// CEP:STATUS: complete
/// CEP:FAILURE: none
/// CEP:ASSUMES: Unknown modes yield no findings, which configuration validation prevents.
/// CEP:COST: cold; constant time.
/// CEP:EVIDENCE: cep_lint self-lint.
pub fn check(rule: &Rule, file: &SourceFile, mode: Option<&str>) -> Vec<Finding> {
    match mode {
        Some("block") => check_block(rule, file),
        Some("naming") => check_naming(rule, file),
        Some("stub_body") => check_stub_body(rule, file),
        Some("constant_return") => check_constant_return(rule, file),
        Some("id") => check_doc_id(rule, file),
        _ => Vec::new(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::json::Value;

    /// Sample rule severity used by the block rule unit test.
    const TEST_SEVERITY: u8 = 1;

    /// Sample minimum statement threshold used by the block rule unit test.
    const TEST_MIN_STATEMENTS: f64 = 2.0;

/// CEP:WHAT: Verifies that the one-statement sample does not require a comment block.
/// CEP:WHY: The mechanical exemption boundary is the mandated approximation of nontrivial.
/// CEP:STATUS: complete
/// CEP:FAILURE: Fails the test binary on regression.
/// CEP:ASSUMES: none
/// CEP:COST: cold; unit test only.
/// CEP:EVIDENCE: cargo test -p cep_lint.
    #[test]
    fn one_statement_function_is_exempt() {
        let file = crate::scan::scan_file("a.rs", "fn tiny() { a(); }\n");
        let rule = Rule {
            id: "CEP-LINT-FUNC-BLOCK".into(), check: "function_policy".into(), mode: Some("block".into()),
            severity: TEST_SEVERITY, message: "function block: {detail}".into(),
            params: Value::Obj(vec![("min_body_statements".into(), Value::Num(TEST_MIN_STATEMENTS))]),
        };
        assert!(check_block(&rule, &file).is_empty());
    }
}
