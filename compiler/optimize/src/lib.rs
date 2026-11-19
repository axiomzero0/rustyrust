// CEP:FILE: compiler/optimize/src/lib.rs
// CEP:WHAT: Library root for certificate-producing MIR passes: constant folding and dead-store elimination.
// CEP:WHY: G4 requires every optimization to preserve semantics with machine-checkable evidence.
// CEP:CLASS: HPC-1
// CEP:STATUS: complete
// CEP:FAILURE: none; passes are total over verified MIR.
// CEP:ASSUMES: Input MIR passes the structural verifier; pass order is fixed in the pipeline.
// CEP:COST: cold; linear in program size per pass.
// CEP:EVIDENCE: cargo test -p rr-optimize.
//! Certificate-producing MIR optimization passes.

/// Constant folding pass over arithmetic and comparisons.
pub mod const_fold;
/// Dead-store elimination pass.
pub mod dce;
/// Ordered pass pipeline with certificate assembly.
pub mod pipeline;
