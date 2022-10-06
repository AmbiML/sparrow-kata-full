#![no_std]

// kata-vec-core is the vector core driver. It is responsible for providing
// convenient methods for interacting with the hardware.

mod vc_top;

use core::assert;
use core::slice;
use kata_ml_interface::MlCoreInterface;
use xmas_elf::program::{SegmentData, Type};
use xmas_elf::ElfFile;

// TODO(jesionowski): Move these constants to an auto-generated file.
const ITCM_SIZE: usize = 0x40000;
const ITCM_PADDR: usize = 0x30000000;
const DTCM_SIZE: usize = 0x1000000;
const DTCM_PADDR: usize = 0x34000000;

// TODO(jesionowski): ITCM / DTCM will eventually be merged into a single memory.
extern "C" {
    static itcm: *mut u32;
}
extern "C" {
    static dtcm: *mut u32;
}

fn get_dtcm_slice() -> &'static mut [u32] {
    unsafe { slice::from_raw_parts_mut(dtcm, DTCM_SIZE / 4) }
}

pub struct MlCore {}

impl MlCoreInterface for MlCore {
    fn enable_interrupts(&mut self, enable: bool) {
        let intr_enable = vc_top::IntrEnable::new()
            .with_host_req(enable)
            .with_finish(enable)
            .with_instruction_fault(enable)
            .with_data_fault(enable);
        vc_top::set_intr_enable(intr_enable);
    }

    // TODO(jesionowski): Implement using hardware clear CSRs.
    fn clear_tcm(&mut self, _start: *const u32, _len: usize) {}

    fn run(&mut self) {
        let ctrl = vc_top::Ctrl::new()
            .with_freeze(false)
            .with_vc_reset(false)
            .with_pc_start(0);
        vc_top::set_ctrl(ctrl);
    }

    fn load_elf(&mut self, elf_slice: &[u8]) -> Result<(), &'static str> {
        let itcm_slice = unsafe { slice::from_raw_parts_mut(itcm as *mut u8, ITCM_SIZE) };
        let dtcm_slice = unsafe { slice::from_raw_parts_mut(dtcm as *mut u8, DTCM_SIZE) };

        let elf = ElfFile::new(elf_slice)?;

        for seg in elf.program_iter() {
            if seg.get_type()? == Type::Load {
                let fsize = seg.file_size() as usize;
                let msize = seg.mem_size() as usize;

                if seg.virtual_addr() as usize == ITCM_PADDR {
                    assert!(
                        fsize <= ITCM_SIZE,
                        "Elf's ITCM section is larger than than ITCM_SIZE"
                    );

                    // Due to being Load types we are guarunteed SegmentData::Undefined as the
                    // data type.
                    if let SegmentData::Undefined(bytes) = seg.get_data(&elf)? {
                        itcm_slice[..fsize].copy_from_slice(&bytes);
                    }
                } else if seg.virtual_addr() as usize == DTCM_PADDR {
                    assert!(
                        msize <= DTCM_SIZE,
                        "Elf's DTCM section is larger than than DTCM_SIZE"
                    );

                    if let SegmentData::Undefined(bytes) = seg.get_data(&elf)? {
                        dtcm_slice[..fsize].copy_from_slice(&bytes);
                    }
                    // TODO(jesionowski): Use clear_tcm instead.
                    // Clear NOBITS sections.
                    dtcm_slice[fsize..msize].fill(0x00);
                } else {
                    assert!(false, "Elf contains LOAD section outside TCM");
                }
            }
        }

        Ok(())
    }

    // TODO(jesionowski): Read these from CSRs when available.
    fn get_return_code() -> u32 {
        const RC_OFFSET: usize = 0x3FFFEE;
        get_dtcm_slice()[RC_OFFSET]
    }

    fn get_fault_register() -> u32 {
        const FAULT_OFFSET: usize = 0x3FFFEF;
        get_dtcm_slice()[FAULT_OFFSET]
    }

    // Interrupts are write 1 to clear.
    fn clear_host_req() {
        let mut intr_state = vc_top::get_intr_state();
        intr_state.set_host_req(true);
        vc_top::set_intr_state(intr_state);
    }

    fn clear_finish() {
        let mut intr_state = vc_top::get_intr_state();
        intr_state.set_finish(true);
        vc_top::set_intr_state(intr_state);
    }

    fn clear_instruction_fault() {
        let mut intr_state = vc_top::get_intr_state();
        intr_state.set_instruction_fault(true);
        vc_top::set_intr_state(intr_state);
    }

    fn clear_data_fault() {
        let mut intr_state = vc_top::get_intr_state();
        intr_state.set_data_fault(true);
        vc_top::set_intr_state(intr_state);
    }
}
