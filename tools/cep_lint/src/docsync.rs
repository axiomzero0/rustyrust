// CEP:FILE: tools/cep_lint/src/docsync.rs
// CEP:WHAT: Documentation synchronization between the standard document and the lint configuration.
// CEP:WHY: Rule CEP-LINT-DOC-SYNC enforces Law 8 mechanically in both coverage directions.
// CEP:CLASS: CEP-2
// CEP:STATUS: complete
// CEP:FAILURE: Returns bootstrap errors when the document is unreadable or patterns are invalid.
// CEP:ASSUMES: doc_path resolves against the process working directory.
// CEP:COST: cold; linear in document size.
// CEP:EVIDENCE: live gate over standards.md and .cep/cep_lint.json in CI.

use crate::model::{BootstrapError, Config, Finding, Rule};
use std::fs;
use std::path::Path;

/// Bootstrap code for file input failures.
const BOOT_IO: u8 = 1;

/// Bootstrap code for invalid regular expressions.
const BOOT_REGEX: u8 = 8;

/// Heading record extracted from the document.
struct DocHeading {
    rule_id: String,
    line: usize,
}

/// CEP:WHAT: Verifies rule coverage and version agreement between document and configuration.
/// CEP:WHY: A rule without a section and a documented rule without configuration are both lint failures.
/// CEP:ID: CS-a08
/// CEP:STATUS: complete
/// CEP:FAILURE: Returns a bootstrap error when the document cannot be read; findings for drift.
/// CEP:ASSUMES: The version line uses the configured version pattern with one capture group.
/// CEP:COST: cold; linear in document size plus rules.
/// CEP:EVIDENCE: doc file docs/functions/CS/CS-a08.md; CI doc-sync job.
pub fn check(rule: &Rule, config: &Config, cwd: &Path) -> Result<Vec<Finding>, BootstrapError> {
    let doc_path = rule.params.get("doc_path").and_then(|v| v.as_str()).unwrap_or("standards.md").to_string();
    let heading_pattern = rule.params.get("heading_pattern").and_then(|v| v.as_str()).unwrap_or("^### Rule (CEP-LINT-[A-Z0-9-]+)$").to_string();
    let text = fs::read_to_string(cwd.join(&doc_path))
        .map_err(|e| BootstrapError::new(BOOT_IO, format!("document {} unreadable: {}", doc_path, e)))?;
    let headings = extract_headings(&text, &heading_pattern)?;
    let mut out = Vec::new();
    out.extend(missing_config_rules(rule, &doc_path, &headings, config, &text));
    out.extend(orphans_and_duplicates(rule, &doc_path, &headings, config));
    if let Some(vp) = rule.params.get("version_pattern").and_then(|v| v.as_str()) {
        out.extend(check_version(rule, config, &text, &doc_path, vp)?);
    }
    Ok(out)
}

/// CEP:WHAT: Reports configured rules that lack a Rule heading in the document.
/// CEP:WHY: Coverage is mechanical in both directions per the DOC-SYNC contract.
/// CEP:STATUS: complete
/// CEP:FAILURE: none
/// CEP:ASSUMES: none
/// CEP:COST: cold; linear in rules.
/// CEP:EVIDENCE: CI doc-sync job.
fn missing_config_rules(rule: &Rule, doc_path: &str, headings: &[DocHeading], config: &Config, text: &str) -> Vec<Finding> {
    let mut out = Vec::new();
    for r in config.rules.iter() {
        if !headings.iter().any(|h| h.rule_id == r.id) {
            out.push(finding(rule, doc_path, version_line(text), format!("configured rule {} has no Rule heading in {}", r.id, doc_path)));
        }
    }
    out
}

/// CEP:WHAT: Reports documented rules absent from configuration and duplicate headings.
/// CEP:WHY: Drift in either direction is a lint failure per Law 8.
/// CEP:STATUS: complete
/// CEP:FAILURE: none
/// CEP:ASSUMES: none
/// CEP:COST: cold; linear in headings.
/// CEP:EVIDENCE: CI doc-sync job.
fn orphans_and_duplicates(rule: &Rule, doc_path: &str, headings: &[DocHeading], config: &Config) -> Vec<Finding> {
    let mut out = Vec::new();
    let mut seen: Vec<String> = Vec::new();
    for h in headings.iter() {
        if seen.contains(&h.rule_id) {
            out.push(finding(rule, doc_path, h.line, format!("duplicate Rule heading for {}", h.rule_id)));
        }
        seen.push(h.rule_id.clone());
        if !config.rules.iter().any(|r| r.id == h.rule_id) {
            out.push(finding(rule, doc_path, h.line, format!("documented rule {} is absent from configuration", h.rule_id)));
        }
    }
    out
}

