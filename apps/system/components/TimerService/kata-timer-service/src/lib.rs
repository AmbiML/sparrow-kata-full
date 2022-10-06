// Copyright 2022 Google LLC
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     https://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

#![no_std]
#![feature(map_first_last)]
#![feature(const_btree_new)]

extern crate alloc;

use alloc::collections::BTreeMap;
use core::time::Duration;
use kata_os_common::sel4_sys::seL4_Word;
use kata_timer_interface::{HardwareTimer, Ticks, TimerId, TimerServiceError};
use opentitan_timer::OtTimer;
use spin::Mutex;

// TODO(jesionowski): NUM_CLIENTS should be derived through the static
// camkes configuration. This may take some template hacking as the number
// of clients is generated as a C #define.
const NUM_CLIENTS: usize = 2;

// We use a TimerId as a bit vector denoting completed timers.
const TIMERS_PER_CLIENT: usize = 32;

// An event represents a future timeout and the associated notification clien.
// If the event is periodic, it includes the period.
struct Event {
    client_id: seL4_Word,
    timer_id: TimerId,
    recurring: Option<Duration>,
}

// We keep track of outstanding timers using a BTreeMap from the deadline to
// the associated event.
// Each client may have multiple outstanding timers, which we represent through
// a bit vector in timer_state.
pub struct KataTimerService {
    timer: OtTimer, // TODO(jesionowski): Option<Box<dyn HardwareTimer>> for testing
    events: BTreeMap<Ticks, Event>,
    timer_state: [u32; NUM_CLIENTS], // XXX: bitvec?
}

pub static mut TIMER_SRV: Mutex<KataTimerService> = Mutex::new(KataTimerService {
    timer: OtTimer,
    events: BTreeMap::new(),
    timer_state: [0; NUM_CLIENTS],
});

impl KataTimerService {
    pub fn init(&mut self) { self.timer.setup(); }

    pub fn completed_timers(&mut self, client_id: seL4_Word) -> u32 {
        assert!(0 < client_id && client_id <= NUM_CLIENTS);

        // client_id is 1-indexed by seL4, timer_state is 0-index.
        let client = client_id - 1;
        let state = self.timer_state[client];
        self.timer_state[client] = 0;

        state
    }

    pub fn add(
        &mut self,
        client_id: seL4_Word,
        timer_id: TimerId,
        duration: Duration,
        periodic: bool,
    ) -> Result<(), TimerServiceError> {
        assert!(0 < client_id && client_id <= NUM_CLIENTS);
        assert!(timer_id < TIMERS_PER_CLIENT as u32);

        if self
            .events
            .iter()
            .any(|(_, ev)| ev.client_id == client_id && ev.timer_id == timer_id)
        {
            return Err(TimerServiceError::TimerAlreadyExists);
        }

        let recurring = if periodic { Some(duration) } else { None };
        self.events.insert(
            self.timer.deadline(duration),
            Event {
                client_id,
                timer_id,
                recurring,
            },
        );

        // Next deadline is always on top of the tree.
        if let Some(event) = self.events.first_entry() {
            self.timer.set_alarm(*event.key())
        }

        Ok(())
    }

    pub fn cancel(
        &mut self,
        client_id: seL4_Word,
        timer_id: TimerId,
    ) -> Result<(), TimerServiceError> {
        assert!(0 < client_id && client_id <= NUM_CLIENTS);
        assert!(timer_id < TIMERS_PER_CLIENT as u32);

        let key = self
            .events
            .iter()
            .find(|(_, ev)| ev.client_id == client_id && ev.timer_id == timer_id)
            .and_then(|(&key, _)| Some(key))
            .ok_or(TimerServiceError::NoSuchTimer)?;
        self.events.remove(&key);

        Ok(())
    }

    pub fn service_interrupt(&mut self) {
        extern "C" {
            fn timer_emit(badge: seL4_Word);
        }

        self.timer.ack_interrupt();

        while let Some(e) = self.events.first_entry() {
            if *e.key() > self.timer.now() {
                break;
            }
            let event = self.events.pop_first().unwrap().1;

            // client_id is 1-indexed by seL4, timer_state is 0-index.
            self.timer_state[event.client_id - 1] |= 1 << event.timer_id;

            unsafe {
                timer_emit(event.client_id);
            }

            // Re-queue if periodic.
            if let Some(period) = event.recurring {
                self.events.insert(self.timer.deadline(period), event);
            }
        }

        if let Some(event) = self.events.first_entry() {
            self.timer.set_alarm(*event.key())
        }
    }
}
