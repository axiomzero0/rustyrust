// CEP:FILE: tools/cep_lint/src/secrets.rs
// CEP:WHAT: Secret probing over comment text and string literals.
// CEP:WHY: Rule CEP-LINT-SECRET-SCAN implements the lexical probe of sections 22.13 and 34.3.
// CEP:CLASS: CEP-2
// CEP:STATUS: complete
// CEP:FAILURE: none
// CEP:ASSUMES: Lexical probing only; obfuscated access material requires review per section 50.3.
// CEP:COST: cold; linear in probe patterns times scan units.
// CEP:EVIDENCE: self-test scenarios for CEP-LINT-SECRET-SCAN.

use crate::model::{Finding, Rule};
use crate::scan::SourceFile;
use std::path::Path;

/// CEP:WHAT: Applies secret-shaped patterns over comment segments and string literals.
/// CEP:WHY: Assignment-style credential occurrences and key material must never enter the repository.
/// CEP:STATUS: complete
/// CEP:FAILURE: none
/// CEP:ASSUMES: Patterns are case-insensitive; prose mentions without assignment context do not match.
/// CEP:COST: cold; linear in patterns times scan units.
/// CEP:EVIDENCE: self-test scenarios for CEP-LINT-SECRET-SCAN.
pub fn check(rule: &Rule, file: &SourceFile, _cwd: &Path) -> Vec<Finding> {
    if path_exempt(rule, &file.path) {
        return Vec::new();
    }
    let patterns = str_param(rule, "patterns");
    let mut out = Vec::new();
    for p in patterns.iter() {
        let re = regex::Regex::new(&format!("(?i){}", p)).expect("secret pattern validated at load");
        out.extend(probe_hits(rule, file, &re));
    }
    out
}

/// CEP:WHAT: Reports whether the file path matches a configured exempt path pattern.
/// CEP:WHY: Exempt paths are configuration data per section 50.3.
/// CEP:STATUS: complete
/// CEP:FAILURE: none
/// CEP:ASSUMES: Exempt patterns are validated at configuration load.
/// CEP:COST: cold; linear in exempt patterns.
/// CEP:EVIDENCE: cep_lint self-lint.
fn path_exempt(rule: &Rule, path: &str) -> bool {
    let exempt = str_param(rule, "exempt_paths");
    exempt.iter().any(|p| regex::Regex::new(p).expect("exempt path validated at load").is_match(path))
}

/// CEP:WHAT: Probes comment segments and string literals with one compiled pattern.
/// CEP:WHY: Both storage shapes can carry credential material.
/// CEP:STATUS: complete
/// CEP:FAILURE: none
/// CEP:ASSUMES: none
/// CEP:COST: cold; linear in scan units.
/// CEP:EVIDENCE: self-test scenarios for CEP-LINT-SECRET-SCAN.
fn probe_hits(rule: &Rule, file: &SourceFile, re: &regex::Regex) -> Vec<Finding> {
    let mut out = Vec::new();
    for seg in file.comment_segs.iter() {
        if re.is_match(&seg.text) {
            out.push(mk(rule, file, seg.line, "secret-shaped content in comment".to_string()));
        }
    }
    for lit in file.strings.iter() {
        if re.is_match(&lit.text) {
            out.push(mk(rule, file, lit.line, "secret-shaped content in string literal".to_string()));
        }
    }
    out
}

/// CEP:WHAT: Reads a string list parameter from rule params.
/// CEP:WHY: Pattern and exempt lists are configuration data.
/// CEP:STATUS: complete
/// CEP:FAILURE: none
/// CEP:ASSUMES: Non-string items are ignored deterministically.
/// CEP:COST: cold; linear in list length.
/// CEP:EVIDENCE: cep_lint self-lint.
fn str_param(rule: &Rule, key: &str) -> Vec<String> {
    rule.params.get(key)
        .and_then(|v| v.as_arr())
        .map(|items| items.iter().filter_map(|v| v.as_str().map(|s| s.to_string())).collect())
        .unwrap_or_default()
}

/// CEP:WHAT: Builds a secret scan finding.
/// CEP:WHY: Findings share one construction path for deterministic output.
/// CEP:STATUS: complete
/// CEP:FAILURE: none
/// CEP:ASSUMES: none
/// CEP:COST: cold; constant time.
/// CEP:EVIDENCE: cep_lint self-lint.
fn mk(rule: &Rule, file: &SourceFile, line: usize, detail: String) -> Finding {
    Finding {
        severity: rule.severity,
        rule: rule.id.clone(),
        file: file.path.clone(),
        line,
        message: crate::model::substitute(&rule.message, &[("detail", &detail)]),
    }
}
