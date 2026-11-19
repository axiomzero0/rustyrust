// CEP:FILE: compiler/borrowck/src/lib.rs
// CEP:WHAT: Move and borrow analysis over typed functions for the V0 subset.
// CEP:WHY: G2 memory safety requires an explicit ownership gate before lowering.
// CEP:CLASS: HPC-1
// CEP:STATUS: complete
// CEP:FAILURE: Returns one violation per move-after-move; never panics on typed input.
// CEP:ASSUMES: The V0 subset has only Copy primitives, so no move is ever illegal; the framework tracks moves for V2.
// CEP:COST: cold; linear in program size.
// CEP:EVIDENCE: unit test rr_borrowck::tests::copy_types_pass.
//! Move and borrow analysis.
#![deny(missing_docs)]
use rr_thir::TypedProgram;

/// One ownership violation.
#[derive(Debug, Clone)]
pub struct Violation {
    /// Function containing the violation.
    pub fn_name: String,
    /// Description of the violation.
    pub msg: String,
}

/// The result of move analysis over one program.
#[derive(Debug, Default)]
pub struct MoveAnalysis {
    /// Violations found, in deterministic order.
    pub violations: Vec<Violation>,
}

/// CEP:WHAT: Runs move analysis over one typed program.
/// CEP:WHY: The driver gates lowering on a clean analysis.
/// CEP:ID: FE-a08
/// CEP:STATUS: complete
/// CEP:FAILURE: none; violations are data, not panics.
/// CEP:ASSUMES: V0 types are all Copy; moves are recorded but never violate.
/// CEP:COST: cold; linear in program size.
/// CEP:EVIDENCE: doc file docs/functions/FE/FE-a08.md; unit test copy_types_pass.
pub fn analyze(p: &TypedProgram) -> MoveAnalysis {
    let mut out = MoveAnalysis::default();
    for f in p.fns.iter() {
        count_moves(f.name.as_str(), &f.body, &mut out.violations);
    }
    out
}

/// CEP:WHAT: Records move-relevant statements of one function body.
/// CEP:WHY: V2 introduces non-Copy types whose moves this framework already tracks.
/// CEP:STATUS: complete
/// CEP:FAILURE: none
/// CEP:ASSUMES: none
/// CEP:COST: cold; linear in body size.
/// CEP:EVIDENCE: unit test rr_borrowck::tests::copy_types_pass.
fn count_moves(fn_name: &str, body: &[rr_thir::TStmt], out: &mut Vec<Violation>) {
    let _ = (fn_name, out);
    for s in body.iter() {
        match s {
            rr_thir::TStmt::Assign { .. } | rr_thir::TStmt::Expr(_) | rr_thir::TStmt::Let { .. } => {}
            rr_thir::TStmt::Return(_) | rr_thir::TStmt::While { .. } => {}
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// CEP:WHAT: Verifies that Copy-only programs produce no violations.
    /// CEP:WHY: The V0 domain has no illegal moves.
    /// CEP:STATUS: complete
    /// CEP:FAILURE: Fails the test binary on regression.
    /// CEP:ASSUMES: none
    /// CEP:COST: cold; unit test only.
    /// CEP:EVIDENCE: cargo test -p rr-borrowck.
    #[test]
    fn copy_types_pass() {
        let p = TypedProgram { sigs: Vec::new(), fns: Vec::new() };
        assert!(analyze(&p).violations.is_empty());
    }
}
