//! # IPASIR Interface
//!
//! Interface to any SAT solver implementing the
//! [IPASIR API](https://github.com/biotomas/ipasir) for incremental SAT solvers.

use core::ffi::{c_void, CStr};

use super::{
    ControlSignal, IncrementalSolve, InternalSolverState, OptLearnCallbackStore,
    OptTermCallbackStore, Solve, SolveMightFail, SolveStats, SolverError, SolverResult,
    SolverState,
};
use crate::types::{Clause, Lit, TernaryVal, Var};
use ffi::IpasirHandle;

/// Type for an IPASIR solver.
pub struct IpasirSolver<'a> {
    handle: *mut IpasirHandle,
    state: InternalSolverState,
    terminate_cb: OptTermCallbackStore<'a>,
    learner_cb: OptLearnCallbackStore<'a>,
    n_sat: u32,
    n_unsat: u32,
    n_terminated: u32,
    n_clauses: u32,
    max_var: Option<Var>,
    avg_clause_len: f32,
    cpu_solve_time: f32,
}

impl Default for IpasirSolver<'_> {
    fn default() -> Self {
        Self {
            handle: unsafe { ffi::ipasir_init() },
            state: Default::default(),
            terminate_cb: Default::default(),
            learner_cb: Default::default(),
            n_sat: Default::default(),
            n_unsat: Default::default(),
            n_terminated: Default::default(),
            n_clauses: Default::default(),
            max_var: Default::default(),
            avg_clause_len: Default::default(),
            cpu_solve_time: Default::default(),
        }
    }
}

impl<'a> IpasirSolver<'a> {
    fn get_core_assumps(&self, assumps: &Vec<Lit>) -> Result<Vec<Lit>, SolverError> {
        let mut core = Vec::new();
        core.reserve(assumps.len());
        for a in assumps {
            match unsafe { ffi::ipasir_failed(self.handle, a.to_ipasir()) } {
                0 => (),
                1 => core.push(!*a),
                invalid => {
                    return Err(SolverError::API(format!(
                        "ipasir_failed returned invalid value: {}",
                        invalid
                    )))
                }
            }
        }
        Ok(core)
    }

    /// Sets a terminator callback that is regularly called during solving.
    ///
    /// # Examples
    ///
    /// Terminate solver after 10 callback calls.
    ///
    /// ```
    /// use rustsat::solvers::{IpasirSolver, ControlSignal, Solve, SolverResult};
    ///
    /// let mut solver = IpasirSolver::new();
    ///
    /// // Load instance
    ///
    /// let mut cnt = 1;
    /// solver.set_terminator(move || {
    ///     if cnt > 10 {
    ///         ControlSignal::Terminate
    ///     } else {
    ///         cnt += 1;
    ///         ControlSignal::Continue
    ///     }
    /// });
    ///
    /// let ret = solver.solve().unwrap();
    ///
    /// // Assuming an instance is actually loaded and runs long enough
    /// // assert_eq!(ret, SolverResult::Interrupted);
    /// ```
    pub fn set_terminator<CB>(&mut self, cb: CB)
    where
        CB: FnMut() -> ControlSignal + 'a,
    {
        self.terminate_cb = Some(Box::new(Box::new(cb)));
        let cb_ptr = self.terminate_cb.as_mut().unwrap().as_mut() as *const _ as *const c_void;
        unsafe { ffi::ipasir_set_terminate(self.handle, cb_ptr, ffi::ipasir_terminate_cb) }
    }

