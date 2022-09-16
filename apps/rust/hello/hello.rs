/*
 * Copyright 2021, Google LLC
 *
 * SPDX-License-Identifier: Apache-2.0
 */
#![no_std]
#![no_main]

extern crate libkata;
use kata_os_common::logger::KataLogger;
use kata_sdk_interface::*;
use log::info;

// Message output is sent through the kata-os-logger which calls logger_log
// to deliver data to the console. Redict to the sdk.
#[no_mangle]
#[allow(unused_variables)]
pub fn logger_log(_level: u8, msg: *const cstr_core::c_char) {
    if let Ok(str) = unsafe { cstr_core::CStr::from_ptr(msg) }.to_str() {
        let _ = kata_sdk_log(str);
    }
}

#[no_mangle]
pub fn main() {
    // Setup logger; (XXX maybe belongs in the SDKRuntime)
    static KATA_LOGGER: KataLogger = KataLogger;
    log::set_logger(&KATA_LOGGER).unwrap();
    log::set_max_level(log::LevelFilter::Trace);

    match kata_sdk_ping() {
        Ok(_) => info!("ping!"),
        Err(e) => info!("kata_sdk_ping failed: {:?}", e),
    }
    info!("I am a Rust app, hear me log!");
    info!("Done, wimper ...");
}
