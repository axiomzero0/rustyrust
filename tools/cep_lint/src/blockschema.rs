// CEP:FILE: tools/cep_lint/src/blockschema.rs
// CEP:WHAT: Comment block schema checks for file headers, status values, and stub requirements.
// CEP:WHY: Rules CEP-LINT-FILE-HEADER, CEP-LINT-STATUS-VALUE, and CEP-LINT-STUB-DECL share the block field model.
// CEP:CLASS: CEP-2
// CEP:STATUS: complete
// CEP:FAILURE: none; checks are total over scanned files.
// CEP:ASSUMES: Blocks are full-line comment blocks as built by scan.rs.
// CEP:COST: cold; linear in blocks per file.
// CEP:EVIDENCE: self-test scenario set for the three comment_block_schema rules.

use crate::model::{substitute, Finding, Rule};
use crate::scan::SourceFile;

/// Fallback line limit for the file header start position.
const DEFAULT_MAX_START: i64 = 3;

/// CEP:WHAT: Reads a string list parameter from rule params.
/// CEP:WHY: Field and class lists are configuration data.
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

/// CEP:WHAT: Checks that the first block is a complete file header within the start limit.
/// CEP:WHY: Section 35 mandates file-level CEP blocks as the first comment block.
/// CEP:STATUS: complete
/// CEP:FAILURE: none; produces findings for missing headers, missing fields, and invalid classes.
/// CEP:ASSUMES: The header is the first contiguous full-line comment block; content above it is a violation by design.
/// CEP:COST: cold; linear in required fields.
/// CEP:EVIDENCE: self-test scenarios for CEP-LINT-FILE-HEADER.
pub fn check_file_header(rule: &Rule, file: &SourceFile) -> Vec<Finding> {
    let required = param_list(rule, "required_fields");
    let class_field = rule.params.get("class_field").and_then(|v| v.as_str()).unwrap_or("CEP:CLASS").to_string();
    let allowed = param_list(rule, "allowed_classes");
    let max_start = rule.params.get("max_start_line").and_then(|v| v.as_i64()).unwrap_or(DEFAULT_MAX_START);
    let mut out = Vec::new();
    let Some(header) = file.header_block.as_ref() else {
        out.push(mk(rule, file, 0, "missing file header block".to_string()));
        return out;
    };
    out.extend(header_findings(rule, file, header, &required, &class_field, &allowed, max_start));
    out
}

/// CEP:WHAT: Produces header findings for one present header block.
/// CEP:WHY: Splitting the header validation keeps the entry check within statement limits.
/// CEP:STATUS: complete
/// CEP:FAILURE: none
/// CEP:ASSUMES: none
/// CEP:COST: cold; linear in required fields plus allowed classes.
/// CEP:EVIDENCE: self-test scenarios for CEP-LINT-FILE-HEADER.
fn header_findings(
    rule: &Rule,
    file: &SourceFile,
    header: &crate::scan::CommentBlock,
    required: &[String],
    class_field: &str,
    allowed: &[String],
    max_start: i64,
) -> Vec<Finding> {
    let mut out = Vec::new();
    if (header.start_line as i64) > max_start {
        out.push(mk(rule, file, header.start_line, format!("file header starts at line {} beyond the limit {}", header.start_line, max_start)));
    }
    let missing: Vec<&String> = required.iter().filter(|r| !header.fields.iter().any(|(k, _)| k == *r)).collect();
    if !missing.is_empty() {
        let joined = missing.iter().map(|s| s.as_str()).collect::<Vec<&str>>().join(", ");
        out.push(mk(rule, file, header.start_line, format!("file header missing fields: {}", joined)));
    }
    let class_ok = header.fields.iter()
        .find(|(k, _)| k == class_field)
        .map(|(_, v)| allowed.iter().any(|a| a == v));
    if class_ok == Some(false) {
        out.push(mk(rule, file, header.start_line, format!("file header declares an unknown class in {}", class_field)));
    }
    out
}

