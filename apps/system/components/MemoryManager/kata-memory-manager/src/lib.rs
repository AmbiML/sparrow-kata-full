//! Kata OS global memory management support

#![cfg_attr(not(test), no_std)]

use core::ops::Range;
use kata_memory_interface::ObjDescBundle;
use kata_memory_interface::MemoryError;
use kata_memory_interface::MemoryManagerInterface;
use kata_memory_interface::MemoryManagerStats;
use kata_os_common::sel4_sys;
use sel4_sys::seL4_CPtr;
use sel4_sys::seL4_UntypedDesc;
use spin::Mutex;

mod memory_manager;
pub use memory_manager::MemoryManager;

// KataMemoryManager bundles an instance of the MemoryManager that operates
// on KataOS interfaces and synchronizes public use with a Mutex. There is
// a two-step dance to setup an instance because we want KATA_MEMORY static
// and MemoryManager is incapable of supplying a const fn due it's use of
// hashbrown::HashMap.
pub struct KataMemoryManager {
    manager: Mutex<Option<MemoryManager>>,
}
impl KataMemoryManager {
    // Constructs a partially-initialized instance; to complete call init().
    pub const fn empty() -> KataMemoryManager {
        KataMemoryManager {
            manager: Mutex::new(None),
        }
    }

    // Finishes the setup started by empty():
    pub fn init(&self, ut_slots: Range<seL4_CPtr>, untypeds: &[seL4_UntypedDesc]) {
        *self.manager.lock() = Some(MemoryManager::new(ut_slots, untypeds));
    }
}
// These just lock accesses and handle the necessary indirection.
impl MemoryManagerInterface for KataMemoryManager {
    fn alloc(&mut self, objs: &ObjDescBundle) -> Result<(), MemoryError> {
        self.manager.lock().as_mut().unwrap().alloc(objs)
    }
    fn free(&mut self, objs: &ObjDescBundle) -> Result<(), MemoryError> {
        self.manager.lock().as_mut().unwrap().free(objs)
    }
    fn stats(&self) -> Result<MemoryManagerStats, MemoryError> {
        self.manager.lock().as_ref().unwrap().stats()
    }
}
