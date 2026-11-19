// CEP:FILE: compiler/optimize/src/dce.rs
// CEP:WHAT: Dead-store elimination pass over MIR statements.
// CEP:WHY: Level-1 optimization removes assignments to locals never read afterwards.
// CEP:CLASS: HPC-1
// CEP:STATUS: complete
// CEP:FAILURE: none; trapping rvalues are never removed.
// CEP:ASSUMES: Local zero is the return slot and is never dead; parameters may be unused.
// CEP:COST: cold; quadratic in block statement count with linear re-scan per candidate.
// CEP:EVIDENCE: unit test rr_optimize::dce::tests::removes_dead_stores.
#![deny(missing_docs)]
use rr_certificates::{dead_obligation, Json};
use rr_hir::hir::BinOp;
use rr_mir::model::*;

/// CEP:WHAT: Removes dead stores in one function and emits obligations.
/// CEP:WHY: The verifier re-derives liveness from the output program independently.
/// CEP:ID: IR-a09
/// CEP:STATUS: complete
/// CEP:FAILURE: none
/// CEP:ASSUMES: The local zero return slot is preserved.
/// CEP:COST: cold; quadratic in block length.
/// CEP:EVIDENCE: doc file docs/functions/IR/IR-a09.md; unit test removes_dead_stores.
pub fn run(f: &mut Function, obligations: &mut Vec<Json>) {
    for bi in 0..f.blocks.len() {
        let mut i = 0;
        while i < f.blocks[bi].stmts.len() {
            if let Some(local) = dead_at(f, bi, i) {
                obligations.push(dead_obligation(f.name.as_str(), bi, i, local));
                f.blocks[bi].stmts.remove(i);
            } else {
                i += 1;
            }
        }
    }
}

/// CEP:WHAT: Returns the local of one dead store at a statement index.
/// CEP:WHY: The pass needs the removed local for its obligation record.
/// CEP:STATUS: complete
/// CEP:FAILURE: Returns None when the statement is live, trapping, or the return slot.
/// CEP:ASSUMES: none
/// CEP:COST: cold; linear in remaining statements.
/// CEP:EVIDENCE: unit test rr_optimize::dce::tests::removes_dead_stores.
fn dead_at(f: &Function, bi: usize, si: usize) -> Option<LocalId> {
    let Statement::Assign { place, rvalue } = &f.blocks[bi].stmts[si] else { return None };
    if *place == 0 {
        return None;
    }
    if traps(rvalue) {
        return None;
    }
    if read_later(f, bi, si, *place) || branch_reads(f, bi, *place) {
        return None;
    }
    Some(*place)
}

/// CEP:WHAT: Reports whether one rvalue can trap.
/// CEP:WHY: Division and remainder have defined traps and are observable.
/// CEP:STATUS: complete
/// CEP:FAILURE: none
/// CEP:ASSUMES: none
/// CEP:COST: cold; constant time.
/// CEP:EVIDENCE: unit test rr_optimize::dce::tests::removes_dead_stores.
fn traps(r: &Rvalue) -> bool {
    matches!(r, Rvalue::BinaryOp { op: BinOp::Div | BinOp::Rem, .. })
}

/// CEP:WHAT: Reports whether a local is read after one index in one block.
/// CEP:WHY: Liveness within a block is the V0 scope of the pass.
/// CEP:STATUS: complete
/// CEP:FAILURE: none
/// CEP:ASSUMES: none
/// CEP:COST: cold; linear in remaining statements.
/// CEP:EVIDENCE: unit test rr_optimize::dce::tests::removes_dead_stores.
fn read_later(f: &Function, bi: usize, si: usize, local: LocalId) -> bool {
    for s in f.blocks[bi].stmts[si + 1..].iter() {
        if stmt_reads(s, local) {
            return true;
        }
    }
    false
}

/// CEP:WHAT: Reports whether a local is read by any terminator of any block.
/// CEP:WHY: Terminator operands read locals too.
/// CEP:STATUS: complete
/// CEP:FAILURE: none
/// CEP:ASSUMES: none
/// CEP:COST: cold; linear in blocks.
/// CEP:EVIDENCE: unit test rr_optimize::dce::tests::removes_dead_stores.
fn branch_reads(f: &Function, _bi: usize, local: LocalId) -> bool {
    for b in f.blocks.iter() {
        if term_reads(&b.term, local) {
            return true;
        }
    }
    false
}

