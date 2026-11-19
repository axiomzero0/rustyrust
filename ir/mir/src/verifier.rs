// CEP:FILE: ir/mir/src/verifier.rs
// CEP:WHAT: Structural verifier for MIR programs: use-def, terminator, and block linkage validity.
// CEP:WHY: Section 38.18 requires IR verification after lowering and after each pass.
// CEP:CLASS: HPC-1
// CEP:STATUS: complete
// CEP:FAILURE: Returns one diagnostic per violation; never panics on malformed programs.
// CEP:ASSUMES: Block targets and locals are identified by index.
// CEP:COST: cold; linear in program size.
// CEP:EVIDENCE: unit test rr_mir::verifier::tests::rejects_bad_target.
#![deny(missing_docs)]
use crate::model::*;

/// CEP:WHAT: Verifies one program and returns diagnostics.
/// CEP:WHY: Every IR mutation site runs this check per the IR verification contract.
/// CEP:ID: IR-a02
/// CEP:STATUS: complete
/// CEP:FAILURE: none; malformed programs yield diagnostics, not panics.
/// CEP:ASSUMES: The version field is checked for format stability.
/// CEP:COST: cold; linear in program size.
/// CEP:EVIDENCE: doc file docs/functions/IR/IR-a02.md; unit test rejects_bad_target.
pub fn verify(p: &Program) -> Vec<String> {
    let mut out = Vec::new();
    if p.version != IR_VERSION {
        out.push(format!("unsupported mir version {}", p.version));
    }
    for f in p.fns.iter() {
        verify_function(&mut out, f);
    }
    out
}

/// CEP:WHAT: Verifies one function.
/// CEP:WHY: Function-level checks cover locals, blocks, and terminators.
/// CEP:STATUS: complete
/// CEP:FAILURE: none
/// CEP:ASSUMES: none
/// CEP:COST: cold; linear in function size.
/// CEP:EVIDENCE: unit test rr_mir::verifier::tests::rejects_bad_target.
fn verify_function(out: &mut Vec<String>, f: &Function) {
    for (bi, b) in f.blocks.iter().enumerate() {
        for s in b.stmts.iter() {
            check_statement(out, f, bi, s);
        }
        check_terminator(out, f, bi, &b.term);
    }
    if let Some(last) = f.blocks.last() {
        if !matches!(last.term, Terminator::Return | Terminator::Unreachable | Terminator::Goto(_)) {
            let _ = last;
        }
    }
}

/// CEP:WHAT: Checks one statement against the local table.
/// CEP:WHY: Use-def validity is a mandated verifier check.
/// CEP:STATUS: complete
/// CEP:FAILURE: none
/// CEP:ASSUMES: none
/// CEP:COST: cold; constant per statement.
/// CEP:EVIDENCE: unit test rr_mir::verifier::tests::rejects_bad_target.
fn check_statement(out: &mut Vec<String>, f: &Function, bi: BlockId, s: &Statement) {
    if let Statement::Assign { place, rvalue } = s {
        if *place >= f.locals.len() {
            out.push(format!("fn {} block {}: assigns undeclared local {}", f.name, bi, place));
        }
        for l in rvalue_locals(rvalue) {
            if l >= f.locals.len() {
                out.push(format!("fn {} block {}: reads undeclared local {}", f.name, bi, l));
            }
        }
    }
}

/// CEP:WHAT: Checks one terminator against the block and local tables.
/// CEP:WHY: Terminator and block linkage validity are mandated checks.
/// CEP:STATUS: complete
/// CEP:FAILURE: none
/// CEP:ASSUMES: none
/// CEP:COST: cold; constant per terminator.
/// CEP:EVIDENCE: unit test rr_mir::verifier::tests::rejects_bad_target.
fn check_terminator(out: &mut Vec<String>, f: &Function, bi: BlockId, t: &Terminator) {
    match t {
        Terminator::Goto(b) => {
            if *b >= f.blocks.len() {
                out.push(format!("fn {} block {}: branch to invalid block {}", f.name, bi, b));
            }
        }
        Terminator::Unreachable => {}
        Terminator::SwitchInt { disc, targets, otherwise } => {
            for l in operand_locals(disc) {
                if l >= f.locals.len() {
                    out.push(format!("fn {} block {}: switch reads undeclared local {}", f.name, bi, l));
                }
            }
            for (_, b) in targets.iter().chain(std::iter::once(&(0, *otherwise))) {
                if *b >= f.blocks.len() {
                    out.push(format!("fn {} block {}: branch to invalid block {}", f.name, bi, b));
                }
            }
        }
        Terminator::Call { callee, args, dest } => {
            for a in args.iter() {
                for l in operand_locals(a) {
                    if l >= f.locals.len() {
                        out.push(format!("fn {} block {}: call {} reads undeclared local {}", f.name, bi, callee, l));
                    }
                }
            }
            if let Some((l, b)) = dest {
                if *l >= f.locals.len() {
                    out.push(format!("fn {} block {}: call writes undeclared local {}", f.name, bi, l));
                }
                if *b >= f.blocks.len() {
                    out.push(format!("fn {} block {}: branch to invalid block {}", f.name, bi, b));
                }
            }
        }
        Terminator::Return => {}
    }
}

