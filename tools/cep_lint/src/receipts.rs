// CEP:FILE: tools/cep_lint/src/receipts.rs
// CEP:WHAT: Hunt and HUNTER receipt verification for the Bloodhound and HUNTER protocol rules.
// CEP:WHY: Rules CEP-LINT-HUNT-RECEIPT and CEP-LINT-HUNTER-RECEIPT are documented in the catalog and must be configured.
// CEP:CLASS: CEP-2
// CEP:STATUS: partial
// CEP:OWNER: axiomzero0
// CEP:TICKET: RR-102
// CEP:FAILURE: Produces severity zero findings when invoked with the corresponding flag and a receipt is missing or insufficient.
// CEP:ASSUMES: Receipt signature verification is limited to field presence until a trusted public key distribution path exists; documented refinement gap.
// CEP:COST: cold; linear in receipt size.
// CEP:EVIDENCE: CI hunt-receipt gate; refinement tracked under ticket RR-102.

use crate::json::Value;
use crate::model::{Finding, Rule};
use crate::scan::SourceFile;
use std::fs;
use std::path::Path;

/// Minimum fuzz executions demanded by the Bloodhound rule of chapter 65.
const MIN_FUZZ_EXECUTIONS: i64 = 100000;

/// Minimum differential inputs demanded by the Bloodhound rule of chapter 65.
const MIN_DIFF_INPUTS: i64 = 10000;

/// CEP:WHAT: Verifies the hunt receipt when the engine runs with the verify-hunt flag.
/// CEP:WHY: The Bloodhound protocol gates CEP-0 class changes on per-commit hunting evidence.
/// CEP:STATUS: complete
/// CEP:FAILURE: Severity zero finding for missing or insufficient receipts; no silent pass.
/// CEP:ASSUMES: The receipt lives at the configured path relative to the working directory.
/// CEP:COST: cold; linear in receipt size.
/// CEP:EVIDENCE: CI hunt-receipt gate.
pub fn check_hunt_receipt(rule: &Rule, file: &SourceFile, cwd: &Path) -> Vec<Finding> {
    let receipt_path = rule.params.get("receipt_path").and_then(|v| v.as_str()).unwrap_or(".cep/hunt_receipt.json").to_string();
    let min_fuzz = rule.params.get("min_fuzz_executions").and_then(|v| v.as_i64()).unwrap_or(MIN_FUZZ_EXECUTIONS);
    let min_diff = rule.params.get("min_diff_inputs").and_then(|v| v.as_i64()).unwrap_or(MIN_DIFF_INPUTS);
    let mut out = Vec::new();
    if !class_applies(rule, file) {
        return out;
    }
    let Some(text) = read_receipt(rule, file, cwd, &receipt_path, "hunt", &mut out) else { return out };
    let Some(v) = parse_receipt(rule, file, &text, "hunt", &mut out) else { return out };
    check_hunt_fields(rule, file, &v, min_fuzz, min_diff, &mut out);
    out
}

/// CEP:WHAT: Reads one receipt file or records a severity zero finding.
/// CEP:WHY: Missing receipts block merges under both protocol rules.
/// CEP:STATUS: complete
/// CEP:FAILURE: Pushes a finding and returns None when the file is unreadable.
/// CEP:ASSUMES: none
/// CEP:COST: cold; linear in receipt size.
/// CEP:EVIDENCE: CI receipt gates.
fn read_receipt(rule: &Rule, file: &SourceFile, cwd: &Path, path: &str, kind: &str, out: &mut Vec<Finding>) -> Option<String> {
    match fs::read_to_string(cwd.join(path)) {
        Ok(t) => Some(t),
        Err(_) => {
            out.push(receipt_finding(rule, file, format!("{} receipt {} is missing; the protocol blocks this merge", kind, path)));
            None
        }
    }
}

/// CEP:WHAT: Parses one receipt document or records a severity zero finding.
/// CEP:WHY: Malformed receipts cannot be trusted as evidence.
/// CEP:STATUS: complete
/// CEP:FAILURE: Pushes a finding and returns None when parsing fails.
/// CEP:ASSUMES: none
/// CEP:COST: cold; linear in receipt size.
/// CEP:EVIDENCE: CI receipt gates.
fn parse_receipt(rule: &Rule, file: &SourceFile, text: &str, kind: &str, out: &mut Vec<Finding>) -> Option<Value> {
    match crate::json::parse(text) {
        Ok(v) => Some(v),
        Err(_) => {
            out.push(receipt_finding(rule, file, format!("{} receipt is not valid JSON", kind)));
            None
        }
    }
}

