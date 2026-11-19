// CEP:FILE: tools/cep_lint/src/manifest.rs
// CEP:WHAT: Self-test manifest runner for exact per-rule scenario verification.
// CEP:WHY: Section 50.6 item three requires every scenario to pass with exact expected finding counts.
// CEP:CLASS: CEP-2
// CEP:STATUS: complete
// CEP:FAILURE: Exits with failure when any scenario deviates from its expected count.
// CEP:ASSUMES: Scenario directories are isolated under one temp root; cross-file rules run only when declared.
// CEP:COST: cold; linear in scenarios times rules.
// CEP:EVIDENCE: cep_lint --self-test in CI.

use crate::config;
use crate::engine::{self, Flags};
use crate::model::{BootstrapError, Config};
use std::fs;
use std::path::Path;

/// Bootstrap code for file input failures.
const BOOT_IO: u8 = 1;

/// Bootstrap code for JSON parse failures.
const BOOT_JSON: u8 = 2;

/// Bootstrap code for missing configuration keys.
const BOOT_MISSING_KEY: u8 = 4;

/// One scenario of the self-test manifest.
struct Scenario {
    rule: String,
    name: String,
    expected: i64,
    files: Vec<(String, String)>,
    cross_file: bool,
    flags: Flags,
}

/// CEP:WHAT: Loads the manifest and runs every scenario with exact count verification.
/// CEP:WHY: The self-verification gate must prove the engine matches the rule catalog.
/// CEP:ID: CS-a05
/// CEP:STATUS: complete
/// CEP:FAILURE: Returns a bootstrap error for manifest loading; returns failures as a string.
/// CEP:ASSUMES: The manifest schema is documented in tools/cep_lint/tests/manifest.json.
/// CEP:COST: cold; linear in scenarios times rules.
/// CEP:EVIDENCE: doc file docs/functions/CS/CS-a05.md; CI self-test job.
pub fn run(config_path: &Path) -> Result<String, BootstrapError> {
    let cfg = config::load(config_path)?;
    let scenarios = load_manifest(config_path)?;
    let root = std::env::temp_dir().join("cep_lint_selftest");
    let _ = fs::remove_dir_all(&root);
    fs::create_dir_all(&root).map_err(|e| BootstrapError::new(BOOT_IO, format!("temp root: {}", e)))?;
    let failures = run_all(&cfg, &scenarios, &root);
    summarize(&scenarios, &failures)
}

/// CEP:WHAT: Loads and parses the manifest file.
/// CEP:WHY: Manifest loading failures are bootstrap errors one and two.
/// CEP:STATUS: complete
/// CEP:FAILURE: Bootstrap error for unreadable or malformed manifests.
/// CEP:ASSUMES: The manifest lives beside the test fixtures of the tool.
/// CEP:COST: cold; linear in manifest size.
/// CEP:EVIDENCE: CI self-test job.
fn load_manifest(config_path: &Path) -> Result<Vec<Scenario>, BootstrapError> {
    let manifest_path = config_path.parent().unwrap_or(Path::new(".")).join("../tools/cep_lint/tests/manifest.json");
    let manifest_path = fs::canonicalize(&manifest_path).unwrap_or(manifest_path);
    let text = fs::read_to_string(&manifest_path)
        .map_err(|e| BootstrapError::new(BOOT_IO, format!("manifest {} unreadable: {}", manifest_path.display(), e)))?;
    let value = crate::json::parse(&text)
        .map_err(|e| BootstrapError::new(BOOT_JSON, format!("manifest {}: {}", manifest_path.display(), e.detail)))?;
    parse_scenarios(&value)
}

/// CEP:WHAT: Runs every scenario and returns failure descriptions.
/// CEP:WHY: Count verification is exact per scenario per section 50.6.
/// CEP:STATUS: complete
/// CEP:FAILURE: none; deviations are returned as textual failures.
/// CEP:ASSUMES: none
/// CEP:COST: cold; linear in scenarios.
/// CEP:EVIDENCE: CI self-test job.
fn run_all(cfg: &Config, scenarios: &[Scenario], root: &Path) -> Vec<String> {
    let mut failures: Vec<String> = Vec::new();
    for s in scenarios.iter() {
        match run_scenario(cfg, s, root) {
            Ok(count) => {
                if count != s.expected {
                    failures.push(format!("{} {} expected {} got {}", s.rule, s.name, s.expected, count));
                }
            }
            Err(e) => failures.push(format!("{} {} bootstrap {}", s.rule, s.name, e.format())),
        }
    }
    failures
}

/// CEP:WHAT: Produces the pass or failure summary text.
/// CEP:WHY: The CI gate reads the exit path, and humans read the summary.
/// CEP:STATUS: complete
/// CEP:FAILURE: none
/// CEP:ASSUMES: none
/// CEP:COST: cold; linear in failures.
/// CEP:EVIDENCE: CI self-test job.
fn summarize(scenarios: &[Scenario], failures: &[String]) -> Result<String, BootstrapError> {
    if failures.is_empty() {
        Ok(format!("self-test: {} scenarios passed\n", scenarios.len()))
    } else {
        Ok(format!("self-test: {} of {} scenarios failed\n{}", failures.len(), scenarios.len(), failures.join("\n")))
    }
}

