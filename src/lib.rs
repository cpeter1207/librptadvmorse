//! Portable Morse renderer behind one versioned C descriptor.

#![deny(warnings)]
#![cfg_attr(coverage, feature(coverage_attribute))]

mod morse;
mod tone_client;

use core::ffi::c_char;
use core::mem::size_of;
use core::ptr;

pub use morse::{MorseError, MorseRenderer};

/// Opaque owner of one validated Morse renderer.
#[repr(C)]
pub struct MorseHandle(MorseRenderer);

/// Immutable ABI 1 Morse operation table.
#[repr(C)]
pub struct MorseDescriptor {
    struct_size: u32,
    abi_version: u32,
    capability_name: *const c_char,
    create: unsafe extern "C" fn(*const u8, usize, u32, f32, i8, *mut *mut MorseHandle) -> i32,
    render: unsafe extern "C" fn(*mut MorseHandle, *mut f32, usize, *mut usize) -> i32,
    restart: unsafe extern "C" fn(*mut MorseHandle) -> i32,
    destroy: unsafe extern "C" fn(*mut MorseHandle),
}

// SAFETY: the descriptor and its NUL-terminated identity never change.
unsafe impl Sync for MorseDescriptor {}

static DESCRIPTOR: MorseDescriptor = MorseDescriptor {
    struct_size: size_of::<MorseDescriptor>() as u32,
    abi_version: 1,
    capability_name: c"rptadv.morse".as_ptr(),
    create,
    render,
    restart,
    destroy,
};

/// Return the immutable ABI 1 Morse descriptor for setup-time validation.
#[unsafe(no_mangle)]
pub extern "C" fn rptadv_morse_descriptor() -> *const MorseDescriptor {
    &raw const DESCRIPTOR
}

unsafe extern "C" fn create(
    text: *const u8,
    text_len: usize,
    speed_wpm: u32,
    frequency_hz: f32,
    level_db: i8,
    out: *mut *mut MorseHandle,
) -> i32 {
    let Some(out) = (unsafe { out.as_mut() }) else {
        return 1;
    };
    *out = ptr::null_mut();
    let bytes = if text_len == 0 {
        &[][..]
    } else if text.is_null() {
        return 1;
    } else {
        // SAFETY: the caller supplies a readable text_len-sized byte span.
        unsafe { core::slice::from_raw_parts(text, text_len) }
    };
    let Ok(text) = core::str::from_utf8(bytes) else {
        return 1;
    };
    let Ok(renderer) = MorseRenderer::new(text, speed_wpm, frequency_hz, level_db) else {
        return 1;
    };
    *out = Box::into_raw(Box::new(MorseHandle(renderer)));
    0
}

unsafe extern "C" fn render(
    handle: *mut MorseHandle,
    output: *mut f32,
    capacity: usize,
    written: *mut usize,
) -> i32 {
    let Some(written) = (unsafe { written.as_mut() }) else {
        return 1;
    };
    *written = 0;
    let Some(handle) = (unsafe { handle.as_mut() }) else {
        return 1;
    };
    let output = if capacity == 0 {
        &mut [][..]
    } else if output.is_null() {
        return 1;
    } else {
        // SAFETY: the caller supplies a writable capacity-sized output span.
        unsafe { core::slice::from_raw_parts_mut(output, capacity) }
    };
    *written = handle.0.render(output);
    0
}

unsafe extern "C" fn restart(handle: *mut MorseHandle) -> i32 {
    let Some(handle) = (unsafe { handle.as_mut() }) else {
        return 1;
    };
    handle.0.restart();
    0
}

unsafe extern "C" fn destroy(handle: *mut MorseHandle) {
    if !handle.is_null() {
        // SAFETY: ownership returns once after callback users have stopped.
        drop(unsafe { Box::from_raw(handle) });
    }
}

#[cfg(test)]
#[cfg_attr(coverage, coverage(off))]
#[path = "abi_tests.rs"]
mod tests;
