//! Kata OS Security Coordinator component support.

// Code here binds the camkes component to the rust code.
#![no_std]

use core::slice;
use kata_allocator;
use kata_logger::KataLogger;
#[cfg(not(test))]
extern crate kata_panic;
use kata_security_common::*;
use kata_security_coordinator::KATA_SECURITY;
use log::trace;

#[no_mangle]
pub extern "C" fn pre_init() {
    static KATA_LOGGER: KataLogger = KataLogger;
    log::set_logger(&KATA_LOGGER).unwrap();
    // NB: set to max; the LoggerInterface will filter
    log::set_max_level(log::LevelFilter::Trace);

    // TODO(sleffler): temp until we integrate with seL4
    // TODO(sleffler): should be used rarely
    static mut HEAP_MEMORY: [u8; 8 * 1024] = [0; 8 * 1024];
    unsafe {
        kata_allocator::ALLOCATOR.init(HEAP_MEMORY.as_mut_ptr() as usize, HEAP_MEMORY.len());
        trace!(
            "setup heap: start_addr {:p} size {}",
            HEAP_MEMORY.as_ptr(),
            HEAP_MEMORY.len()
        );
    }

    // Complete KATA_SECURITY setup. This is as early as we can do it given that
    // it needs the GlobalAllocator.
    unsafe {
        KATA_SECURITY.init();
    }
}

#[no_mangle]
pub extern "C" fn security_request(
    c_request: SecurityRequest,
    c_request_buffer_len: u32,
    c_request_buffer: *const u8,
    c_reply_buffer: *mut SecurityReplyData,
) -> SecurityRequestError {
    unsafe {
        KATA_SECURITY.request(
            c_request,
            slice::from_raw_parts(c_request_buffer, c_request_buffer_len as usize),
            &mut (*c_reply_buffer)[..],
        )
    }
    .map_or_else(|e| e, |_v| SecurityRequestError::SreSuccess)
}