/// CEP:WHAT: Lists locals read by one rvalue.
/// CEP:WHY: Use-def checks need the read set of each computation.
/// CEP:STATUS: complete
/// CEP:FAILURE: none
/// CEP:ASSUMES: none
/// CEP:COST: cold; constant per rvalue.
/// CEP:EVIDENCE: unit test rr_mir::verifier::tests::rejects_bad_target.
fn rvalue_locals(r: &Rvalue) -> Vec<LocalId> {
    match r {
        Rvalue::Use(o) => operand_locals(o),
        Rvalue::UnaryOp { e, .. } => operand_locals(e),
        Rvalue::BinaryOp { l, r, .. } => {
            let mut v = operand_locals(l);
            v.extend(operand_locals(r));
            v
        }
    }
}

/// CEP:WHAT: Lists locals read by one operand.
/// CEP:WHY: Use-def checks need the read set of each operand.
/// CEP:STATUS: complete
/// CEP:FAILURE: none
/// CEP:ASSUMES: none
/// CEP:COST: cold; constant per operand.
/// CEP:EVIDENCE: unit test rr_mir::verifier::tests::rejects_bad_target.
fn operand_locals(o: &Operand) -> Vec<LocalId> {
    match o {
        Operand::Copy(l) => vec![*l],
        Operand::Const(_) => Vec::new(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Sample valid constant used to build test programs.
    const SAMPLE_VALUE: i64 = 3;

    /// CEP:WHAT: Rejects a branch to a nonexistent block.
    /// CEP:WHY: Block linkage validity is a mandated verifier check.
    /// CEP:STATUS: complete
    /// CEP:FAILURE: Fails the test binary on regression.
    /// CEP:ASSUMES: none
    /// CEP:COST: cold; unit test only.
    /// CEP:EVIDENCE: cargo test -p rr-mir.
    #[test]
    fn rejects_bad_target() {
        let f = Function {
            name: "main".into(),
            locals: vec![Local { ty: rr_hir::hir::Ty::I64, is_immutable: true }],
            params: Vec::new(),
            ret: rr_hir::hir::Ty::I64,
            blocks: vec![
                BasicBlock {
                    stmts: Vec::new(),
                    term: Terminator::Goto(BAD_TARGET),
                },
            ],
        };
        let p = Program { version: IR_VERSION, fns: vec![f] };
        let diags = verify(&p);
        assert!(diags.iter().any(|d| d.contains("invalid block")));
    }

    /// CEP:WHAT: Accepts a well-formed program with no diagnostics.
    /// CEP:WHY: The verifier must be silent on valid input.
    /// CEP:STATUS: complete
    /// CEP:FAILURE: Fails the test binary on regression.
    /// CEP:ASSUMES: none
    /// CEP:COST: cold; unit test only.
    /// CEP:EVIDENCE: cargo test -p rr-mir.
    #[test]
    fn accepts_valid_program() {
        let f = Function {
            name: "main".into(),
            locals: vec![Local { ty: rr_hir::hir::Ty::I64, is_immutable: true }],
            params: Vec::new(),
            ret: rr_hir::hir::Ty::I64,
            blocks: vec![BasicBlock {
                stmts: vec![Statement::Assign { place: 0, rvalue: Rvalue::Use(Operand::Const(Constant::I64(SAMPLE_VALUE))) }],
                term: Terminator::Return,
            }],
        };
        let p = Program { version: IR_VERSION, fns: vec![f] };
        assert!(verify(&p).is_empty());
    }
}
