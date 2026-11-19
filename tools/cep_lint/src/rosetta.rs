// CEP:FILE: tools/cep_lint/src/rosetta.rs
// CEP:WHAT: Rosetta protocol checks: registry agreement, dual-tier documentation, and inquisition sections.
// CEP:WHY: Rules CEP-LINT-DOC-FILE, CEP-LINT-DOC-TIERS, and CEP-LINT-DOC-INQUISITION enforce chapter 68.
// CEP:CLASS: CEP-2
// CEP:STATUS: complete
// CEP:FAILURE: Returns a bootstrap error when a configured registry is missing; findings for drift.
// CEP:ASSUMES: The registry lists identifiers as strings; documentation paths derive from the domain prefix.
// CEP:COST: cold; linear in registry size plus scanned identifiers.
// CEP:EVIDENCE: self-test scenarios for the rosetta rules; CI registry gate.

use crate::model::{BootstrapError, Finding, Rule};
use crate::scan::SourceFile;
use std::collections::BTreeSet;
use std::fs;
use std::path::Path;

/// Bootstrap code for file input failures.
const BOOT_IO: u8 = 1;

/// Bootstrap code for JSON parse failures.
const BOOT_JSON: u8 = 2;

/// Required heading for the beginner tier.
const TIER_ONE: &str = "## Tier 1: Beginner";

/// Required heading for the silicon reality tier.
const TIER_TWO: &str = "## Tier 2: Silicon Reality";

/// Required heading for the silent failure checklist.
const INQUISITION: &str = "## Silent Failure Checklist";

/// Default documentation prefix for identifier files.
const DOCS_PREFIX: &str = "docs/functions";

/// Default registry path relative to the working directory.
const REGISTRY_PATH: &str = "docs/registry.json";

/// CEP:WHAT: Collects all CEP:ID values declared in scanned files.
/// CEP:WHY: Registry agreement needs the union of declarations across the repository.
/// CEP:STATUS: complete
/// CEP:FAILURE: none
/// CEP:ASSUMES: none
/// CEP:COST: cold; linear in blocks and fields.
/// CEP:EVIDENCE: CI registry gate.
fn collect_ids(files: &[SourceFile]) -> BTreeSet<String> {
    let mut ids = BTreeSet::new();
    for f in files.iter() {
        for b in f.blocks.iter() {
            for (k, v) in b.fields.iter() {
                if k == "CEP:ID" {
                    ids.insert(v.clone());
                }
            }
        }
    }
    ids
}

/// CEP:WHAT: Loads the registry identifier set from the registry path.
/// CEP:WHY: Registry agreement is a two-directional mechanical check.
/// CEP:STATUS: complete
/// CEP:FAILURE: Bootstrap errors one and two for unreadable or malformed registries.
/// CEP:ASSUMES: The registry uses the functions array of identifier strings.
/// CEP:COST: cold; linear in registry size.
/// CEP:EVIDENCE: CI registry gate.
fn load_registry(path: &Path) -> Result<BTreeSet<String>, BootstrapError> {
    let text = fs::read_to_string(path)
        .map_err(|e| BootstrapError::new(BOOT_IO, format!("registry {} unreadable: {}", path.display(), e)))?;
    let value = crate::json::parse(&text)
        .map_err(|e| BootstrapError::new(BOOT_JSON, format!("registry {}: {}", path.display(), e.detail)))?;
    let mut ids = BTreeSet::new();
    if let Some(arr) = value.get("functions").and_then(|v| v.as_arr()) {
        for item in arr {
            if let Some(s) = item.as_str() {
                ids.insert(s.to_string());
            }
        }
    }
    Ok(ids)
}

/// CEP:WHAT: Verifies that declared identifiers and registry entries agree in both directions.
/// CEP:WHY: A function with an unregistered identifier and a registry entry without documentation are both defects.
/// CEP:STATUS: complete
/// CEP:FAILURE: Bootstrap error when the registry is configured but missing.
/// CEP:ASSUMES: Documentation paths follow docs/functions/DOMAIN/ID.md.
/// CEP:COST: cold; linear in identifiers.
/// CEP:EVIDENCE: CI registry gate; self-test scenario for CEP-LINT-DOC-FILE.
pub fn check_doc_file(rule: &Rule, files: &[SourceFile], cwd: &Path) -> Result<Vec<Finding>, BootstrapError> {
    let registry_path = rule.params.get("registry_path").and_then(|v| v.as_str()).unwrap_or(REGISTRY_PATH).to_string();
    let docs_prefix = rule.params.get("docs_prefix").and_then(|v| v.as_str()).unwrap_or(DOCS_PREFIX).to_string();
    let registry = load_registry(&cwd.join(&registry_path))?;
    let declared = collect_ids(files);
    let mut out = Vec::new();
    for id in declared.iter() {
        if !registry.contains(id) {
            out.push(mk(rule, id, format!("identifier {} is declared in code but absent from the registry", id)));
        }
    }
    for id in registry.iter() {
        if !declared.contains(id) {
            out.push(mk(rule, id, format!("registry entry {} has no declaring source file", id)));
        }
        let md = doc_path_for(&docs_prefix, id);
        if !cwd.join(&md).exists() {
            out.push(mk(rule, id, format!("registry entry {} has no documentation file at {}", id, md)));
        }
    }
    Ok(out)
}

