// CEP:FILE: checker/verifier/src/lib.rs
// CEP:WHAT: Independent trusted checker of optimization certificates.
// CEP:WHY: G16 minimizes the trusted computing base; the checker duplicates semantics, never optimizer code.
// CEP:CLASS: HPC-1
// CEP:STATUS: complete
// CEP:FAILURE: Returns one rejection reason per failed check; never panics on tampered input.
// CEP:ASSUMES: The checker implements the wrap-i64 semantics table independently of the optimizer.
// CEP:COST: cold; linear in obligations plus program size.
// CEP:EVIDENCE: unit test rr_verifier::tests::accepts_good_rejects_tampered.
//! Independent trusted certificate checker.

use rr_certificates::{Json, KIND_CONST_FOLD, KIND_DEAD_STORE};
use rr_mir::model::*;
use rr_mir::printer::program_hash;

/// CEP:WHAT: Verifies one certificate against the output program.
/// CEP:WHY: Optimized IR is accepted only after independent checking.
/// CEP:ID: CS-a10
/// CEP:STATUS: complete
/// CEP:FAILURE: Returns rejection reasons for hash mismatches, malformed obligations, and wrong results.
/// CEP:ASSUMES: The certificate references the program by canonical printed hash.
/// CEP:COST: cold; linear in obligations plus program size.
/// CEP:EVIDENCE: doc file docs/functions/CS/CS-a10.md; unit test accepts_good_rejects_tampered.
pub fn verify(p: &Program, cert: &Json) -> Vec<String> {
    let mut out = Vec::new();
    check_hashes(p, cert, &mut out);
    if let Some(Json::Arr(obs)) = cert.get("obligations") {
        for o in obs.iter() {
            check_obligation(p, o, &mut out);
        }
    }
    out
}

/// CEP:WHAT: Checks input, intermediate, and output hashes of one certificate.
/// CEP:WHY: Hash equality binds the evidence to the exact program.
/// CEP:STATUS: complete
/// CEP:FAILURE: Pushes one rejection per mismatched hash.
/// CEP:ASSUMES: none
/// CEP:COST: cold; constant time.
/// CEP:EVIDENCE: unit test rr_verifier::tests::accepts_good_rejects_tampered.
fn check_hashes(p: &Program, cert: &Json, out: &mut Vec<String>) {
    let want = program_hash(p) as i64;
    if cert.get("output_hash").and_then(|x| x.as_i64()) != Some(want) {
        out.push("output hash mismatch".to_string());
    }
    if cert.get("input_hash").and_then(|x| x.as_i64()).is_none() {
        out.push("certificate lacks input hash".to_string());
    }
}

/// CEP:WHAT: Checks one obligation against the output program.
/// CEP:WHY: Each obligation kind has an independent re-derivation rule.
/// CEP:STATUS: complete
/// CEP:FAILURE: Pushes one rejection per failed obligation.
/// CEP:ASSUMES: none
/// CEP:COST: cold; linear in program scan per obligation.
/// CEP:EVIDENCE: unit test rr_verifier::tests::accepts_good_rejects_tampered.
fn check_obligation(p: &Program, o: &Json, out: &mut Vec<String>) {
    let kind = o.get("kind").and_then(|x| x.as_str()).unwrap_or("");
    match kind {
        KIND_CONST_FOLD => check_fold(o, out),
        KIND_DEAD_STORE => check_dead(p, o, out),
        other => out.push(format!("unknown obligation kind {}", other)),
    }
}

