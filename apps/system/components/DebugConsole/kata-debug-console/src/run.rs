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

use core::slice;
use kata_io;
use kata_os_common::allocator;
use kata_os_common::logger::KataLogger;
use kata_os_common::sel4_sys;
use kata_os_common::slot_allocator;
use kata_shell;
use kata_uart_client;
use log::trace;

use sel4_sys::seL4_CPtr;

use slot_allocator::KATA_CSPACE_SLOTS;

extern "C" {
    static SELF_CNODE_FIRST_SLOT: seL4_CPtr;
    static SELF_CNODE_LAST_SLOT: seL4_CPtr;

    static cpio_archive: *const u8; // CPIO archive of built-in files
}

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
        allocator::ALLOCATOR.init(HEAP_MEMORY.as_mut_ptr() as usize, HEAP_MEMORY.len());
        trace!(
            "setup heap: start_addr {:p} size {}",
            HEAP_MEMORY.as_ptr(),
            HEAP_MEMORY.len()
        );
    }

    unsafe {
        KATA_CSPACE_SLOTS.init(
            /*first_slot=*/ SELF_CNODE_FIRST_SLOT,
            /*size=*/ SELF_CNODE_LAST_SLOT - SELF_CNODE_FIRST_SLOT
        );
        trace!("setup cspace slots: first slot {} free {}",
               KATA_CSPACE_SLOTS.base_slot(),
               KATA_CSPACE_SLOTS.free_slots());
    }
}

/// Entry point for DebugConsole. Runs the shell with UART IO.
#[no_mangle]
pub extern "C" fn run() -> ! {
    let mut tx = kata_uart_client::Tx::new();
    let mut rx = kata_io::BufReader::new(kata_uart_client::Rx::new());
    let cpio_archive_ref = unsafe {
        // XXX want begin-end or begin+size instead of a fixed-size block
        slice::from_raw_parts(cpio_archive, 16777216)
    };
    kata_shell::repl(&mut tx, &mut rx, cpio_archive_ref);
}