    /// Sets a learner callback that gets passed clauses up to a certain length learned by the solver.
    ///
    /// The callback goes out of scope with the solver, afterwards captured variables become accessible.
    ///
    /// # Examples
    ///
    /// Count number of learned clauses up to length 10.
    ///
    /// ```
    /// use rustsat::solvers::{IpasirSolver, Solve, SolverResult};
    ///
    /// let mut cnt = 0;
    ///
    /// {
    ///     let mut solver = IpasirSolver::new();
    ///     // Load instance
    ///
    ///     solver.set_learner(|_| cnt += 1, 10);
    ///
    ///     solver.solve().unwrap();
    /// }
    ///
    /// // cnt variable can be accessed from here on
    /// ```
    pub fn set_learner<CB>(&mut self, cb: CB, max_len: usize)
    where
        CB: FnMut(Vec<Lit>) + 'a,
    {
        self.learner_cb = Some(Box::new(Box::new(cb)));
        let cb_ptr = self.learner_cb.as_mut().unwrap().as_mut() as *const _ as *const c_void;
        unsafe {
            ffi::ipasir_set_learn(
                self.handle,
                cb_ptr,
                max_len.try_into().unwrap(),
                ffi::ipasir_learn_cb,
            )
        }
    }
}

impl Solve for IpasirSolver<'_> {
    fn new() -> Self {
        Self::default()
    }

    fn signature(&self) -> &'static str {
        let c_chars = unsafe { ffi::ipasir_signature() };
        let c_str = unsafe { CStr::from_ptr(c_chars) };
        c_str
            .to_str()
            .expect("IPASIR signature returned invalid UTF-8.")
    }

    fn solve(&mut self) -> Result<SolverResult, SolverError> {
        // If already solved, return state
        if let InternalSolverState::Sat = self.state {
            return Ok(SolverResult::SAT);
        } else if let InternalSolverState::Unsat(_) = self.state {
            return Ok(SolverResult::UNSAT);
        } else if let InternalSolverState::Error(desc) = &self.state {
            return Err(SolverError::State(
                SolverState::Error(desc.clone()),
                SolverState::Input,
            ));
        }
        // Solve with IPASIR backend
        match unsafe { ffi::ipasir_solve(self.handle) } {
            0 => {
                self.n_terminated += 1;
                self.state = InternalSolverState::Input;
                Ok(SolverResult::Interrupted)
            }
            10 => {
                self.n_sat += 1;
                self.state = InternalSolverState::Sat;
                Ok(SolverResult::SAT)
            }
            20 => {
                self.n_unsat += 1;
                self.state = InternalSolverState::Unsat(vec![]);
                Ok(SolverResult::UNSAT)
            }
            invalid => Err(SolverError::API(format!(
                "ipasir_solve returned invalid value: {}",
                invalid
            ))),
        }
    }

    fn lit_val(&self, lit: &Lit) -> Result<TernaryVal, SolverError> {
        match &self.state {
            InternalSolverState::Sat => {
                let lit = lit.to_ipasir();
                match unsafe { ffi::ipasir_val(self.handle, lit) } {
                    0 => Ok(TernaryVal::DontCare),
                    p if p == lit => Ok(TernaryVal::True),
                    n if n == -lit => Ok(TernaryVal::False),
                    invalid => Err(SolverError::API(format!(
                        "ipasir_val returned invalid value: {}",
                        invalid
                    ))),
                }
            }
            other => Err(SolverError::State(other.to_external(), SolverState::SAT)),
        }
    }

    fn add_clause(&mut self, clause: Clause) -> SolveMightFail {
        if let InternalSolverState::Error(_) = self.state {
            // Don't add clause if already in error state.
            return Err(SolverError::State(
                self.state.to_external(),
                SolverState::Input,
            ));
        }
        // Update wrapper-internal state
        self.n_clauses += 1;
        clause.iter().for_each(|l| match self.max_var {
            None => self.max_var = Some(l.var()),
            Some(var) => {
                if l.var() > var {
                    self.max_var = Some(l.var())
                }
            }
        });
        self.avg_clause_len = (self.avg_clause_len * ((self.n_clauses - 1) as f32)
            + clause.len() as f32)
            / self.n_clauses as f32;
        self.state = InternalSolverState::Input;
        // Call IPASIR backend
        for lit in &clause {
            unsafe { ffi::ipasir_add(self.handle, lit.to_ipasir()) }
        }
        unsafe { ffi::ipasir_add(self.handle, 0) };
        Ok(())
    }
}

