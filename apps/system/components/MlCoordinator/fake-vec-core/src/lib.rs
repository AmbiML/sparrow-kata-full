#![no_std]

// fake-vec-core is a stubbed out version of kata-vec-core.
extern crate alloc;

use alloc::boxed::Box;
use kata_io::Read;
use kata_ml_shared::Permission;

pub fn enable_interrupts(_enable: bool) {}

pub fn set_wmmu_window(
    _window_id: usize,
    _start_address: usize,
    _length: usize,
    _permission: Permission,
) {
}

pub fn run() {}

pub fn write_image_part(
    _image: &mut Box<dyn Read>,
    _start_address: usize,
    _on_flash_size: usize,
    _unpacked_size: usize,
) -> Result<(), &'static str> {
    Ok(())
}

pub fn tcm_move(_src_offset: usize, _dest_offset: usize, _byte_length: usize) {}

pub fn clear_host_req() {}

pub fn clear_finish() {}

pub fn clear_instruction_fault() {}

pub fn clear_data_fault() {}

pub fn clear_tcm(_addr: usize, _len: usize) {}

pub fn wait_for_clear_to_finish() {}

pub fn get_return_code() -> u32 { 0 }

pub fn get_fault_register() -> u32 { 0 }