/// CEP:WHAT: Verifies that every documentation file carries the two tier headings.
/// CEP:WHY: Chapter 68 mandates dual-tier documentation for every registered function.
/// CEP:STATUS: complete
/// CEP:FAILURE: Bootstrap error when the registry is configured but missing.
/// CEP:ASSUMES: none
/// CEP:COST: cold; linear in documentation files.
/// CEP:EVIDENCE: CI registry gate; self-test scenario for CEP-LINT-DOC-TIERS.
pub fn check_doc_tiers(rule: &Rule, cwd: &Path) -> Result<Vec<Finding>, BootstrapError> {
    let registry = load_registry(&cwd.join(REGISTRY_PATH))?;
    let mut out = Vec::new();
    for id in registry.iter() {
        let md = doc_path_for(DOCS_PREFIX, id);
        let text = fs::read_to_string(cwd.join(&md)).unwrap_or_default();
        if !text.contains(TIER_ONE) {
            out.push(mk(rule, id, format!("documentation {} lacks the heading {}", md, TIER_ONE)));
        }
        if !text.contains(TIER_TWO) {
            out.push(mk(rule, id, format!("documentation {} lacks the heading {}", md, TIER_TWO)));
        }
    }
    Ok(out)
}

/// CEP:WHAT: Verifies that every documentation file carries the silent failure checklist.
/// CEP:WHY: Chapter 68 section four mandates inquisition reminders on every page.
/// CEP:STATUS: complete
/// CEP:FAILURE: Bootstrap error when the registry is configured but missing.
/// CEP:ASSUMES: none
/// CEP:COST: cold; linear in documentation files.
/// CEP:EVIDENCE: CI registry gate; self-test scenario for CEP-LINT-DOC-INQUISITION.
pub fn check_doc_inquisition(rule: &Rule, cwd: &Path) -> Result<Vec<Finding>, BootstrapError> {
    let registry = load_registry(&cwd.join(REGISTRY_PATH))?;
    let mut out = Vec::new();
    for id in registry.iter() {
        let md = doc_path_for(DOCS_PREFIX, id);
        let text = fs::read_to_string(cwd.join(&md)).unwrap_or_default();
        if !text.contains(INQUISITION) {
            out.push(mk(rule, id, format!("documentation {} lacks the section {}", md, INQUISITION)));
        }
    }
    Ok(out)
}

/// CEP:WHAT: Dispatches a rosetta_policy rule by its mode.
/// CEP:WHY: Mode selection keeps one check name per rule catalog entry.
/// CEP:STATUS: complete
/// CEP:FAILURE: Propagates bootstrap errors from registry loading.
/// CEP:ASSUMES: none
/// CEP:COST: cold; constant time.
/// CEP:EVIDENCE: CI registry gate.
pub fn check(rule: &Rule, files: &[SourceFile], cwd: &Path) -> Result<Vec<Finding>, BootstrapError> {
    match rule.mode.as_deref() {
        Some("doc_file") => check_doc_file(rule, files, cwd),
        Some("doc_tiers") => check_doc_tiers(rule, cwd),
        Some("doc_inquisition") => check_doc_inquisition(rule, cwd),
        _ => Ok(Vec::new()),
    }
}

/// CEP:WHAT: Derives the documentation path for an identifier.
/// CEP:WHY: Chapter 68 fixes the documentation topology.
/// CEP:STATUS: complete
/// CEP:FAILURE: none
/// CEP:ASSUMES: The domain is the prefix before the first hyphen.
/// CEP:COST: cold; constant time.
/// CEP:EVIDENCE: CI registry gate.
fn doc_path_for(prefix: &str, id: &str) -> String {
    let domain = id.split('-').next().unwrap_or(id);
    format!("{}/{}/{}.md", prefix, domain, id)
}

/// CEP:WHAT: Builds a rosetta finding anchored at the documentation path.
/// CEP:WHY: Rosetta findings point at the missing documentation artifact.
/// CEP:STATUS: complete
/// CEP:FAILURE: none
/// CEP:ASSUMES: none
/// CEP:COST: cold; constant time.
/// CEP:EVIDENCE: CI registry gate.
fn mk(rule: &Rule, id: &str, detail: String) -> Finding {
    Finding {
        severity: rule.severity,
        rule: rule.id.clone(),
        file: doc_path_for(DOCS_PREFIX, id),
        line: 0,
        message: crate::model::substitute(&rule.message, &[("detail", &detail)]),
    }
}
