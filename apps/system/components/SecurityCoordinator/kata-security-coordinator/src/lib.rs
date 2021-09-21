//! Kata OS security coordinator support

#![cfg_attr(not(test), no_std)]
// NB: "error[E0658]: trait bounds other than `Sized` on const fn parameters are unstable"
#![feature(const_fn_trait_bound)]

extern crate alloc;
use alloc::boxed::Box;
use kata_security_common::*;

#[cfg(all(feature = "fake", feature = "sel4"))]
compile_error!("features \"fake\" and \"sel4\" are mutually exclusive");

#[cfg_attr(feature = "sel4", path = "impl.rs")]
#[cfg_attr(feature = "fake", path = "fakeimpl/mod.rs")]
mod platform;
pub use platform::KataSecurityCoordinatorInterface;

#[cfg(not(test))]
pub static mut KATA_SECURITY: KataSecurityCoordinator = KataSecurityCoordinator::empty();

// KataSecurityCoordinator bundles an instance of the SecurityCoordinator that operates
// on KataOS interfaces. There is a two-step dance to setup an instance because we want
// KATA_STORAGE static.
// NB: no locking is done; we assume the caller/user is single-threaded
pub struct KataSecurityCoordinator {
    manager: Option<Box<dyn SecurityCoordinatorInterface + Sync>>,
}
impl KataSecurityCoordinator {
    // Constructs a partially-initialized instance; to complete call init().
    // This is needed because we need a const fn for static setup.
    const fn empty() -> KataSecurityCoordinator {
        KataSecurityCoordinator { manager: None }
    }

    pub fn init(&mut self) {
        self.manager = Some(Box::new(KataSecurityCoordinatorInterface::new()));
    }
}
impl SecurityCoordinatorInterface for KataSecurityCoordinator {
    fn request(
        &mut self,
        request_id: SecurityRequest,
        request_buffer: &[u8],
        reply_buffer: &mut [u8],
    ) -> Result<(), SecurityRequestError> {
        self.manager
            .as_mut()
            .unwrap()
            .request(request_id, request_buffer, reply_buffer)
    }
}
