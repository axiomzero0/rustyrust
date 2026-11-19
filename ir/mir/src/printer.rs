// CEP:FILE: ir/mir/src/printer.rs
// CEP:WHAT: Deterministic MIR printer and stable FNV-1a hash over printed form.
// CEP:WHY: Sections 38.17 through 38.19 require printable, hashable, deterministic IR.
// CEP:CLASS: HPC-1
// CEP:STATUS: complete
// CEP:FAILURE: none
// CEP:ASSUMES: Hashing runs over the printed text; print order is stable by construction.
// CEP:COST: cold; linear in program size.
// CEP:EVIDENCE: unit test rr_mir::printer::tests::hash_is_stable.
#![deny(missing_docs)]
use crate::model::*;
use rr_hir::hir::{Ty, UnOp};
use std::fmt::Write;

/// CEP:WHAT: Prints one MIR program to canonical text.
/// CEP:WHY: Certificates and logs reference MIR by its printed form.
/// CEP:ID: IR-a01
/// CEP:STATUS: complete
/// CEP:FAILURE: none
/// CEP:ASSUMES: Functions and blocks print in declaration order.
/// CEP:COST: cold; linear in program size.
/// CEP:EVIDENCE: doc file docs/functions/IR/IR-a01.md; unit test hash_is_stable.
pub fn print_program(p: &Program) -> String {
    let mut out = String::new();
    let _ = writeln!(out, "mir v{}", p.version);
    for f in p.fns.iter() {
        print_function(&mut out, f);
    }
    out
}

/// CEP:WHAT: Prints one MIR function.
/// CEP:WHY: Function output is the unit of golden IR comparison.
/// CEP:STATUS: complete
/// CEP:FAILURE: none
/// CEP:ASSUMES: none
/// CEP:COST: cold; linear in function size.
/// CEP:EVIDENCE: unit test rr_mir::printer::tests::hash_is_stable.
pub fn print_function(out: &mut String, f: &Function) {
    let _ = writeln!(out, "fn {} -> {}", f.name, ty_name(f.ret));
    for (i, l) in f.locals.iter().enumerate() {
        let _ = writeln!(out, "  local {}: {}{}", i, ty_name(l.ty), if l.is_immutable { " param" } else { "" });
    }
    for (i, b) in f.blocks.iter().enumerate() {
        let _ = writeln!(out, "  block {}:", i);
        for s in b.stmts.iter() {
            print_statement(out, s);
        }
        print_terminator(out, &b.term);
    }
}

/// CEP:WHAT: Prints one statement.
/// CEP:WHY: Statement text is part of the canonical form.
/// CEP:STATUS: complete
/// CEP:FAILURE: none
/// CEP:ASSUMES: none
/// CEP:COST: cold; linear in statement size.
/// CEP:EVIDENCE: unit test rr_mir::printer::tests::hash_is_stable.
fn print_statement(out: &mut String, s: &Statement) {
    match s {
        Statement::Nop => {
            let _ = writeln!(out, "    nop");
        }
        Statement::Assign { place, rvalue } => {
            let _ = write!(out, "    _{} = ", place);
            print_rvalue(out, rvalue);
            let _ = writeln!(out);
        }
    }
}

/// CEP:WHAT: Prints one rvalue.
/// CEP:WHY: Rvalue text is part of the canonical form.
/// CEP:STATUS: complete
/// CEP:FAILURE: none
/// CEP:ASSUMES: none
/// CEP:COST: cold; constant per rvalue.
/// CEP:EVIDENCE: unit test rr_mir::printer::tests::hash_is_stable.
fn print_rvalue(out: &mut String, r: &Rvalue) {
    match r {
        Rvalue::Use(o) => print_operand(out, o),
        Rvalue::UnaryOp { op, e } => {
            let _ = write!(out, "{} ", un_symbol(op));
            print_operand(out, e);
        }
        Rvalue::BinaryOp { op, l, r } => {
            print_operand(out, l);
            let _ = write!(out, " {} ", op.symbol());
            print_operand(out, r);
        }
    }
}

/// CEP:WHAT: Prints one operand.
/// CEP:WHY: Operand text is part of the canonical form.
/// CEP:STATUS: complete
/// CEP:FAILURE: none
/// CEP:ASSUMES: none
/// CEP:COST: cold; constant per operand.
/// CEP:EVIDENCE: unit test rr_mir::printer::tests::hash_is_stable.
fn print_operand(out: &mut String, o: &Operand) {
    match o {
        Operand::Copy(l) => {
            let _ = write!(out, "_{}", l);
        }
        Operand::Const(c) => print_constant(out, c),
    }
}

/// CEP:WHAT: Prints one constant.
/// CEP:WHY: Constant text is part of the canonical form.
/// CEP:STATUS: complete
/// CEP:FAILURE: none
/// CEP:ASSUMES: none
/// CEP:COST: cold; constant per constant.
/// CEP:EVIDENCE: unit test rr_mir::printer::tests::hash_is_stable.
fn print_constant(out: &mut String, c: &Constant) {
    match c {
        Constant::I64(v) => {
            let _ = write!(out, "{}", v);
        }
        Constant::Bool(b) => {
            let _ = write!(out, "{}", b);
        }
        Constant::Unit => {
            let _ = write!(out, "()");
        }
    }
}

