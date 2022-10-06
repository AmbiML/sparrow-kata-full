//! An allocator for an array of integer-numbered objects. This is
//! typically used to track capability slots in a CNode, though it
//! can be used for other purposes.

#![cfg_attr(not(test), no_std)]
#![allow(non_snake_case)]

use bitvec::prelude::*;
use core::ops::Range;
use spin::Mutex;

mod one_slot;
pub use one_slot::CSpaceSlot;

struct Slots {
    bits: Option<BitBox<Lsb0, u8>>,
    used: usize,
    // TODO(sleffler): maybe track last alloc for O(1) sequential allocations
}
impl Slots {
    fn new(size: usize) -> Self {
        Slots {
            bits: Some(bitvec![Lsb0, u8; 0; size].into_boxed_bitslice()),
            used: 0,
        }
    }
    const fn empty() -> Self {
        Slots {
            bits: None,
            used: 0,
        }
    }
    fn init(&mut self, size: usize) {
        self.bits = Some(bitvec![Lsb0, u8; 0; size].into_boxed_bitslice());
    }
    fn used_slots(&self) -> usize { self.used }
    fn free_slots(&self) -> usize {
        let bits = self.bits.as_ref().unwrap();
        bits.len() - self.used
    }

    fn not_any_in_range(&self, range: Range<usize>) -> bool {
        let bslice = self.bits.as_ref().unwrap().as_bitslice();
        // NB: check there is a valid slice
        if range.start < bslice.len() && range.end <= bslice.len() {
            bslice[range].not_any()
        } else {
            false
        }
    }
    fn set_range(&mut self, range: Range<usize>, value: bool) {
        let bits = self.bits.as_mut().unwrap();
        let count = range.len();
        let bslice = &mut bits.as_mut_bitslice()[range];
        if value {
            assert!(bslice.not_any());
            bslice.set_all(true);
            self.used = self.used + count;
        } else {
            assert!(bslice.all());
            bslice.set_all(false);
            self.used = self.used - count;
        }
    }
    fn alloc_first_fit(&mut self, count: usize) -> Option<usize> {
        if count == 0 { return None; }
        if count > self.free_slots() { return None; }
        if count == 1  {
            let bits = self.bits.as_mut().unwrap();
            let bit = bits.first_zero()?;
            unsafe { bits.set_unchecked(bit, true) };
            self.used = self.used + 1;
            Some(bit)
        } else {
            let first_slot = self.bits.as_ref().unwrap().iter_zeros()
                .find(|bit| self.not_any_in_range(bit + 1..bit + count))?;
            self.set_range(first_slot..first_slot + count, true);
            Some(first_slot)
        }
    }

    fn free(&mut self, first_slot: usize, count: usize) {
        assert!(count <= self.used);
        self.set_range(first_slot..first_slot + count, false);
    }
}

pub struct KataSlotAllocator {
    slots: Mutex<Slots>,
    base_slot: usize,
}

#[cfg(not(test))]
pub static mut KATA_CSPACE_SLOTS: KataSlotAllocator = KataSlotAllocator::empty();

impl KataSlotAllocator {
    /// Initializes the Slot state
    pub fn new(first_slot: usize, size: usize) -> Self {
        KataSlotAllocator {
            slots: Mutex::new(Slots::new(size)),
            base_slot: first_slot,
        }
    }

    /// Create a new UNINITIALIZED slot allocator. You must initialize this
    /// using the init method before using the allocator.
    pub const fn empty() -> Self {
        KataSlotAllocator {
            slots: Mutex::new(Slots::empty()),
            base_slot: 0,
        }
    }

    /// Initializes the Slot state
    pub unsafe fn init(&mut self, first_slot: usize, size: usize) {
        self.base_slot = first_slot;
        (*self.slots.lock()).init(size);
    }

    /// Returns the base slot number.
    pub fn base_slot(&self) -> usize { self.base_slot }

    /// Returns the number of slots in use.
    pub fn used_slots(&self) -> usize {
        (*self.slots.lock()).used_slots()
    }

    /// Returns the number of slots available.
    pub fn free_slots(&self) -> usize {
        (*self.slots.lock()).free_slots()
    }

    pub fn alloc(&self, count: usize) -> Option<usize> {
        (*self.slots.lock()).alloc_first_fit(count).map(|x| x + self.base_slot)
    }

    pub fn free(&self, first_slot: usize, count: usize) {
        assert!(first_slot >= self.base_slot);
        (*self.slots.lock()).free(first_slot - self.base_slot, count)
    }
}

#[cfg(test)]
mod slot_tests {
    use super::*;

    const NSLOTS: usize = 64;

    #[test]
    fn test_slots_new() {
        let slots = Slots::new(NSLOTS);
        assert_eq!(slots.used_slots(), 0);
        assert_eq!(slots.free_slots(), NSLOTS);
    }

