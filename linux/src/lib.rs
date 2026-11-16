#[cfg(target_os = "linux")]
mod linux;
#[cfg(target_os = "openbsd")]
mod openbsd;

#[cfg(target_os = "linux")]
pub use linux::fb as fb;
#[cfg(target_os = "openbsd")]
pub use openbsd::fb as fb;

pub mod raw;
pub mod keys;
pub mod pty;

//touch screen = linux only for now
#[cfg(target_os = "linux")]
pub mod input;