/// CEP:WHAT: Checks that every status value in every block is a sanctioned status word.
/// CEP:WHY: Section 10.5 restricts status values to four words with defined meanings.
/// CEP:STATUS: complete
/// CEP:FAILURE: none
/// CEP:ASSUMES: none
/// CEP:COST: cold; linear in blocks and fields.
/// CEP:EVIDENCE: self-test scenarios for CEP-LINT-STATUS-VALUE.
pub fn check_status_value(rule: &Rule, file: &SourceFile) -> Vec<Finding> {
    let status_field = rule.params.get("status_field").and_then(|v| v.as_str()).unwrap_or("CEP:STATUS").to_string();
    let allowed = param_list(rule, "allowed_values");
    let mut out = Vec::new();
    for b in file.blocks.iter() {
        for (k, v) in b.fields.iter() {
            if k == &status_field && !allowed.iter().any(|a| a == v) {
                out.push(mk(rule, file, b.start_line, format!("status value {} is not one of {}", v, allowed.join(", "))));
            }
        }
    }
    out
}

/// CEP:WHAT: Checks that stub, partial, and reserved-status blocks declare owner and ticket.
/// CEP:WHY: Anonymous stubs are banned by sections 10.5 and 10.10.
/// CEP:STATUS: complete
/// CEP:FAILURE: none
/// CEP:ASSUMES: none
/// CEP:COST: cold; linear in blocks and fields.
/// CEP:EVIDENCE: self-test scenarios for CEP-LINT-STUB-DECL.
pub fn check_stub_requirements(rule: &Rule, file: &SourceFile) -> Vec<Finding> {
    let status_field = rule.params.get("status_field").and_then(|v| v.as_str()).unwrap_or("CEP:STATUS").to_string();
    let statuses = param_list(rule, "statuses");
    let required = param_list(rule, "required_fields");
    let mut out = Vec::new();
    for b in file.blocks.iter() {
        let status = b.fields.iter().find(|(k, _)| k == &status_field).map(|(_, v)| v.clone());
        if let Some(status) = status {
            if statuses.iter().any(|s| s == &status) {
                out.extend(missing_owner_fields(rule, file, b, &status, &required));
            }
        }
    }
    out
}

/// CEP:WHAT: Reports required ownership fields missing from one stub-status block.
/// CEP:WHY: One finding per missing field gives precise repair instructions.
/// CEP:STATUS: complete
/// CEP:FAILURE: none
/// CEP:ASSUMES: none
/// CEP:COST: cold; linear in required fields.
/// CEP:EVIDENCE: self-test scenarios for CEP-LINT-STUB-DECL.
fn missing_owner_fields(rule: &Rule, file: &SourceFile, b: &crate::scan::CommentBlock, status: &str, required: &[String]) -> Vec<Finding> {
    let mut out = Vec::new();
    for req in required.iter() {
        if !b.fields.iter().any(|(k, _)| k == req) {
            out.push(mk(rule, file, b.start_line, format!("status {} requires field {} in the same block", status, req)));
        }
    }
    out
}

/// CEP:WHAT: Dispatches a comment_block_schema rule by its mode.
/// CEP:WHY: Mode selection keeps one check name per rule catalog entry.
/// CEP:STATUS: complete
/// CEP:FAILURE: none
/// CEP:ASSUMES: Unknown modes yield no findings, which configuration validation prevents.
/// CEP:COST: cold; constant time.
/// CEP:EVIDENCE: cep_lint self-lint.
pub fn check(rule: &Rule, file: &SourceFile, mode: Option<&str>) -> Vec<Finding> {
    match mode {
        Some("file_header") => check_file_header(rule, file),
        Some("status_value") => check_status_value(rule, file),
        Some("stub_requirements") => check_stub_requirements(rule, file),
        _ => Vec::new(),
    }
}