/// CEP:WHAT: Verifies the document version line against the configuration standard version.
/// CEP:WHY: The revision history records drift between document and configuration versions.
/// CEP:STATUS: complete
/// CEP:FAILURE: Bootstrap error eight for invalid version patterns.
/// CEP:ASSUMES: The pattern contains one capture group for the version text.
/// CEP:COST: cold; linear in document lines.
/// CEP:EVIDENCE: CI doc-sync job.
fn check_version(rule: &Rule, config: &Config, text: &str, doc_path: &str, version_pattern: &str) -> Result<Vec<Finding>, BootstrapError> {
    let vre = regex::Regex::new(version_pattern).map_err(|e| BootstrapError::new(BOOT_REGEX, format!("version pattern: {}", e)))?;
    let doc_version = find_version(text, &vre);
    let mut out = Vec::new();
    match doc_version {
        None => out.push(finding(rule, doc_path, 1, "document has no version line matching the configured pattern".to_string())),
        Some(v) if v != config.standard_version => {
            out.push(finding(rule, doc_path, version_line(text), format!("document version {} disagrees with configuration standard version {}", v, config.standard_version)));
        }
        Some(_) => {}
    }
    Ok(out)
}

/// CEP:WHAT: Extracts the first version capture group from the document text.
/// CEP:WHY: The first matching line is the version line.
/// CEP:STATUS: complete
/// CEP:FAILURE: none
/// CEP:ASSUMES: none
/// CEP:COST: cold; linear in document lines.
/// CEP:EVIDENCE: CI doc-sync job.
fn find_version(text: &str, vre: &regex::Regex) -> Option<String> {
    for line in text.lines() {
        if let Some(c) = vre.captures(line) {
            return Some(c.get(1).map(|m| m.as_str().to_string()).unwrap_or_default());
        }
    }
    None
}

/// CEP:WHAT: Extracts rule headings from the document text.
/// CEP:WHY: Heading extraction is the coverage anchor for both directions.
/// CEP:STATUS: complete
/// CEP:FAILURE: Bootstrap error eight for invalid heading patterns.
/// CEP:ASSUMES: none
/// CEP:COST: cold; linear in document lines.
/// CEP:EVIDENCE: CI doc-sync job.
fn extract_headings(text: &str, pattern: &str) -> Result<Vec<DocHeading>, BootstrapError> {
    let re = regex::Regex::new(pattern).map_err(|e| BootstrapError::new(BOOT_REGEX, format!("heading pattern: {}", e)))?;
    let mut out = Vec::new();
    for (idx, line) in text.lines().enumerate() {
        if let Some(c) = re.captures(line) {
            if let Some(m) = c.get(1) {
                out.push(DocHeading { rule_id: m.as_str().to_string(), line: idx + 1 });
            }
        }
    }
    Ok(out)
}

/// CEP:WHAT: Returns the one-based line of the version line, or line one when absent.
/// CEP:WHY: Version findings anchor at the version line for readable output.
/// CEP:STATUS: complete
/// CEP:FAILURE: none
/// CEP:ASSUMES: none
/// CEP:COST: cold; linear in document lines.
/// CEP:EVIDENCE: CI doc-sync job.
fn version_line(text: &str) -> usize {
    for (idx, line) in text.lines().enumerate() {
        if line.starts_with("Version:") {
            return idx + 1;
        }
    }
    1
}

/// CEP:WHAT: Builds a DOC-SYNC finding.
/// CEP:WHY: Findings share one construction path for deterministic output.
/// CEP:STATUS: complete
/// CEP:FAILURE: none
/// CEP:ASSUMES: none
/// CEP:COST: cold; constant time.
/// CEP:EVIDENCE: CI doc-sync job.
fn finding(rule: &Rule, doc_path: &str, line: usize, detail: String) -> Finding {
    Finding {
        severity: rule.severity,
        rule: rule.id.clone(),
        file: doc_path.to_string(),
        line,
        message: crate::model::substitute(&rule.message, &[("detail", &detail)]),
    }
}