impl IncrementalSolve for IpasirSolver<'_> {
    fn solve_assumps(&mut self, assumps: Vec<Lit>) -> Result<SolverResult, SolverError> {
        // If in error state, remain there
        // If not, need to resolve because assumptions might have changed
        if let InternalSolverState::Error(desc) = &self.state {
            return Err(SolverError::State(
                SolverState::Error(desc.clone()),
                SolverState::Input,
            ));
        }
        // Solve with IPASIR backend
        for a in &assumps {
            unsafe { ffi::ipasir_assume(self.handle, a.to_ipasir()) }
        }
        match unsafe { ffi::ipasir_solve(self.handle) } {
            0 => {
                self.n_terminated += 1;
                self.state = InternalSolverState::Input;
                Ok(SolverResult::Interrupted)
            }
            10 => {
                self.n_sat += 1;
                self.state = InternalSolverState::Sat;
                Ok(SolverResult::SAT)
            }
            20 => {
                self.n_unsat += 1;
                self.state = InternalSolverState::Unsat(self.get_core_assumps(&assumps)?);
                Ok(SolverResult::UNSAT)
            }
            invalid => Err(SolverError::API(format!(
                "ipasir_solve returned invalid value: {}",
                invalid
            ))),
        }
    }

    fn core(&mut self) -> Result<Vec<Lit>, SolverError> {
        match &self.state {
            InternalSolverState::Unsat(core) => Ok(core.clone()),
            other => Err(SolverError::State(other.to_external(), SolverState::UNSAT)),
        }
    }
}

impl SolveStats for IpasirSolver<'_> {
    fn n_sat_solves(&self) -> u32 {
        self.n_sat
    }

    fn n_unsat_solves(&self) -> u32 {
        self.n_unsat
    }

    fn n_terminated(&self) -> u32 {
        self.n_terminated
    }

    fn n_clauses(&self) -> u32 {
        self.n_clauses
    }

    fn max_var(&self) -> Option<Var> {
        self.max_var
    }

    fn avg_clause_len(&self) -> f32 {
        self.avg_clause_len
    }

    fn cpu_solve_time(&self) -> f32 {
        self.cpu_solve_time
    }
}

impl Drop for IpasirSolver<'_> {
    fn drop(&mut self) {
        unsafe { ffi::ipasir_release(self.handle) }
    }
}

#[cfg(test)]
mod test {
    use super::IpasirSolver;
    use crate::{
        lit,
        solvers::{ControlSignal, Solve, SolverResult},
        types::Lit,
    };

    #[test]
    fn build_destroy() {
        let _solver = IpasirSolver::new();
    }

    #[test]
    fn build_two() {
        let _solver1 = IpasirSolver::new();
        let _solver2 = IpasirSolver::new();
    }

    #[test]
    fn tiny_instance() {
        let mut solver = IpasirSolver::new();
        solver.add_binary(lit![0], !lit![1]).unwrap();
        solver.add_binary(lit![1], !lit![2]).unwrap();
        let ret = solver.solve();
        match ret {
            Err(e) => panic!("got error when solving: {}", e),
            Ok(res) => assert_eq!(res, SolverResult::SAT),
        }
    }

    #[test]
    fn termination_callback() {
        let mut solver = IpasirSolver::new();
        solver.add_binary(lit![0], !lit![1]).unwrap();
        solver.add_binary(lit![1], !lit![2]).unwrap();
        solver.add_binary(lit![2], !lit![3]).unwrap();
        solver.add_binary(lit![3], !lit![4]).unwrap();
        solver.add_binary(lit![4], !lit![5]).unwrap();
        solver.add_binary(lit![5], !lit![6]).unwrap();
        solver.add_binary(lit![6], !lit![7]).unwrap();
        solver.add_binary(lit![7], !lit![8]).unwrap();
        solver.add_binary(lit![8], !lit![9]).unwrap();

        solver.set_terminator(|| ControlSignal::Terminate);

        let ret = solver.solve();

        match ret {
            Err(e) => panic!("got error when solving: {}", e),
            Ok(res) => assert_eq!(res, SolverResult::Interrupted),
        }

        // Note: since IPASIR doesn't specify _when_ the terminator callback needs
        // to be called, there is no guarantee that the callback is actually
        // called during solving. This might cause this test to fail with some solvers.
    }