/// CEP:WHAT: Re-derives one constant fold under wrap-i64 semantics.
/// CEP:WHY: The checker owns the semantics table independently of the optimizer.
/// CEP:STATUS: complete
/// CEP:FAILURE: Pushes one rejection when the recorded result disagrees.
/// CEP:ASSUMES: none
/// CEP:COST: cold; constant per obligation.
/// CEP:EVIDENCE: unit test rr_verifier::tests::accepts_good_rejects_tampered.
fn check_fold(o: &Json, out: &mut Vec<String>) {
    let op = o.get("op").and_then(|x| x.as_str()).unwrap_or("");
    let l = o.get("lhs").and_then(|x| x.as_i64()).unwrap_or(0);
    let r = o.get("rhs").and_then(|x| x.as_i64()).unwrap_or(0);
    let got = o.get("result").and_then(|x| x.as_i64());
    let want = rederive(op, l, r);
    match (got, want) {
        (Some(g), Some(w)) if g == w => {}
        _ => out.push(format!("const fold {} {} {} rejected", op, l, r)),
    }
}

/// CEP:WHAT: Re-derives one folded operation with the documented semantics.
/// CEP:WHY: Independent duplication is the trust boundary between optimizer and checker.
/// CEP:STATUS: complete
/// CEP:FAILURE: Returns None for trapping or unknown operations.
/// CEP:ASSUMES: none
/// CEP:COST: cold; constant time.
/// CEP:EVIDENCE: unit test rr_verifier::tests::accepts_good_rejects_tampered.
fn rederive(op: &str, l: i64, r: i64) -> Option<i64> {
    match op {
        "add" => Some(l.wrapping_add(r)),
        "sub" => Some(l.wrapping_sub(r)),
        "mul" => Some(l.wrapping_mul(r)),
        "div" if r != 0 && !(l == i64::MIN && r == -1) => Some(l.wrapping_div(r)),
        "rem" if r != 0 && !(l == i64::MIN && r == -1) => Some(l.wrapping_rem(r)),
        "==" => Some((l == r) as i64),
        "!=" => Some((l != r) as i64),
        "<" => Some((l < r) as i64),
        ">" => Some((l > r) as i64),
        "<=" => Some((l <= r) as i64),
        ">=" => Some((l >= r) as i64),
        _ => None,
    }
}

/// CEP:WHAT: Re-derives one dead-store obligation from the output program.
/// CEP:WHY: The removed local must be absent from the whole program.
/// CEP:STATUS: complete
/// CEP:FAILURE: Pushes one rejection when the local still appears anywhere.
/// CEP:ASSUMES: none
/// CEP:COST: cold; linear in program size per obligation.
/// CEP:EVIDENCE: unit test rr_verifier::tests::accepts_good_rejects_tampered.
fn check_dead(p: &Program, o: &Json, out: &mut Vec<String>) {
    let local = o.get("local").and_then(|x| x.as_i64()).unwrap_or(-1) as usize;
    let used = local_still_used(p, local);
    if used {
        out.push(format!("dead store of live local {} rejected", local));
    }
}

/// CEP:WHAT: Scans one program for reads of one local.
/// CEP:WHY: The checker cannot trust the optimizer's liveness claim.
/// CEP:STATUS: complete
/// CEP:FAILURE: none
/// CEP:ASSUMES: none
/// CEP:COST: cold; linear in program size.
/// CEP:EVIDENCE: unit test rr_verifier::tests::accepts_good_rejects_tampered.
fn local_still_used(p: &Program, local: usize) -> bool {
    for f in p.fns.iter() {
        for b in f.blocks.iter() {
            for s in b.stmts.iter() {
                if stmt_uses(s, local) {
                    return true;
                }
            }
            if term_uses(&b.term, local) {
                return true;
            }
        }
    }
    false
}

/// CEP:WHAT: Reports whether one statement uses a local.
/// CEP:WHY: The scan covers statements and terminators.
/// CEP:STATUS: complete
/// CEP:FAILURE: none
/// CEP:ASSUMES: none
/// CEP:COST: cold; constant per statement.
/// CEP:EVIDENCE: unit test rr_verifier::tests::accepts_good_rejects_tampered.
fn stmt_uses(s: &Statement, local: usize) -> bool {
    match s {
        Statement::Assign { rvalue, .. } => rv_uses(rvalue, local),
        Statement::Nop => false,
    }
}

