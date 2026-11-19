// CEP:FILE: compiler/optimize/src/const_fold.rs
// CEP:WHAT: Constant folding pass for MIR rvalues with wrap-i64 semantics.
// CEP:WHY: Level-1 optimization eliminates compile-time computable work; semantics are the dialect rules.
// CEP:CLASS: HPC-1
// CEP:STATUS: complete
// CEP:FAILURE: none; trapping operations are never folded.
// CEP:ASSUMES: Arithmetic wraps with two's complement semantics; division and remainder trap on zero or MIN over minus one.
// CEP:COST: cold; linear in program size.
// CEP:EVIDENCE: unit test rr_optimize::const_fold::tests::folds_with_wrap.
#![deny(missing_docs)]
use rr_certificates::{fold_obligation, Json};
use rr_hir::hir::{BinOp, UnOp};
use rr_mir::model::*;

/// CEP:WHAT: Folds constant rvalues in one function and emits obligations.
/// CEP:WHY: Folding at MIR level applies across all block contexts uniformly.
/// CEP:ID: IR-a07
/// CEP:STATUS: complete
/// CEP:FAILURE: none; trapping operations are skipped, never folded.
/// CEP:ASSUMES: none
/// CEP:COST: cold; linear in statements.
/// CEP:EVIDENCE: doc file docs/functions/IR/IR-a07.md; unit test folds_with_wrap.
pub fn run(f: &mut Function, obligations: &mut Vec<Json>) {
    for (bi, b) in f.blocks.iter_mut().enumerate() {
        for (si, s) in b.stmts.iter_mut().enumerate() {
            if let Statement::Assign { place, rvalue } = s {
                fold_assign(f.name.as_str(), bi, si, *place, rvalue, obligations);
            }
        }
    }
}

/// CEP:WHAT: Folds one assign statement when both operands are constants.
/// CEP:WHY: Statement-level folding keeps obligation addresses precise.
/// CEP:STATUS: complete
/// CEP:FAILURE: none
/// CEP:ASSUMES: none
/// CEP:COST: cold; constant per statement.
/// CEP:EVIDENCE: unit test rr_optimize::const_fold::tests::folds_with_wrap.
fn fold_assign(name: &str, bi: usize, si: usize, place: LocalId, rv: &mut Rvalue, obligations: &mut Vec<Json>) {
    let candidate = match rv {
        Rvalue::BinaryOp { op, l: Operand::Const(a), r: Operand::Const(b) } => Some((*op, *a, *b)),
        Rvalue::UnaryOp { op: UnOp::Neg, e: Operand::Const(Constant::I64(v)) } => {
            Some((BinOp::Sub, Constant::I64(0), Constant::I64(*v)))
        }
        _ => None,
    };
    if let Some((op, l, r)) = candidate {
        if let Some(res) = eval(&op, l, r) {
            let lv = int_of(l);
            let rvv = int_of(r);
            obligations.push(fold_obligation(name, bi, place, op.symbol(), lv, rvv, res));
            *rv = Rvalue::Use(Operand::Const(const_of(&op, res)));
        }
        let _ = si;
    }
}

/// CEP:WHAT: Returns the integer payload of one constant when i64.
/// CEP:WHY: Obligations record operand values for independent re-derivation.
/// CEP:STATUS: complete
/// CEP:FAILURE: Returns zero for non-integer constants; folding skipped them anyway.
/// CEP:ASSUMES: none
/// CEP:COST: cold; constant time.
/// CEP:EVIDENCE: unit test rr_optimize::const_fold::tests::folds_with_wrap.
fn int_of(c: Constant) -> i64 {
    match c {
        Constant::I64(v) => v,
        _ => 0,
    }
}

/// CEP:WHAT: Builds the folded constant by operator result class.
/// CEP:WHY: Comparisons fold to bool; arithmetic folds to i64.
/// CEP:STATUS: complete
/// CEP:FAILURE: none
/// CEP:ASSUMES: none
/// CEP:COST: cold; constant time.
/// CEP:EVIDENCE: unit test rr_optimize::const_fold::tests::folds_with_wrap.
fn const_of(op: &BinOp, v: i64) -> Constant {
    if op.is_cmp() {
        Constant::Bool(v != 0)
    } else {
        Constant::I64(v)
    }
}

