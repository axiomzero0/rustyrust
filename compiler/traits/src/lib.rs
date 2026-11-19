// CEP:FILE: compiler/traits/src/lib.rs
// CEP:WHAT: Trait goal solving skeleton with canonicalized goals and memoization.
// CEP:WHY: The V0 dialect has no traits; the solver interface exists so V1 generics integrate without pipeline changes.
// CEP:CLASS: HPC-1
// CEP:STATUS: stub
// CEP:OWNER: axiomzero0
// CEP:TICKET: RR-101
// CEP:FAILURE: Returns Unsupported for every goal; never reports a false positive.
// CEP:ASSUMES: V0 programs emit zero goals; the type checker asserts this.
// CEP:COST: cold; memo lookup is linear over cached goals.
// CEP:EVIDENCE: unit test rr_traits::tests::goals_are_memoized.
//! Trait solver skeleton for the V1 milestone.
#![deny(missing_docs)]

/// A canonicalized trait goal.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Goal {
    /// Trait name.
    pub trait_name: String,
    /// Self type name.
    pub self_ty: String,
}

/// The outcome of solving one goal.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Solution {
    /// The goal holds.
    Holds,
    /// The goal does not hold.
    Fails,
    /// No solver support for the goal shape.
    Unsupported,
}

/// A goal solver with a memo table.
#[derive(Debug, Default)]
pub struct Solver {
    memo: Vec<(Goal, Solution)>,
}

impl Solver {
    /// CEP:WHAT: Solves one goal with memo lookup and Unsupported fallback.
    /// CEP:WHY: V1 will replace the fallback; the interface stays fixed.
    /// CEP:STATUS: stub
    /// CEP:OWNER: axiomzero0
    /// CEP:TICKET: RR-101
    /// CEP:FAILURE: none
    /// CEP:ASSUMES: Goals are canonical before solving.
    /// CEP:COST: cold; linear over the memo table.
    /// CEP:EVIDENCE: unit test rr_traits::tests::goals_are_memoized.
    pub fn solve(&mut self, g: Goal) -> Solution {
        if let Some((_, s)) = self.memo.iter().find(|(k, _)| k == &g) {
            return s.clone();
        }
        let s = Solution::Unsupported;
        self.memo.push((g, s.clone()));
        s
    }

    /// CEP:WHAT: Returns the number of memoized goals.
    /// CEP:WHY: Tests and diagnostics inspect solver state.
    /// CEP:STATUS: complete
    /// CEP:FAILURE: none
    /// CEP:ASSUMES: none
    /// CEP:COST: cold; constant time.
    /// CEP:EVIDENCE: unit test rr_traits::tests::goals_are_memoized.
    pub fn memo_len(&self) -> usize {
        self.memo.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Sample goal reused by the memo test.
    fn sample_goal() -> Goal {
        Goal { trait_name: "Send".into(), self_ty: "i64".into() }
    }

    /// CEP:WHAT: Verifies memoization of repeated goals.
    /// CEP:WHY: Canonicalized goals must solve once.
    /// CEP:STATUS: complete
    /// CEP:FAILURE: Fails the test binary on regression.
    /// CEP:ASSUMES: none
    /// CEP:COST: cold; unit test only.
    /// CEP:EVIDENCE: cargo test -p rr-traits.
    #[test]
    fn goals_are_memoized() {
        let mut s = Solver::default();
        assert_eq!(s.solve(sample_goal()), Solution::Unsupported);
        assert_eq!(s.solve(sample_goal()), Solution::Unsupported);
        assert_eq!(s.memo_len(), 1);
    }
}
