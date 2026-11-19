// CEP:FILE: tools/cep_lint/src/topology.rs
// CEP:WHAT: File topology policy: class-based line limits and junk drawer name rejection.
// CEP:WHY: Rule CEP-LINT-ATOMIC-FILE enforces chapter 62 mechanically.
// CEP:CLASS: CEP-2
// CEP:STATUS: complete
// CEP:FAILURE: none
// CEP:ASSUMES: The declared class comes from the file header; a missing class falls back to the default limit.
// CEP:COST: cold; constant per file.
// CEP:EVIDENCE: self-test scenarios for CEP-LINT-ATOMIC-FILE.

use crate::model::{substitute, Finding, Rule};
use crate::scan::SourceFile;

/// Fallback line limit for files without a declared class.
const DEFAULT_LINE_LIMIT: i64 = 800;

/// CEP:WHAT: Checks line counts against class limits and rejects junk drawer file names.
/// CEP:WHY: Chapter 62 mandates atomic files with mechanical limits.
/// CEP:STATUS: complete
/// CEP:FAILURE: none
/// CEP:ASSUMES: Generated-code exemption relies on directory policy, not detection.
/// CEP:COST: cold; constant per file.
/// CEP:EVIDENCE: self-test scenarios for CEP-LINT-ATOMIC-FILE.
pub fn check(rule: &Rule, file: &SourceFile) -> Vec<Finding> {
    let mut out = Vec::new();
    if let Some(f) = line_limit_finding(rule, file) {
        out.push(f);
    }
    out.extend(junk_name_findings(rule, file));
    out
}

/// CEP:WHAT: Produces a finding when the file exceeds its class line limit.
/// CEP:WHY: File size limits are the mechanical form of the atomic file mandate.
/// CEP:STATUS: complete
/// CEP:FAILURE: none
/// CEP:ASSUMES: The declared class is taken from the file header.
/// CEP:COST: cold; constant time.
/// CEP:EVIDENCE: self-test scenarios for CEP-LINT-ATOMIC-FILE.
fn line_limit_finding(rule: &Rule, file: &SourceFile) -> Option<Finding> {
    let default_limit = rule.params.get("default_limit").and_then(|v| v.as_i64()).unwrap_or(DEFAULT_LINE_LIMIT);
    let class_limit = file.class.as_ref()
        .and_then(|c| rule.params.get("line_limits").and_then(|l| l.get(c)).and_then(|v| v.as_i64()))
        .unwrap_or(default_limit);
    let line_count = file.lines.len() as i64;
    if line_count > class_limit {
        let detail = format!("file has {} lines beyond the class limit {}", line_count, class_limit);
        return Some(Finding {
            severity: rule.severity,
            rule: rule.id.clone(),
            file: file.path.clone(),
            line: 0,
            message: substitute(&rule.message, &[("detail", &detail)]),
        });
    }
    None
}

/// CEP:WHAT: Produces findings when the file stem contains a junk drawer marker.
/// CEP:WHY: Junk drawer names hide dead code and unmaintained assumptions.
/// CEP:STATUS: complete
/// CEP:FAILURE: none
/// CEP:ASSUMES: Markers are matched case-insensitively against the file stem.
/// CEP:COST: cold; linear in markers.
/// CEP:EVIDENCE: self-test scenarios for CEP-LINT-ATOMIC-FILE.
fn junk_name_findings(rule: &Rule, file: &SourceFile) -> Vec<Finding> {
    let junk = rule.params.get("junk_names")
        .and_then(|v| v.as_arr())
        .map(|items| items.iter().filter_map(|v| v.as_str().map(|s| s.to_string())).collect::<Vec<String>>())
        .unwrap_or_default();
    let stem = stem_of(&file.path);
    let mut out = Vec::new();
    for j in junk.iter() {
        if stem.contains(j) {
            let detail = format!("file name contains junk drawer marker {}", j);
            out.push(Finding {
                severity: rule.severity,
                rule: rule.id.clone(),
                file: file.path.clone(),
                line: 0,
                message: substitute(&rule.message, &[("detail", &detail)]),
            });
        }
    }
    out
}

/// CEP:WHAT: Extracts the lowercased file stem from a path.
/// CEP:WHY: Junk drawer matching applies to the file name only.
/// CEP:STATUS: complete
/// CEP:FAILURE: none
/// CEP:ASSUMES: none
/// CEP:COST: cold; linear in name length.
/// CEP:EVIDENCE: cep_lint self-lint.
fn stem_of(path: &str) -> String {
    let name = path.rsplit('/').next().unwrap_or(path);
    name.split('.').next().unwrap_or(name).to_lowercase()
}