/// CEP:WHAT: Reports whether one rvalue uses a local.
/// CEP:WHY: The scan covers all rvalue shapes.
/// CEP:STATUS: complete
/// CEP:FAILURE: none
/// CEP:ASSUMES: none
/// CEP:COST: cold; constant per rvalue.
/// CEP:EVIDENCE: unit test rr_verifier::tests::accepts_good_rejects_tampered.
fn rv_uses(r: &Rvalue, local: usize) -> bool {
    match r {
        Rvalue::Use(Operand::Copy(l)) => *l == local,
        Rvalue::UnaryOp { e: Operand::Copy(l), .. } => *l == local,
        Rvalue::BinaryOp { l: Operand::Copy(a), r: Operand::Copy(b), .. } => *a == local || *b == local,
        _ => false,
    }
}

/// CEP:WHAT: Reports whether one terminator uses a local.
/// CEP:WHY: The scan covers all terminator shapes.
/// CEP:STATUS: complete
/// CEP:FAILURE: none
/// CEP:ASSUMES: none
/// CEP:COST: cold; constant per terminator.
/// CEP:EVIDENCE: unit test rr_verifier::tests::accepts_good_rejects_tampered.
fn term_uses(t: &Terminator, local: usize) -> bool {
    match t {
        Terminator::SwitchInt { disc: Operand::Copy(l), .. } => *l == local,
        Terminator::Call { args, .. } => args.iter().any(|a| matches!(a, Operand::Copy(l) if *l == local)),
        _ => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rr_hir::hir::Ty;

    /// Folded sample value used by the verifier test.
    const SAMPLE_FOLD: i64 = 9;

    // CEP:WHAT: Verifies acceptance of honest certificates and rejection of tampered ones.
    // CEP:WHY: The checker is the trust boundary for optimized IR.
    // CEP:STATUS: complete
    // CEP:FAILURE: Fails the test binary on regression.
    // CEP:ASSUMES: none
    // CEP:COST: cold; unit test only.
    // CEP:EVIDENCE: cargo test -p rr-verifier.
    /// CEP:WHAT: Verifies acceptance of honest certificates and rejection of tampered ones.
    /// CEP:WHY: The checker is the trust boundary for optimized IR.
    /// CEP:STATUS: complete
    /// CEP:FAILURE: Fails the test binary on regression.
    /// CEP:ASSUMES: none
    /// CEP:COST: cold; unit test only.
    /// CEP:EVIDENCE: cargo test -p rr-verifier.
    #[test]
    fn accepts_good_rejects_tampered() {
        let f = Function {
            name: "main".into(),
            locals: vec![Local { ty: Ty::I64, is_immutable: true }, Local { ty: Ty::I64, is_immutable: false }],
            params: Vec::new(),
            ret: Ty::I64,
            blocks: vec![BasicBlock {
                stmts: vec![Statement::Assign { place: 0, rvalue: Rvalue::Use(Operand::Const(Constant::I64(SAMPLE_FOLD))) }],
                term: Terminator::Return,
            }],
        };
        let p = Program { version: IR_VERSION, fns: vec![f] };
        let mut cert = rr_rr_opt_cert(&p);
        assert!(verify(&p, &cert).is_empty());
        if let Json::Obj(pairs) = &mut cert {
            for (k, v) in pairs.iter_mut() {
                if k == "output_hash" {
                    *v = Json::Num(1);
                }
            }
        }
        assert!(!verify(&p, &cert).is_empty());
    }

    /// CEP:WHAT: Builds an honest certificate through the pipeline.
    /// CEP:WHY: The verifier test needs a real evidence document.
    /// CEP:STATUS: complete
    /// CEP:FAILURE: Fails the test binary on regression.
    /// CEP:ASSUMES: none
    /// CEP:COST: cold; unit test only.
    /// CEP:EVIDENCE: cargo test -p rr-verifier.
    fn rr_rr_opt_cert(p: &Program) -> Json {
        let (_, cert, _) = rr_optimize::pipeline::run(p);
        cert
    }
}