/// CEP:WHAT: Checks the numeric and divergence fields of one hunt receipt.
/// CEP:WHY: Chapter 65 mandates fuzz executions, diff inputs, and zero divergences.
/// CEP:STATUS: complete
/// CEP:FAILURE: One severity zero finding per failed field check.
/// CEP:ASSUMES: none
/// CEP:COST: cold; constant time.
/// CEP:EVIDENCE: CI hunt-receipt gate.
fn check_hunt_fields(rule: &Rule, file: &SourceFile, v: &Value, min_fuzz: i64, min_diff: i64, out: &mut Vec<Finding>) {
    let fuzz = v.get("fuzz_executions").and_then(|x| x.as_i64()).unwrap_or(0);
    let diff = v.get("diff_inputs").and_then(|x| x.as_i64()).unwrap_or(0);
    let divergences = v.get("divergences").and_then(|x| x.as_i64()).unwrap_or(i64::MAX);
    if fuzz < min_fuzz {
        out.push(receipt_finding(rule, file, format!("hunt receipt fuzz_executions {} is below {}", fuzz, min_fuzz)));
    }
    if diff < min_diff {
        out.push(receipt_finding(rule, file, format!("hunt receipt diff_inputs {} is below {}", diff, min_diff)));
    }
    if divergences != 0 {
        out.push(receipt_finding(rule, file, format!("hunt receipt divergences {} is not zero", divergences)));
    }
    if v.get("signature").and_then(|x| x.as_str()).is_none() {
        out.push(receipt_finding(rule, file, "hunt receipt lacks a signature field".to_string()));
    }
}

/// CEP:WHAT: Verifies the HUNTER receipt when the engine runs with the verify-hunter flag.
/// CEP:WHY: The HUNTER protocol gates CEP-0 class changes on bounded symbolic analysis evidence.
/// CEP:STATUS: complete
/// CEP:FAILURE: Severity zero finding for missing or insufficient receipts; no silent pass.
/// CEP:ASSUMES: The receipt lives at the configured path relative to the working directory.
/// CEP:COST: cold; linear in receipt size.
/// CEP:EVIDENCE: CI hunter-receipt gate.
pub fn check_hunter_receipt(rule: &Rule, file: &SourceFile, cwd: &Path) -> Vec<Finding> {
    let receipt_path = rule.params.get("receipt_path").and_then(|v| v.as_str()).unwrap_or(".cep/hunter_receipt.json").to_string();
    let mut out = Vec::new();
    if !class_applies(rule, file) {
        return out;
    }
    let Some(text) = read_receipt(rule, file, cwd, &receipt_path, "HUNTER", &mut out) else { return out };
    let Some(v) = parse_receipt(rule, file, &text, "HUNTER", &mut out) else { return out };
    let unproven = v.get("unproven_paths").and_then(|x| x.as_i64()).unwrap_or(i64::MAX);
    if unproven != 0 {
        out.push(receipt_finding(rule, file, format!("HUNTER receipt unproven_paths {} is not zero", unproven)));
    }
    if v.get("functions_analyzed").and_then(|x| x.as_arr()).is_none() {
        out.push(receipt_finding(rule, file, "HUNTER receipt lacks a functions_analyzed array".to_string()));
    }
    if v.get("signature").and_then(|x| x.as_str()).is_none() {
        out.push(receipt_finding(rule, file, "HUNTER receipt lacks a signature field".to_string()));
    }
    out
}

/// CEP:WHAT: Reports whether the scanned file's declared class is covered by the receipt rule.
/// CEP:WHY: Both receipt rules target cycle-exact classes only.
/// CEP:STATUS: complete
/// CEP:FAILURE: none
/// CEP:ASSUMES: none
/// CEP:COST: cold; linear in class list length.
/// CEP:EVIDENCE: CI receipt gates.
fn class_applies(rule: &Rule, file: &SourceFile) -> bool {
    let applies = rule.params.get("applies_to_classes")
        .and_then(|v| v.as_arr())
        .map(|items| items.iter().filter_map(|x| x.as_str().map(|s| s.to_string())).collect::<Vec<String>>())
        .unwrap_or_default();
    file.class.as_ref().map(|c| applies.iter().any(|a| a == c)).unwrap_or(false)
}

/// CEP:WHAT: Builds a receipt finding anchored at the file header line.
/// CEP:WHY: Receipt violations point at the guarded file for actionable output.
/// CEP:STATUS: complete
/// CEP:FAILURE: none
/// CEP:ASSUMES: none
/// CEP:COST: cold; constant time.
/// CEP:EVIDENCE: CI receipt gates.
fn receipt_finding(rule: &Rule, file: &SourceFile, detail: String) -> Finding {
    Finding {
        severity: rule.severity,
        rule: rule.id.clone(),
        file: file.path.clone(),
        line: 0,
        message: crate::model::substitute(&rule.message, &[("detail", &detail)]),
    }
}
