// CEP:FILE: ir/hir/src/span.rs
// CEP:WHAT: Byte-span tracking for source positions.
// CEP:WHY: Diagnostics must point at exact source locations per the HPC diagnostics contract.
// CEP:CLASS: HPC-1
// CEP:STATUS: complete
// CEP:FAILURE: none
// CEP:ASSUMES: Spans index into UTF-8 source bytes; end is exclusive.
// CEP:COST: compile-time only; plain data.
// CEP:EVIDENCE: unit test rr_hir::span::tests::span_operations.
#![deny(missing_docs)]

/// A half-open byte range in one source file.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Span {
    /// First byte of the range.
    pub lo: u32,
    /// One past the last byte of the range.
    pub hi: u32,
}

impl Span {
    /// CEP:WHAT: Builds a span from byte bounds.
    /// CEP:WHY: Lexers construct spans while scanning.
    /// CEP:STATUS: complete
    /// CEP:FAILURE: none
    /// CEP:ASSUMES: none
    /// CEP:COST: compile-time only; plain data.
    /// CEP:EVIDENCE: unit test rr_hir::span::tests::span_operations.
    pub fn new(lo: u32, hi: u32) -> Span {
        Span { lo, hi }
    }

    /// CEP:WHAT: Merges two spans into the covering range.
    /// CEP:WHY: Expression spans are the union of operand spans.
    /// CEP:STATUS: complete
    /// CEP:FAILURE: none
    /// CEP:ASSUMES: Both spans come from the same file.
    /// CEP:COST: compile-time only; plain data.
    /// CEP:EVIDENCE: unit test rr_hir::span::tests::span_operations.
    pub fn to(self, other: Span) -> Span {
        Span { lo: self.lo.min(other.lo), hi: self.hi.max(other.hi) }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Sample bound used by the span test.
    const MID: u32 = 5;

    /// Sample end bound used by the span test.
    const END: u32 = 9;

    /// CEP:WHAT: Verifies span construction and merging.
    /// CEP:WHY: Diagnostics depend on exact spans.
    /// CEP:STATUS: complete
    /// CEP:FAILURE: Fails the test binary on regression.
    /// CEP:ASSUMES: none
    /// CEP:COST: cold; unit test only.
    /// CEP:EVIDENCE: cargo test -p rr-hir.
    #[test]
    fn span_operations() {
        let a = Span::new(0, MID);
        let b = Span::new(MID, END);
        assert_eq!(a.to(b), Span::new(0, END));
    }
}