    #[test]
    fn learner_callback() {
        let mut solver = IpasirSolver::new();
        solver.add_binary(lit![0], !lit![1]).unwrap();
        solver.add_binary(lit![1], !lit![2]).unwrap();
        solver.add_binary(lit![2], !lit![3]).unwrap();
        solver.add_binary(lit![3], !lit![4]).unwrap();
        solver.add_binary(lit![4], !lit![5]).unwrap();
        solver.add_binary(lit![5], !lit![6]).unwrap();
        solver.add_binary(lit![6], !lit![7]).unwrap();
        solver.add_binary(lit![7], !lit![8]).unwrap();
        solver.add_binary(lit![8], !lit![9]).unwrap();
        solver.add_unit(lit![9]).unwrap();
        solver.add_unit(!lit![0]).unwrap();

        let mut cl_len = 0;
        let ret;

        solver.set_learner(
            |clause| {
                cl_len = clause.len();
            },
            10,
        );

        ret = solver.solve();

        drop(solver);

        // Note: it is hard to create a testing instance on which clause learning
        // actually happens and therefore it is not actually tested if the
        // callback is called

        match ret {
            Err(e) => panic!("got error when solving: {}", e),
            Ok(res) => assert_eq!(res, SolverResult::UNSAT),
        }
    }
}

mod ffi {
    use crate::solvers::{ControlSignal, LearnCallback, TermCallback};
    use crate::types::Lit;
    use core::ffi::{c_char, c_int, c_void};
    use std::slice;

    #[repr(C)]
    pub struct IpasirHandle {
        _private: [u8; 0],
    }

    extern "C" {
        // Redefinitions of IPASIR functions
        pub fn ipasir_signature() -> *const c_char;
        pub fn ipasir_init() -> *mut IpasirHandle;
        pub fn ipasir_release(solver: *mut IpasirHandle);
        pub fn ipasir_add(solver: *mut IpasirHandle, lit_or_zero: c_int);
        pub fn ipasir_assume(solver: *mut IpasirHandle, lit: c_int);
        pub fn ipasir_solve(solver: *mut IpasirHandle) -> c_int;
        pub fn ipasir_val(solver: *mut IpasirHandle, lit: c_int) -> c_int;
        pub fn ipasir_failed(solver: *mut IpasirHandle, lit: c_int) -> c_int;
        pub fn ipasir_set_terminate(
            solver: *mut IpasirHandle,
            state: *const c_void,
            terminate: extern "C" fn(state: *const c_void) -> c_int,
        );
        pub fn ipasir_set_learn(
            solver: *mut IpasirHandle,
            state: *const c_void,
            max_length: c_int,
            learn: extern "C" fn(state: *const c_void, clause: *const c_int),
        );
    }

    // Raw callbacks forwarding to user callbacks
    pub extern "C" fn ipasir_terminate_cb(ptr: *const c_void) -> c_int {
        let cb = unsafe { &mut *(ptr as *mut TermCallback<'_>) };
        match cb() {
            ControlSignal::Continue => 0,
            ControlSignal::Terminate => 1,
        }
    }

    pub extern "C" fn ipasir_learn_cb(ptr: *const c_void, clause: *const c_int) {
        let cb = unsafe { &mut *(ptr as *mut LearnCallback<'_>) };

        let mut cnt = 0;
        for n in 0.. {
            if unsafe { *clause.offset(n) } != 0 {
                cnt += 1;
            }
        }
        let int_slice = unsafe { slice::from_raw_parts(clause, cnt) };
        let clause: Vec<Lit> = int_slice
            .iter()
            .map(|il| {
                Lit::from_ipasir(*il).expect("Invalid literal in learned clause from IPASIR solver")
            })
            .collect();
        cb(clause)
    }
}
