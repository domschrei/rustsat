use rustsat::{
    instances::{BasicVarManager, SatInstance},
    lit,
    solvers::{SolveIncremental, SolverResult},
    types::Lit,
};
use rustsat_glucose::core::Glucose;

fn test_assumption_sequence<S: SolveIncremental>(mut solver: S) {
    let inst: SatInstance<BasicVarManager> =
        SatInstance::from_dimacs_path("./data/small.cnf").unwrap();
    solver.add_cnf(inst.as_cnf().0).unwrap();
    let res = solver.solve().unwrap();
    assert_eq!(res, SolverResult::Sat);
    let res = solver.solve_assumps(vec![!lit![0], !lit![1]]).unwrap();
    assert_eq!(res, SolverResult::Unsat);
    let res = solver
        .solve_assumps(vec![lit![0], lit![1], lit![2], lit![3]])
        .unwrap();
    assert_eq!(res, SolverResult::Unsat);
    let res = solver
        .solve_assumps(vec![lit![0], lit![1], lit![2], !lit![3]])
        .unwrap();
    assert_eq!(res, SolverResult::Unsat);
    let res = solver
        .solve_assumps(vec![lit![0], lit![1], !lit![2], lit![3]])
        .unwrap();
    assert_eq!(res, SolverResult::Unsat);
    let res = solver
        .solve_assumps(vec![lit![0], lit![1], !lit![2], !lit![3]])
        .unwrap();
    assert_eq!(res, SolverResult::Sat);
    let res = solver
        .solve_assumps(vec![lit![0], !lit![1], lit![2], lit![3]])
        .unwrap();
    assert_eq!(res, SolverResult::Unsat);
    let res = solver
        .solve_assumps(vec![lit![0], !lit![1], lit![2], !lit![3]])
        .unwrap();
    assert_eq!(res, SolverResult::Sat);
    let res = solver
        .solve_assumps(vec![lit![0], !lit![1], !lit![2], lit![3]])
        .unwrap();
    assert_eq!(res, SolverResult::Unsat);
    let res = solver
        .solve_assumps(vec![lit![0], !lit![1], !lit![2], !lit![3]])
        .unwrap();
    assert_eq!(res, SolverResult::Unsat);
    let res = solver
        .solve_assumps(vec![!lit![0], lit![1], lit![2], lit![3]])
        .unwrap();
    assert_eq!(res, SolverResult::Unsat);
    let res = solver
        .solve_assumps(vec![!lit![0], lit![1], lit![2], !lit![3]])
        .unwrap();
    assert_eq!(res, SolverResult::Unsat);
    let res = solver
        .solve_assumps(vec![!lit![0], lit![1], !lit![2], lit![3]])
        .unwrap();
    assert_eq!(res, SolverResult::Sat);
    let res = solver
        .solve_assumps(vec![!lit![0], lit![1], !lit![2], !lit![3]])
        .unwrap();
    assert_eq!(res, SolverResult::Sat);
    let res = solver
        .solve_assumps(vec![!lit![0], !lit![1], lit![2], lit![3]])
        .unwrap();
    assert_eq!(res, SolverResult::Unsat);
    let res = solver
        .solve_assumps(vec![!lit![0], !lit![1], lit![2], !lit![3]])
        .unwrap();
    assert_eq!(res, SolverResult::Unsat);
    let res = solver
        .solve_assumps(vec![!lit![0], !lit![1], !lit![2], lit![3]])
        .unwrap();
    assert_eq!(res, SolverResult::Unsat);
    let res = solver
        .solve_assumps(vec![!lit![0], !lit![1], !lit![2], !lit![3]])
        .unwrap();
    assert_eq!(res, SolverResult::Unsat);
}

#[test]
fn assumption_sequence() {
    let solver = Glucose::default();
    test_assumption_sequence(solver);
}