    #[test]
    fn test_slots_init() {
        static mut SLOTS: Slots = Slots::empty();
        unsafe {
            SLOTS.init(NSLOTS);
            assert_eq!(SLOTS.used_slots(), 0);
            assert_eq!(SLOTS.free_slots(), NSLOTS);
        }
    }

    #[test]
    fn test_slots_one() {
        let mut slots = Slots::new(64);
        assert_eq!(slots.used_slots(), 0);
        assert_eq!(slots.free_slots(), NSLOTS);
        let first = slots.alloc_first_fit(1).unwrap();
        assert_eq!(slots.used_slots(), 1);
        assert_eq!(slots.free_slots(), NSLOTS - 1);
        slots.free(first, 1);
        assert_eq!(slots.used_slots(), 0);
        assert_eq!(slots.free_slots(), NSLOTS);
    }

    #[test]
    fn test_slots_one_multi() {
        let mut slots = Slots::new(64);
        assert_eq!(slots.used_slots(), 0);
        assert_eq!(slots.free_slots(), NSLOTS);
        let first = slots.alloc_first_fit(1).unwrap();
        let second = slots.alloc_first_fit(1).unwrap();
        assert_eq!(slots.used_slots(), 2);
        assert_eq!(slots.free_slots(), NSLOTS - 2);
        slots.free(first, 1);
        assert_eq!(slots.used_slots(), 1);
        assert_eq!(slots.free_slots(), NSLOTS - 1);
        slots.free(second, 1);
        assert_eq!(slots.used_slots(), 0);
        assert_eq!(slots.free_slots(), NSLOTS );

        let again = slots.alloc_first_fit(1).unwrap();
        // first-fit so should get same thing
        assert_eq!(first, again);
    }

    #[test]
    fn test_slots_range() {
        let mut slots = Slots::new(64);
        assert_eq!(slots.used_slots(), 0);
        assert_eq!(slots.free_slots(), NSLOTS);
        let first = slots.alloc_first_fit(3).unwrap();
        assert_eq!(slots.used_slots(), 3);
        assert_eq!(slots.free_slots(), NSLOTS - 3);
        slots.free(first, 3);
        assert_eq!(slots.used_slots(), 0);
        assert_eq!(slots.free_slots(), NSLOTS);

        let again = slots.alloc_first_fit(3).unwrap();
        assert_eq!(first, again);
    }

    #[test]
    fn test_slots_range_multi() {
        let mut slots = Slots::new(64);
        assert_eq!(slots.used_slots(), 0);
        assert_eq!(slots.free_slots(), NSLOTS);
        let first = slots.alloc_first_fit(4).unwrap();
        let second = slots.alloc_first_fit(4).unwrap();
        assert_eq!(slots.used_slots(), 4 + 4);
        assert_eq!(slots.free_slots(), NSLOTS - (4 + 4));
        slots.free(first, 4);
        assert_eq!(slots.used_slots(), 4);
        assert_eq!(slots.free_slots(), NSLOTS - 4);
        slots.free(second, 4);
        assert_eq!(slots.used_slots(), 0);
        assert_eq!(slots.free_slots(), NSLOTS);

        let again = slots.alloc_first_fit(4).unwrap();
        assert_eq!(first, again);
    }

    #[test]
    fn test_slots_range_split_free() {
        let mut slots = Slots::new(64);
        assert_eq!(slots.used_slots(), 0);
        assert_eq!(slots.free_slots(), NSLOTS);
        let first = slots.alloc_first_fit(4).unwrap();
        assert_eq!(slots.used_slots(), 4);
        assert_eq!(slots.free_slots(), NSLOTS - 4);
        slots.free(first, 2);
        assert_eq!(slots.used_slots(), 2);
        assert_eq!(slots.free_slots(), NSLOTS - 2);
        slots.free(first + 2, 2);
        assert_eq!(slots.used_slots(), 0);
        assert_eq!(slots.free_slots(), NSLOTS);
    }

    #[test]
    fn test_slots_range_split_free_multi() {
        let mut slots = Slots::new(64);
        assert_eq!(slots.used_slots(), 0);
        assert_eq!(slots.free_slots(), NSLOTS);

        // Request 4 slots
        let first = slots.alloc_first_fit(4).unwrap();
        assert_eq!(slots.used_slots(), 4);
        assert_eq!(slots.free_slots(), NSLOTS - 4);
        // Free the first 2 slots to create a hole
        slots.free(first, 2);
        assert_eq!(slots.used_slots(), 2);
        assert_eq!(slots.free_slots(), NSLOTS - 2);

        // Requet another 4 slots
        let second = slots.alloc_first_fit(4).unwrap();
        // The hole is 2-large so our request should go after
        assert_eq!(first + 4, second);
        slots.free(first + 2, 2);
        assert_eq!(slots.used_slots(), 4);
        assert_eq!(slots.free_slots(), NSLOTS - 4);
    }

