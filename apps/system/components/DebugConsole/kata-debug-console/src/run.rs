//! Kata OS command line interface

// This brief bootstrap of Rust-in-Kata prototypes a minimal modular design
// for the DebugConsole CLI use case.
//
// * kata_io Read/Write interface (or move to std::, but that requires alloc)
// * kata_uart_client implementation of the kata_io interface
// * kata_line_reader
// * kata_shell
// * kata_debug_console main entry point fn run()

// std:: requires at least an allocator, which Kata does not have yet. For now
// the CLI will be implemented with only core::.
#![no_std]

extern crate panic_halt;

use kata_allocator;
use kata_shell;
use kata_uart_client;

#[no_mangle]
// NB: use post_init insted of pre_init so syslog interface is setup
pub extern "C" fn post_init() {
    // TODO(sleffler): temp until we integrate with seL4
    static mut HEAP_MEMORY: [u8; 16 * 1024] = [0; 16 * 1024];
    unsafe {
        kata_allocator::ALLOCATOR.init(HEAP_MEMORY.as_mut_ptr() as usize, HEAP_MEMORY.len());
    }
}

/// Entry point for DebugConsole. Runs the shell with UART IO.
#[no_mangle]
pub extern "C" fn run() -> ! {
    let mut tx = kata_uart_client::Tx {};
    let mut rx = kata_uart_client::Rx {};
    kata_shell::repl(&mut tx, &mut rx);
}
