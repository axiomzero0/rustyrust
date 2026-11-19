// CEP:FILE: ir/hir/src/lib.rs
// CEP:WHAT: Library root for the high-level IR.
// CEP:WHY: Frontend and type checking consume one shared syntax definition.
// CEP:CLASS: HPC-1
// CEP:STATUS: complete
// CEP:FAILURE: none
// CEP:ASSUMES: none
// CEP:COST: cold; type definitions only.
// CEP:EVIDENCE: cargo build -p rr-hir.
//! High-level IR: spans and V0 dialect syntax.
#![deny(missing_docs)]

/// Byte-span tracking for diagnostics.
pub mod span;
/// Source syntax of the V0 dialect.
pub mod hir;