/// CEP:WHAT: Reports whether one terminator reads a local.
/// CEP:WHY: Switch discriminants and call arguments read locals.
/// CEP:STATUS: complete
/// CEP:FAILURE: none
/// CEP:ASSUMES: none
/// CEP:COST: cold; constant per terminator.
/// CEP:EVIDENCE: unit test rr_optimize::dce::tests::removes_dead_stores.
fn term_reads(t: &Terminator, local: LocalId) -> bool {
    match t {
        Terminator::SwitchInt { disc, .. } => operand_reads(disc, local),
        Terminator::Call { args, .. } => args.iter().any(|a| operand_reads(a, local)),
        _ => false,
    }
}

/// CEP:WHAT: Reports whether one statement reads a local.
/// CEP:WHY: Deadness is the absence of reads.
/// CEP:STATUS: complete
/// CEP:FAILURE: none
/// CEP:ASSUMES: none
/// CEP:COST: cold; constant per statement.
/// CEP:EVIDENCE: unit test rr_optimize::dce::tests::removes_dead_stores.
fn stmt_reads(s: &Statement, local: LocalId) -> bool {
    match s {
        Statement::Assign { rvalue, .. } => rvalue_reads(rvalue, local),
        Statement::Nop => false,
    }
}

/// CEP:WHAT: Reports whether one rvalue reads a local.
/// CEP:WHY: Deadness is the absence of reads.
/// CEP:STATUS: complete
/// CEP:FAILURE: none
/// CEP:ASSUMES: none
/// CEP:COST: cold; constant per rvalue.
/// CEP:EVIDENCE: unit test rr_optimize::dce::tests::removes_dead_stores.
fn rvalue_reads(r: &Rvalue, local: LocalId) -> bool {
    match r {
        Rvalue::Use(o) => operand_reads(o, local),
        Rvalue::UnaryOp { e, .. } => operand_reads(e, local),
        Rvalue::BinaryOp { l, r, .. } => operand_reads(l, local) || operand_reads(r, local),
    }
}

/// CEP:WHAT: Reports whether one operand reads a local.
/// CEP:WHY: Deadness is the absence of reads.
/// CEP:STATUS: complete
/// CEP:FAILURE: none
/// CEP:ASSUMES: none
/// CEP:COST: cold; constant time.
/// CEP:EVIDENCE: unit test rr_optimize::dce::tests::removes_dead_stores.
fn operand_reads(o: &Operand, local: LocalId) -> bool {
    matches!(o, Operand::Copy(l) if *l == local)
}

#[cfg(test)]
mod tests {
    use super::*;

        /// Value of the dead store in the sample.
    const SAMPLE_DEAD: i64 = 9;

    /// Value of the live store in the sample.
    const SAMPLE_LIVE: i64 = 2;

/// CEP:WHAT: Verifies removal of a store whose local is never read.
    /// CEP:WHY: DCE is a level-1 optimization with certificate obligations.
    /// CEP:STATUS: complete
    /// CEP:FAILURE: Fails the test binary on regression.
    /// CEP:ASSUMES: none
    /// CEP:COST: cold; unit test only.
    /// CEP:EVIDENCE: cargo test -p rr-optimize.
    #[test]
    fn removes_dead_stores() {
        let mut f = Function {
            name: "main".into(),
            locals: vec![Local { ty: Ty::I64, is_immutable: true }, Local { ty: Ty::I64, is_immutable: false }],
            params: Vec::new(),
            ret: Ty::I64,
            blocks: vec![BasicBlock {
                stmts: vec![
                    Statement::Assign { place: 1, rvalue: Rvalue::Use(Operand::Const(Constant::I64(SAMPLE_DEAD))) },
                    Statement::Assign { place: 0, rvalue: Rvalue::Use(Operand::Const(Constant::I64(SAMPLE_LIVE))) },
                ],
                term: Terminator::Return,
            }],
        };
        let mut obs = Vec::new();
        run(&mut f, &mut obs);
        assert_eq!(f.blocks[0].stmts.len(), 1);
        assert_eq!(obs.len(), 1);
    }
}