/// CEP:WHAT: Evaluates one binary operation under wrap-i64 semantics.
/// CEP:WHY: The folding semantics and the verifier semantics must be the same table.
/// CEP:ID: IR-a08
/// CEP:STATUS: complete
/// CEP:FAILURE: Returns None for trapping operations; the statement stays unfolded.
/// CEP:ASSUMES: Comparison results encode as zero or one.
/// CEP:COST: cold; constant time.
/// CEP:EVIDENCE: doc file docs/functions/IR/IR-a08.md; unit test folds_with_wrap.
pub fn eval(op: &BinOp, l: Constant, r: Constant) -> Option<i64> {
    let (a, b) = match (l, r) {
        (Constant::I64(a), Constant::I64(b)) => (a, b),
        _ => return None,
    };
    match op {
        BinOp::Add => Some(a.wrapping_add(b)),
        BinOp::Sub => Some(a.wrapping_sub(b)),
        BinOp::Mul => Some(a.wrapping_mul(b)),
        BinOp::Div | BinOp::Rem => div_eval(*op == BinOp::Div, a, b),
        BinOp::Eq => Some((a == b) as i64),
        BinOp::Ne => Some((a != b) as i64),
        BinOp::Lt => Some((a < b) as i64),
        BinOp::Gt => Some((a > b) as i64),
        BinOp::Le => Some((a <= b) as i64),
        BinOp::Ge => Some((a >= b) as i64),
        BinOp::And | BinOp::Or => None,
    }
}

/// CEP:WHAT: Evaluates division and remainder with trap exclusions.
/// CEP:WHY: Zero divisors and MIN over minus one trap in the dialect; folding must not remove traps.
/// CEP:STATUS: complete
/// CEP:FAILURE: Returns None for trapping inputs.
/// CEP:ASSUMES: none
/// CEP:COST: cold; constant time.
/// CEP:EVIDENCE: unit test rr_optimize::const_fold::tests::never_folds_traps.
fn div_eval(is_div: bool, a: i64, b: i64) -> Option<i64> {
    if b == 0 || (a == i64::MIN && b == -1) {
        return None;
    }
    Some(if is_div { a.wrapping_div(b) } else { a.wrapping_rem(b) })
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Maximum i64 value used by the wrap test.
    const MAX: i64 = i64::MAX;

    /// CEP:WHAT: Verifies folding with two's complement wrap.
    /// CEP:WHY: The dialect defines wrapping semantics in every profile.
    /// CEP:STATUS: complete
    /// CEP:FAILURE: Fails the test binary on regression.
    /// CEP:ASSUMES: none
    /// CEP:COST: cold; unit test only.
    /// CEP:EVIDENCE: cargo test -p rr-optimize.
    #[test]
    fn folds_with_wrap() {
        assert_eq!(eval(&BinOp::Add, Constant::I64(MAX), Constant::I64(1)), Some(i64::MIN));
        assert_eq!(eval(&BinOp::Eq, Constant::I64(2), Constant::I64(2)), Some(1));
    }

    /// CEP:WHAT: Verifies that trapping operations never fold.
    /// CEP:WHY: Folding a trap would remove observable behavior.
    /// CEP:STATUS: complete
    /// CEP:FAILURE: Fails the test binary on regression.
    /// CEP:ASSUMES: none
    /// CEP:COST: cold; unit test only.
    /// CEP:EVIDENCE: cargo test -p rr-optimize.
    #[test]
    fn never_folds_traps() {
        assert_eq!(eval(&BinOp::Div, Constant::I64(1), Constant::I64(0)), None);
        assert_eq!(eval(&BinOp::Div, Constant::I64(i64::MIN), Constant::I64(-1)), None);
    }
}