/// CEP:WHAT: Prints one terminator.
/// CEP:WHY: Terminator text is part of the canonical form.
/// CEP:ID: IR-a03
/// CEP:STATUS: complete
/// CEP:FAILURE: none
/// CEP:ASSUMES: none
/// CEP:COST: cold; constant per terminator.
/// CEP:EVIDENCE: unit test rr_mir::printer::tests::hash_is_stable.
fn print_terminator(out: &mut String, t: &Terminator) {
    match t {
        Terminator::Goto(b) => {
            let _ = writeln!(out, "    goto bb{}", b);
        }
        Terminator::SwitchInt { disc, targets, otherwise } => {
            print_operand(out, disc);
            let _ = write!(out, " => switch");
            for (v, b) in targets.iter() {
                let _ = write!(out, " {} -> bb{}", v, b);
            }
            let _ = writeln!(out, " else bb{}", otherwise);
        }
        Terminator::Call { callee, args, dest } => {
            let _ = write!(out, "    call {}", callee);
            for a in args.iter() {
                let _ = write!(out, " ");
                print_operand(out, a);
            }
            match dest {
                Some((l, b)) => {
                    let _ = writeln!(out, " -> _{}, bb{}", l, b);
                }
                None => {
                    let _ = writeln!(out);
                }
            }
        }
        Terminator::Return => {
            let _ = writeln!(out, "    return");
        }
        Terminator::Unreachable => {
            let _ = writeln!(out, "    unreachable");
        }
    }
}

/// CEP:WHAT: Returns the printable name of one type.
/// CEP:WHY: Function signatures print type names.
/// CEP:STATUS: complete
/// CEP:FAILURE: none
/// CEP:ASSUMES: none
/// CEP:COST: cold; constant time.
/// CEP:EVIDENCE: unit test rr_mir::printer::tests::hash_is_stable.
fn ty_name(t: Ty) -> &'static str {
    match t {
        Ty::I64 => "i64",
        Ty::Bool => "bool",
        Ty::Unit => "unit",
    }
}

/// CEP:WHAT: Returns the printable symbol of one unary operator.
/// CEP:WHY: Rvalue text prints operator symbols.
/// CEP:STATUS: complete
/// CEP:FAILURE: none
/// CEP:ASSUMES: none
/// CEP:COST: cold; constant time.
/// CEP:EVIDENCE: unit test rr_mir::printer::tests::hash_is_stable.
fn un_symbol(op: &UnOp) -> &'static str {
    match op {
        UnOp::Neg => "-",
        UnOp::Not => "!",
    }
}

/// Byte count used by the FNV-1a prime multiplication step.
const FNV_PRIME: u64 = 0x01000193;

/// FNV-1a offset basis.
const FNV_BASIS: u64 = 0x811c9dc5;

/// CEP:WHAT: Computes the FNV-1a 64-bit hash of a byte string.
/// CEP:WHY: The IR hash must be stable across runs and platforms without external crates.
/// CEP:STATUS: complete
/// CEP:FAILURE: none
/// CEP:ASSUMES: none
/// CEP:COST: cold; linear in input length.
/// CEP:EVIDENCE: unit test rr_mir::printer::tests::hash_is_stable.
pub fn fnv1a(bytes: &[u8]) -> u64 {
    let mut h = FNV_BASIS;
    for b in bytes.iter() {
        h ^= *b as u64;
        h = h.wrapping_mul(FNV_PRIME);
    }
    h
}

/// CEP:WHAT: Hashes one program over its canonical printed form.
/// CEP:WHY: Certificates reference programs by hash, not by identity.
/// CEP:STATUS: complete
/// CEP:FAILURE: none
/// CEP:ASSUMES: The printer output is deterministic.
/// CEP:COST: cold; linear in program size.
/// CEP:EVIDENCE: unit test rr_mir::printer::tests::hash_is_stable.
pub fn program_hash(p: &Program) -> u64 {
    fnv1a(print_program(p).as_bytes())
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Sample constant used by the hash test.
    const SAMPLE_CONST: i64 = 7;

    /// CEP:WHAT: Verifies that equal programs hash equally and printing is deterministic.
    /// CEP:WHY: Section 38.19 forbids hash-order dependence.
    /// CEP:STATUS: complete
    /// CEP:FAILURE: Fails the test binary on regression.
    /// CEP:ASSUMES: none
    /// CEP:COST: cold; unit test only.
    /// CEP:EVIDENCE: cargo test -p rr-mir.
    #[test]
    fn hash_is_stable() {
        let f = Function {
            name: "main".into(),
            locals: vec![Local { ty: Ty::I64, is_immutable: true }],
            params: Vec::new(),
            ret: Ty::I64,
            blocks: vec![BasicBlock {
                stmts: vec![Statement::Assign { place: 0, rvalue: Rvalue::Use(Operand::Const(Constant::I64(SAMPLE_CONST))) }],
                term: Terminator::Return,
            }],
        };
        let p1 = Program { version: IR_VERSION, fns: vec![f.clone()] };
        let p2 = Program { version: IR_VERSION, fns: vec![f] };
        assert_eq!(program_hash(&p1), program_hash(&p2));
        assert!(print_program(&p1).starts_with("mir v1\nfn main -> i64"));
    }
}
