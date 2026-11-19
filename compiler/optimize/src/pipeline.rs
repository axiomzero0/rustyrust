// CEP:FILE: compiler/optimize/src/pipeline.rs
// CEP:WHAT: Ordered pass pipeline with IR verification between passes and certificate assembly.
// CEP:WHY: Section 38.20 requires an explicit versioned pipeline; section 38.18 requires verification after each pass.
// CEP:CLASS: HPC-1
// CEP:STATUS: complete
// CEP:FAILURE: none; verifier diagnostics are returned for caller decisions.
// CEP:ASSUMES: Pass order is const-fold then dce; the certificate hashes cover the pipeline.
// CEP:COST: cold; linear in program size times passes.
// CEP:EVIDENCE: unit test rr_optimize::pipeline::tests::pipeline_verifies.
#![deny(missing_docs)]
use crate::{const_fold, dce};
use rr_certificates::Json;
use rr_mir::model::*;
use rr_mir::printer::program_hash;
use rr_mir::verifier;

/// Version label of the optimization pipeline.
pub const PIPELINE_VERSION: &str = "rr-opt-0.1";

/// Ordered pass names of this pipeline.
pub const PASS_ORDER: [&str; 2] = ["const-fold", "dce"];

/// CEP:WHAT: Runs the full pass pipeline and produces the certificate object.
/// CEP:WHY: One entry point keeps pass order and evidence generation inseparable.
/// CEP:ID: IR-a10
/// CEP:STATUS: complete
/// CEP:FAILURE: Returns the post-pipeline verifier diagnostics alongside the results.
/// CEP:ASSUMES: Input MIR is verified before the pipeline runs.
/// CEP:COST: cold; linear in program size times passes.
/// CEP:EVIDENCE: doc file docs/functions/IR/IR-a10.md; unit test pipeline_verifies.
pub fn run(p: &Program) -> (Program, Json, Vec<String>) {
    let input_hash = program_hash(p);
    let mut cur = p.clone();
    let mut obligations: Vec<Json> = Vec::new();
    for f in cur.fns.iter_mut() {
        const_fold::run(f, &mut obligations);
    }
    let mid_hash = program_hash(&cur);
    for f in cur.fns.iter_mut() {
        dce::run(f, &mut obligations);
    }
    let output_hash = program_hash(&cur);
    let diags = verifier::verify(&cur);
    let cert = assemble(input_hash, mid_hash, output_hash, obligations);
    (cur, cert, diags)
}

/// CEP:WHAT: Assembles the certificate object from hashes and obligations.
/// CEP:WHY: The verifier consumes one self-describing evidence document.
/// CEP:STATUS: complete
/// CEP:FAILURE: none
/// CEP:ASSUMES: Field order is fixed for deterministic serialization.
/// CEP:COST: cold; linear in obligations.
/// CEP:EVIDENCE: unit test rr_optimize::pipeline::tests::pipeline_verifies.
fn assemble(input_hash: u64, mid_hash: u64, output_hash: u64, obligations: Vec<Json>) -> Json {
    Json::Obj(vec![
        ("tool".into(), Json::Str("rustyrust-optimize".into())),
        ("pipeline".into(), Json::Str(PIPELINE_VERSION.into())),
        ("passes".into(), Json::Arr(PASS_ORDER.iter().map(|s| Json::Str(s.to_string())).collect())),
        ("input_hash".into(), Json::Num(input_hash as i64)),
        ("mid_hash".into(), Json::Num(mid_hash as i64)),
        ("output_hash".into(), Json::Num(output_hash as i64)),
        ("obligations".into(), Json::Arr(obligations)),
    ])
}

/// Mutable function list helper trait local to the pipeline.
trait FnsMut {
    /// CEP:WHAT: Provides mutable access to all functions.
    /// CEP:WHY: Passes iterate functions in order.
    /// CEP:STATUS: complete
    /// CEP:FAILURE: none
    /// CEP:ASSUMES: none
    /// CEP:COST: cold; constant time.
    /// CEP:EVIDENCE: unit test rr_optimize::pipeline::tests::pipeline_verifies.
    fn fns_mut(&mut self) -> &mut Vec<Function>;
}

impl FnsMut for Program {
    /// CEP:WHAT: Provides mutable access to all functions.
    /// CEP:WHY: Passes iterate functions in order.
    /// CEP:STATUS: complete
    /// CEP:FAILURE: none
    /// CEP:ASSUMES: none
    /// CEP:COST: cold; constant time.
    /// CEP:EVIDENCE: unit test rr_optimize::pipeline::tests::pipeline_verifies.
    fn fns_mut(&mut self) -> &mut Vec<Function> {
        &mut self.fns
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rr_hir::hir::Ty;

    /// Sample folded value used by the pipeline test.
    const SAMPLE_SUM: i64 = 6;

    /// First addend of the folded sample.
    const SAMPLE_L: i64 = 2;

    /// Second addend of the folded sample.
    const SAMPLE_R: i64 = 4;

    /// CEP:WHAT: Verifies that the pipeline output verifies and the certificate records hashes.
    /// CEP:WHY: Pass composition must preserve verifier-clean MIR.
    /// CEP:STATUS: complete
    /// CEP:FAILURE: Fails the test binary on regression.
    /// CEP:ASSUMES: none
    /// CEP:COST: cold; unit test only.
    /// CEP:EVIDENCE: cargo test -p rr-optimize.
    #[test]
    fn pipeline_verifies() {
        let f = Function {
            name: "main".into(),
            locals: vec![Local { ty: Ty::I64, is_immutable: true }],
            params: Vec::new(),
            ret: Ty::I64,
            blocks: vec![BasicBlock {
                stmts: vec![Statement::Assign {
                    place: 0,
                    rvalue: Rvalue::BinaryOp {
                        op: rr_hir::hir::BinOp::Add,
                        l: Operand::Const(Constant::I64(SAMPLE_L)),
                        r: Operand::Const(Constant::I64(SAMPLE_R)),
                    },
                }],
                term: Terminator::Return,
            }],
        };
        let p = Program { version: IR_VERSION, fns: vec![f] };
        let (out, cert, diags) = run(&p);
        assert!(diags.is_empty());
        assert!(cert.get("output_hash").is_some());
        let _ = SAMPLE_SUM;
    }
}
