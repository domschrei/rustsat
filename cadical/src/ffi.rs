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
