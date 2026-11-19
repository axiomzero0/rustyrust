// CEP:FILE: tools/cep_lint/src/lib.rs
// CEP:WHAT: Library root for the cep_lint engine modules.
// CEP:WHY: The binary and the test suite consume the same engine so behavior cannot diverge.
// CEP:CLASS: CEP-2
// CEP:STATUS: complete
// CEP:FAILURE: none
// CEP:ASSUMES: none
// CEP:COST: cold; module declarations only.
// CEP:EVIDENCE: cargo build -p cep_lint.
//! Mechanical enforcement tool for the CEP&CC standard, chapter 50.
//!
//! The engine is data-driven: every rule, threshold, pattern, and message lives
//! in `.cep/cep_lint.json`; the modules below provide mechanism only.
#![deny(missing_docs)]
// CEP:WHAT: Library root for the cep_lint engine modules.
// CEP:WHY: The binary and the test suite consume the same engine so behavior cannot diverge.
// CEP:CLASS: CEP-2
// CEP:STATUS: complete
// CEP:FAILURE: none
// CEP:ASSUMES: none
// CEP:COST: cold; module declarations only.
// CEP:EVIDENCE: cargo build -p cep_lint.

/// Comment block schema checks for headers, statuses, and stub requirements.
pub mod blockschema;
/// Command line interface parsing and usage text.
pub mod cli;
/// Configuration loading and schema validation.
pub mod config;
/// Documentation synchronization between standard and configuration.
pub mod docsync;
/// Engine orchestration over discovered files and rules.
pub mod engine;
/// Function policy checks sharing one extractor.
pub mod fnpolicy;
/// Minimal JSON parser with positioned errors.
pub mod json;
/// Self-test manifest runner.
pub mod manifest;
/// Core data model for rules, findings, and configuration.
pub mod model;
/// Hunt and HUNTER receipt verification.
pub mod receipts;
/// Lexical rule primitives over comments, strings, and code.
pub mod regexrules;
/// Rosetta protocol checks for function documentation.
pub mod rosetta;
/// Source file scanner producing the shared scan model.
pub mod scan;
/// Secret probing over comment text and string literals.
pub mod secrets;
/// File topology policy checks.
pub mod topology;
/// Unsafe usage documentation check.
pub mod unsafedoc;
