//! # Foreign Function Interface for CaDiCaL

#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]

use core::ffi::{c_int, c_void};
use std::slice;

use rustsat::{solvers::ControlSignal, types::Lit};

use super::{LearnCallbackPtr, TermCallbackPtr};

include!(concat!(env!("OUT_DIR"), "/bindings.rs"));

// Raw callbacks forwarding to user callbacks
pub extern "C" fn rustsat_ccadical_terminate_cb(ptr: *mut c_void) -> c_int {
    let cb = unsafe { &mut *ptr.cast::<TermCallbackPtr<'_>>() };
    match cb() {
        ControlSignal::Continue => 0,
        ControlSignal::Terminate => 1,
    }
}

pub extern "C" fn rustsat_ccadical_learn_cb(ptr: *mut c_void, clause: *mut c_int) {
    let cb = unsafe { &mut *ptr.cast::<LearnCallbackPtr<'_>>() };

    let mut cnt = 0;
    for n in 0.. {
        if unsafe { *clause.offset(n) } != 0 {
            cnt += 1;
        }
    }
    let int_slice = unsafe { slice::from_raw_parts(clause, cnt) };
    let clause = int_slice
        .iter()
        .map(|il| Lit::from_ipasir(*il).expect("Invalid literal in learned clause from CaDiCaL"))
        .collect();
    cb(clause);
}

pub extern "C" fn rustsat_cadical_collect_lits(vec: *mut c_void, lit: c_int) {
    let vec = vec.cast::<Vec<Lit>>();
    let lit = Lit::from_ipasir(lit).expect("got invalid IPASIR lit from CaDiCaL");
    unsafe { (*vec).push(lit) };
}

// >= 1.6.0
#[cfg(all(
    not(feature = "v1-5-0"),
    not(feature = "v1-5-1"),
    not(feature = "v1-5-2"),
    not(feature = "v1-5-3"),
    not(feature = "v1-5-4"),
    not(feature = "v1-5-5"),
    not(feature = "v1-5-6"),
))]
pub mod ipasirup {
    use std::os::raw::{c_int, c_void};

    use rustsat::types::{Assignment, Clause, Lit};

    use crate::ExternalPropagate;

    pub struct Data {
        pub prop: *mut dyn ExternalPropagate,
        reason_buffer: Option<Clause>,
        external_buffer: Option<Clause>,
    }

    impl Data {
        pub fn new(prop: *mut dyn ExternalPropagate) -> Self {
            Self {
                prop,
                reason_buffer: None,
                external_buffer: None,
            }
        }
    }

    pub const DISPATCH_CALLBACKS: super::CCaDiCaLExternalPropagatorCallbacks =
        super::CCaDiCaLExternalPropagatorCallbacks {
            notify_assignment: Some(rustsat_cadical_notify_assignment),
            notify_new_decision_level: Some(rustsat_cadical_notify_new_decision_level),
            notify_backtrack: Some(rustsat_cadical_notify_backtrack),
            cb_check_found_model: Some(rustsat_cadical_cb_check_found_model),
            cb_decide: Some(rustsat_cadical_cb_decide),
            cb_propagate: Some(rustsat_cadical_cb_propagate),
            cb_add_reason_clause_lit: Some(rustsat_cadical_cb_add_reason_clause_lit),
            cb_has_external_clause: Some(rustsat_cadical_cb_has_external_clause),
            cb_add_external_clause_lit: Some(rustsat_cadical_cb_add_external_clause_lit),
        };

    extern "C" fn rustsat_cadical_notify_assignment(
        data: *mut c_void,
        lit: c_int,
        is_fixed: c_int,
    ) {
        unsafe {
            (*(*data.cast::<Data>()).prop).assignment(
                Lit::from_ipasir(lit).expect("external propagator got invalid lit from CaDiCaL"),
                is_fixed != 0,
            )
        };
    }

    extern "C" fn rustsat_cadical_notify_new_decision_level(data: *mut c_void) {
        unsafe { (*(*data.cast::<Data>()).prop).new_decision_level() }
    }

    extern "C" fn rustsat_cadical_notify_backtrack(data: *mut c_void, new_level: usize) {
        unsafe { (*(*data.cast::<Data>()).prop).backtrack(new_level) }
    }

    extern "C" fn rustsat_cadical_cb_check_found_model(
        data: *mut c_void,
        model: *const c_int,
        model_len: usize,
    ) -> c_int {
        let model = unsafe { std::slice::from_raw_parts(model, model_len) };
        let sol: Assignment = model
            .iter()
            .map(|&l| {
                Lit::from_ipasir(l).expect("external propagator got invalid lit from CaDiCaL")
            })
            .collect();
        c_int::from(unsafe { (*(*data.cast::<Data>()).prop).check_found_solution(&sol) })
    }

    extern "C" fn rustsat_cadical_cb_decide(data: *mut c_void) -> c_int {
        unsafe { (*(*data.cast::<Data>()).prop).decide() }
            .map(Lit::to_ipasir)
            .unwrap_or(0)
    }

    extern "C" fn rustsat_cadical_cb_propagate(data: *mut c_void) -> c_int {
        unsafe { (*(*data.cast::<Data>()).prop).propagate() }
            .map(Lit::to_ipasir)
            .unwrap_or(0)
    }

    extern "C" fn rustsat_cadical_cb_add_reason_clause_lit(
        data: *mut c_void,
        propagated_lit: c_int,
    ) -> c_int {
        todo!()
    }

    extern "C" fn rustsat_cadical_cb_has_external_clause(data: *mut c_void) -> c_int {
        todo!()
    }

    extern "C" fn rustsat_cadical_cb_add_external_clause_lit(data: *mut c_void) -> c_int {
        todo!()
    }
}
