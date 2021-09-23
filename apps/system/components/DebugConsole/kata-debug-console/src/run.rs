//! Kata OS command line interface

// This brief bootstrap of Rust-in-Kata prototypes a minimal modular design
// for the DebugConsole CLI use case.
//
// * kata_io Read/Write interface (or move to std::, but that requires alloc)
// * kata_uart_client implementation of the kata_io interface
// * kata_line_reader
// * kata_shell
// * kata_debug_console main entry point fn run()

#![no_std]

#[cfg(not(test))]
extern crate kata_panic;

use kata_allocator;
use kata_logger::KataLogger;
use kata_shell;
use kata_uart_client;
use log::trace;

#[no_mangle]
pub extern "C" fn pre_init() {
    static KATA_LOGGER: KataLogger = KataLogger;
    log::set_logger(&KATA_LOGGER).unwrap();
    // NB: set to Trace for early-boot msgs
    log::set_max_level(log::LevelFilter::Debug);

    // TODO(b/200946906): Review per-component heap allocations, including this one.
    const HEAP_SIZE: usize = 1 << 20;
    static mut HEAP_MEMORY: [u8; HEAP_SIZE] = [0; HEAP_SIZE];
    unsafe {
        kata_allocator::ALLOCATOR.init(HEAP_MEMORY.as_mut_ptr() as usize, HEAP_MEMORY.len());
        trace!(
            "setup heap: start_addr {:p} size {}",
            HEAP_MEMORY.as_ptr(),
            HEAP_MEMORY.len()
        );
    }
}

/// Entry point for DebugConsole. Runs the shell with UART IO.
#[no_mangle]
pub extern "C" fn run() -> ! {
    trace!("run");
    let mut tx = kata_uart_client::Tx {};
    let mut rx = kata_uart_client::Rx {};
    kata_shell::repl(&mut tx, &mut rx);
}
