//! # IPASIR-UP Interface

use rustsat::types::{Assignment, Clause, Lit};

/// Trait representing the IPASIR-UP interface
pub trait ExternalPropagate {
    /// Returns true if the propagator only checks complete assignments
    #[must_use]
    fn is_lazy() -> bool {
        false
    }

    /// Notify the propagator about assignments to observed variables. The notification is not
    /// necessarily eager. It usually happens before the call of propagator callbacks and when a
    /// driving clause is leading to an assignment.
    fn assignment(&mut self, lit: Lit, is_fixed: bool);

    /// Notify the propagator about a new decision level
    fn new_decision_level(&mut self);

    /// Notify the propagator of backtracking to a given level
    fn backtrack(&mut self, new_level: usize);

    /// Checks the satisfiability of the current model
    ///
    /// If it returns `false`, the propagator must provide an external clause during the next call
    /// to [`Self::add_external_clause`]
    #[must_use]
    fn check_found_model(&mut self, solution: &Assignment) -> bool;

    /// Ask the external propagator for the next decision literal. If it returns 0, the solver
    /// makes its own choice.
    #[must_use]
    fn decide(&mut self) -> Option<Lit> {
        None
    }

    /// Ask the external propagator if there is an external propagation to make under the current
    /// assignment. It returns either a literal to be propagated or 0, indicating that there is no
    /// external propagation under the current assignment.
    #[must_use]
    fn propagate(&mut self) -> Option<Lit> {
        None
    }

    /// Ask the external propagator for the reason clause of a previous external propagation step
    /// (done by `propagate`). The clause must contain the propagated literal.
    #[must_use]
    #[allow(unused_variables)]
    fn add_reason_clause(&mut self, propagated_lit: Lit) -> Clause {
        Clause::default()
    }

    /// The solver queries the external propagator whether there is an external clause to be added
    ///
    /// The clause can be arbitrary, but if it is root-satisfied or tautology, the solver will
    /// ignore it without learning it. Root-falsified literals are eagerly removed from the clause.
    /// Falsified clauses trigger conflict analysis, propagating clauses trigger propagation. In
    /// case chrono is 0, the solver backtracks to propagate the new literal on the right decision
    /// level, otherwise it potentially will be an out-of-order assignment on the current level.
    /// Unit clauses always (unless root-satisfied, see above) trigger backtracking (independently
    /// from the value of the chrono option and independently from being falsified or satisfied or
    /// unassigned) to level 0. Empty clause (or root falsified clause, see above) makes the
    /// problem unsat and stops the search immediately. A literal 0 must close the clause.
    #[must_use]
    fn add_external_clause(&mut self) -> Option<Clause>;
}
