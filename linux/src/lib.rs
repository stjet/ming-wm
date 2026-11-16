pub mod fb;
pub mod raw;
pub mod keys;
pub mod pty;
//touch screen = linux only for now
#[cfg(target_os = "linux")]
pub mod input;