/// CEP:WHAT: Parses scenario objects from the manifest value.
/// CEP:WHY: The manifest schema declares rule, expectation, and scenario files.
/// CEP:STATUS: complete
/// CEP:FAILURE: Bootstrap error four when the scenarios array is missing.
/// CEP:ASSUMES: none
/// CEP:COST: cold; linear in scenarios.
/// CEP:EVIDENCE: CI self-test job.
fn parse_scenarios(value: &crate::json::Value) -> Result<Vec<Scenario>, BootstrapError> {
    let arr = value.get("scenarios").and_then(|v| v.as_arr())
        .ok_or_else(|| BootstrapError::new(BOOT_MISSING_KEY, "manifest.scenarios is missing"))?;
    let mut out = Vec::new();
    for item in arr {
        out.push(parse_scenario(item)?);
    }
    Ok(out)
}

/// CEP:WHAT: Parses one scenario object with its files.
/// CEP:WHY: Splitting keeps the parser within statement limits.
/// CEP:STATUS: complete
/// CEP:FAILURE: none; missing fields take deterministic defaults.
/// CEP:ASSUMES: none
/// CEP:COST: cold; linear in scenario files.
/// CEP:EVIDENCE: CI self-test job.
fn parse_scenario(item: &crate::json::Value) -> Result<Scenario, BootstrapError> {
    let rule = item.get("rule").and_then(|v| v.as_str()).unwrap_or("").to_string();
    let name = item.get("name").and_then(|v| v.as_str()).unwrap_or("").to_string();
    let expected = item.get("expected").and_then(|v| v.as_i64()).unwrap_or(0);
    let cross_file = item.get("cross_file").and_then(|v| v.as_bool()).unwrap_or(false);
    let flags = Flags {
        verify_hunt: item.get("flags").and_then(|f| f.get("verify_hunt")).and_then(|v| v.as_bool()).unwrap_or(false),
        verify_hunter: item.get("flags").and_then(|f| f.get("verify_hunter")).and_then(|v| v.as_bool()).unwrap_or(false),
    };
    let files = parse_files(item);
    Ok(Scenario { rule, name, expected, files, cross_file, flags })
}

/// CEP:WHAT: Parses the file list of one scenario.
/// CEP:WHY: Scenarios declare one or more files with inline content.
/// CEP:STATUS: complete
/// CEP:FAILURE: none
/// CEP:ASSUMES: File entries use the path and content keys.
/// CEP:COST: cold; linear in files.
/// CEP:EVIDENCE: CI self-test job.
fn parse_files(item: &crate::json::Value) -> Vec<(String, String)> {
    let mut files = Vec::new();
    if let Some(farr) = item.get("files").and_then(|v| v.as_arr()) {
        for f in farr {
            let p = f.get("path").and_then(|v| v.as_str()).unwrap_or("input.rs").to_string();
            let c = f.get("content").and_then(|v| v.as_str()).unwrap_or("").to_string();
            files.push((p, c));
        }
    }
    files
}

/// CEP:WHAT: Runs one scenario in an isolated directory and returns the rule finding count.
/// CEP:WHY: Isolation keeps scenarios independent of repository state.
/// CEP:STATUS: complete
/// CEP:FAILURE: Bootstrap error for scenario directory or file creation failures.
/// CEP:ASSUMES: The engine honors the rule filter and the cross-file skip flag.
/// CEP:COST: cold; one engine run per scenario.
/// CEP:EVIDENCE: CI self-test job.
fn run_scenario(cfg: &Config, s: &Scenario, root: &Path) -> Result<i64, BootstrapError> {
    let dir = root.join(format!("{}_{}", s.rule, s.name));
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).map_err(|e| BootstrapError::new(BOOT_IO, format!("scenario dir: {}", e)))?;
    write_files(&dir, &s.files)?;
    let filter = vec![s.rule.clone()];
    let findings = engine::run(cfg, &[".".to_string()], &dir, Some(&filter), !s.cross_file, s.flags)?;
    Ok(findings.iter().filter(|f| f.rule == s.rule).count() as i64)
}

/// CEP:WHAT: Writes the scenario files under the scenario directory.
/// CEP:WHY: Scenarios materialize their fixtures before the engine runs.
/// CEP:STATUS: complete
/// CEP:FAILURE: Bootstrap error for write failures.
/// CEP:ASSUMES: Paths stay inside the scenario directory.
/// CEP:COST: cold; linear in files.
/// CEP:EVIDENCE: CI self-test job.
fn write_files(dir: &Path, files: &[(String, String)]) -> Result<(), BootstrapError> {
    for (p, c) in files.iter() {
        let full = dir.join(p);
        if let Some(parent) = full.parent() {
            fs::create_dir_all(parent).map_err(|e| BootstrapError::new(BOOT_IO, format!("scenario parent: {}", e)))?;
        }
        fs::write(&full, c).map_err(|e| BootstrapError::new(BOOT_IO, format!("scenario file: {}", e)))?;
    }
    Ok(())
}
