//! The Timer Service provides multiplexed access to a hardware timer.
#![no_std]
#![allow(clippy::missing_safety_doc)]

use core::time::Duration;
use kata_os_common::allocator;
use kata_os_common::logger::KataLogger;
use kata_os_common::sel4_sys::seL4_Word;
use kata_timer_interface::{TimerId, TimerServiceError};
use kata_timer_service::TIMER_SRV;
use log::trace;

#[no_mangle]
pub unsafe extern "C" fn pre_init() {
    static KATA_LOGGER: KataLogger = KataLogger;
    log::set_logger(&KATA_LOGGER).unwrap();
    log::set_max_level(log::LevelFilter::Trace);

    // TODO(jesionowski): temp until we integrate with seL4
    static mut HEAP_MEMORY: [u8; 4 * 1024] = [0; 4 * 1024];
    allocator::ALLOCATOR.init(HEAP_MEMORY.as_mut_ptr() as usize, HEAP_MEMORY.len());
    trace!(
        "setup heap: start_addr {:p} size {}",
        HEAP_MEMORY.as_ptr(),
        HEAP_MEMORY.len()
    );
}

#[no_mangle]
pub unsafe extern "C" fn timer__init() {
    TIMER_SRV.lock().init();
}

extern "C" {
    fn timer_get_sender_id() -> seL4_Word;
}

#[no_mangle]
pub unsafe extern "C" fn timer_completed_timers() -> u32 {
    let client_id = timer_get_sender_id();
    return TIMER_SRV.lock().completed_timers(client_id);
}

#[no_mangle]
pub unsafe extern "C" fn timer_oneshot(
    timer_id: TimerId,
    duration_ms: u32
) -> TimerServiceError {
    let duration = Duration::from_millis(duration_ms as u64);
    let is_periodic = false;
    let client_id = timer_get_sender_id();
    match TIMER_SRV
        .lock()
        .add(client_id, timer_id, duration, is_periodic)
    {
        Err(e) => e,
        Ok(()) => TimerServiceError::TimerOk,
    }
}

#[no_mangle]
pub unsafe extern "C" fn timer_periodic(
    timer_id: TimerId,
    duration_ms: u32
) -> TimerServiceError {
    let duration = Duration::from_millis(duration_ms as u64);
    let is_periodic = true;
    let client_id = timer_get_sender_id();
    match TIMER_SRV
        .lock()
        .add(client_id, timer_id, duration, is_periodic)
    {
        Err(e) => e,
        Ok(()) => TimerServiceError::TimerOk,
    }
}

#[no_mangle]
pub unsafe extern "C" fn timer_cancel(
    timer_id: TimerId
) -> TimerServiceError {
    let client_id = timer_get_sender_id();
    match TIMER_SRV.lock().cancel(client_id, timer_id) {
        Err(e) => e,
        Ok(()) => TimerServiceError::TimerOk,
    }
}

#[no_mangle]
pub unsafe extern "C" fn timer_interrupt_handle() {
    extern "C" {
        fn timer_interrupt_acknowledge() -> u32;
    }
    TIMER_SRV.lock().service_interrupt();
    assert!(timer_interrupt_acknowledge() == 0);
}