    #[test]
    fn test_slots_empty() {
        let mut slots = Slots::new(0);
        assert_eq!(slots.used_slots(), 0);
        assert_eq!(slots.free_slots(), 0);
        assert!(slots.alloc_first_fit(1).is_none());
        assert!(slots.alloc_first_fit(8).is_none());

    }

    #[test]
    fn test_slots_too_small() {
        let mut slots = Slots::new(NSLOTS);
        assert_eq!(slots.used_slots(), 0);
        assert_eq!(slots.free_slots(), NSLOTS);
        assert!(slots.alloc_first_fit(NSLOTS + 1).is_none());
    }

    #[test]
    fn test_slots_oospace() {
        let mut slots = Slots::new(4);
        // Allocate 3 of 4 slots
        let first = slots.alloc_first_fit(3).unwrap();
        assert_eq!(slots.free_slots(), 1);
        // Request 2 slots, not enough space
        assert!(slots.alloc_first_fit(2).is_none());

        // Free up [0,1] so free is [0, 1, 3]
        slots.free(first, 2);
        // Request 3 which does not fit.
        assert!(slots.alloc_first_fit(3).is_none());
    }

    #[test]
    #[should_panic]
    fn test_slots_free_single_invalid() {
        let mut slots = Slots::new(NSLOTS);
        assert_eq!(slots.used_slots(), 0);
        assert_eq!(slots.free_slots(), NSLOTS);
        // Free an already free slot
        slots.free(0, 1);
    }

    #[test]
    #[should_panic]
    fn test_slots_free_hole_invalid() {
        let mut slots = Slots::new(NSLOTS);
        assert_eq!(slots.used_slots(), 0);
        assert_eq!(slots.free_slots(), NSLOTS);
        // Allocate the first 4 slots
        let first = slots.alloc_first_fit(4).unwrap();
        // Free slot 2, should work.
        slots.free(first + 2, 1);
        // Try to free a range that includes a previously free'd slot
        slots.free(first, 4);
    }
}

#[cfg(test)]
mod kata_slot_tests {
    use super::*;

    // NB: all these tests will run concurrently to exercise locking

    const SLOT_RANGE: Range<usize> = Range { start: 10, end: 10 + 64 };
    static mut SLOTS: KataSlotAllocator = KataSlotAllocator::empty();
    fn setup() {
        use std::sync::Once;
        static INIT: Once = Once::new();
        INIT.call_once(|| { unsafe { SLOTS.init(SLOT_RANGE.start, SLOT_RANGE.len()) }; })
    }

    #[test]
    fn test_slots_one() {
        setup();
        unsafe {
            let first = SLOTS.alloc(1).unwrap();
            assert!(SLOT_RANGE.contains(&first));
            SLOTS.free(first, 1);
        }
    }

    #[test]
    fn test_slots_one_multi() {
        setup();
        unsafe {
            let first = SLOTS.alloc(1).unwrap();
            assert!(SLOT_RANGE.contains(&first));
            let second = SLOTS.alloc(1).unwrap();
            assert!(SLOT_RANGE.contains(&second));
            SLOTS.free(first, 1);
            SLOTS.free(second, 1);
        }
    }

    #[test]
    fn test_slots_range() {
        setup();
        unsafe {
            let first = SLOTS.alloc(3).unwrap();
            assert!(SLOT_RANGE.contains(&first));
            assert!(SLOT_RANGE.contains(&(first + 1)));
            assert!(SLOT_RANGE.contains(&(first + 2)));
            SLOTS.free(first, 3);
        }
    }

    #[test]
    fn test_slots_range_multi() {
        setup();
        unsafe {
            let first = SLOTS.alloc(4).unwrap();
            let second = SLOTS.alloc(4).unwrap();
            SLOTS.free(first, 4);
            SLOTS.free(second, 4);
        }
    }

    #[test]
    fn test_slots_range_split_free() {
        setup();
        unsafe {
            let first = SLOTS.alloc(4).unwrap();
            SLOTS.free(first, 2);
            SLOTS.free(first + 2, 2);
        }
    }

    #[test]
    fn test_slots_range_split_free_multi() {
        setup();
        unsafe {
            // Request 4 slots
            let first = SLOTS.alloc(4).unwrap();
            // Free the first 2 slots to create a hole
            SLOTS.free(first, 2);

            // Requet another 4 slots
            let _second = SLOTS.alloc(4).unwrap();
            // The hole is 2-large so our request should go after
            SLOTS.free(first + 2, 2);
        }
    }

    #[test]
    fn test_slots_oospace() {
        setup();
        unsafe {
            assert!(SLOTS.alloc(SLOT_RANGE.len() + 1).is_none());
        }
    }
}
