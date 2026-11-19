// CEP:FILE: tools/cep_lint/src/unsafedoc.rs
// CEP:WHAT: Unsafe usage documentation check.
// CEP:WHY: Rule CEP-LINT-UNSAFE-DOC requires a CEP:UNSAFE field on every unsafe usage per sections 22.7 and 25.7.
// CEP:CLASS: CEP-2
// CEP:STATUS: complete
// CEP:FAILURE: none
// CEP:ASSUMES: A file header CEP:UNSAFE field documents file-wide usage; otherwise the nearest preceding block must document it.
// CEP:COST: cold; linear in unsafe occurrences.
// CEP:EVIDENCE: self-test scenarios for CEP-LINT-UNSAFE-DOC.

use crate::model::{Finding, Rule};
use crate::scan::SourceFile;
use std::path::Path;

/// Word-bounded pattern for unsafe usage in stripped code.
const UNSAFE_TOKEN: &str = r"\bunsafe\b";

/// CEP:WHAT: Flags undocumented unsafe occurrences in stripped code.
/// CEP:WHY: Every unsafe usage must carry a documented invariant at the block or file level.
/// CEP:STATUS: complete
/// CEP:FAILURE: none
/// CEP:ASSUMES: none
/// CEP:COST: cold; linear in file lines.
/// CEP:EVIDENCE: self-test scenarios for CEP-LINT-UNSAFE-DOC.
pub fn check(rule: &Rule, file: &SourceFile, _cwd: &Path) -> Vec<Finding> {
    if path_exempt(rule, &file.path) {
        return Vec::new();
    }
    let re = regex::Regex::new(UNSAFE_TOKEN).expect("unsafe token regex");
    let header_documented = file.header_block.as_ref()
        .map(|b| b.fields.iter().any(|(k, _)| k == "CEP:UNSAFE"))
        .unwrap_or(false);
    let mut out = Vec::new();
    for (idx, line) in file.stripped_lines.iter().enumerate() {
        if re.is_match(line) && !header_documented && !block_above_documents(file, idx + 1) {
            out.push(occurrence_finding(rule, file, idx + 1));
        }
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
    let exempt = rule.params.get("exempt_paths")
        .and_then(|v| v.as_arr())
        .map(|items| items.iter().filter_map(|v| v.as_str().map(|s| s.to_string())).collect::<Vec<String>>())
        .unwrap_or_default();
    exempt.iter().any(|p| regex::Regex::new(p).expect("exempt path validated at load").is_match(path))
}

/// CEP:WHAT: Reports whether the nearest preceding comment block documents unsafe usage.
/// CEP:WHY: Block-level documentation is the sanctioned alternative to file headers.
/// CEP:STATUS: complete
/// CEP:FAILURE: none
/// CEP:ASSUMES: none
/// CEP:COST: cold; linear in preceding blocks.
/// CEP:EVIDENCE: self-test scenarios for CEP-LINT-UNSAFE-DOC.
fn block_above_documents(file: &SourceFile, line: usize) -> bool {
    file.blocks.iter().rev()
        .find(|b| b.end_line < line)
        .map(|b| b.fields.iter().any(|(k, _)| k == "CEP:UNSAFE"))
        .unwrap_or(false)
}

/// CEP:WHAT: Builds one undocumented unsafe finding.
/// CEP:WHY: Findings share one construction path for deterministic output.
/// CEP:STATUS: complete
/// CEP:FAILURE: none
/// CEP:ASSUMES: none
/// CEP:COST: cold; constant time.
/// CEP:EVIDENCE: cep_lint self-lint.
fn occurrence_finding(rule: &Rule, file: &SourceFile, line: usize) -> Finding {
    Finding {
        severity: rule.severity,
        rule: rule.id.clone(),
        file: file.path.clone(),
        line,
        message: crate::model::substitute(&rule.message, &[("detail", "unsafe usage lacks a CEP:UNSAFE field")]),
    }
}
