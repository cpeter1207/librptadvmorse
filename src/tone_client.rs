//! Dynamic client for the released tone oscillator ABI.

use core::ffi::c_char;
use core::mem::size_of;
use core::ptr::NonNull;
use std::ffi::CStr;

use crate::morse::MorseError;

#[repr(C)]
struct ToneOscillator {
    _private: [u8; 0],
}

/// Prefix safe to inspect before the complete descriptor's size is known.
#[repr(C)]
#[derive(Clone, Copy)]
struct TonePrefix {
    struct_size: u32,
    abi_version: u32,
    capability_name: *const c_char,
}

type Create = unsafe extern "C" fn(f32, f32, *mut *mut ToneOscillator) -> i32;
type Reset = unsafe extern "C" fn(*mut ToneOscillator) -> i32;
type Render = unsafe extern "C" fn(*mut ToneOscillator, *mut f32, usize) -> i32;
type Destroy = unsafe extern "C" fn(*mut ToneOscillator);

/// The ABI 1 tone descriptor, including its sequence entries before oscillator.
#[repr(C)]
#[derive(Clone, Copy)]
struct ToneDescriptor {
    prefix: TonePrefix,
    sequence_create: Option<unsafe extern "C" fn(*const u8, usize, i8, *mut *mut ()) -> i32>,
    sequence_rendered_samples: Option<unsafe extern "C" fn(*const (), *mut usize) -> i32>,
    sequence_render: Option<unsafe extern "C" fn(*mut (), *mut f32, usize, *mut usize) -> i32>,
    sequence_destroy: Option<unsafe extern "C" fn(*mut ())>,
    oscillator_create: Option<Create>,
    oscillator_reset: Option<Reset>,
    oscillator_render: Option<Render>,
    oscillator_destroy: Option<Destroy>,
}

#[link(name = "rptadvtone")]
unsafe extern "C" {
    fn rptadv_tone_descriptor() -> *const TonePrefix;
}

/// One setup-allocated oscillator, exclusively owned by a Morse renderer.
pub(crate) struct Oscillator {
    handle: NonNull<ToneOscillator>,
    reset: Reset,
    render: Render,
    destroy: Destroy,
}

// SAFETY: the handle is exclusively owned and never used concurrently.
unsafe impl Send for Oscillator {}

impl Oscillator {
    /// Validate the released descriptor and prepare the original waveform parameters.
    pub(crate) fn new(frequency_hz: f32, amplitude: f32) -> Result<Self, MorseError> {
        // SAFETY: the versioned shared object keeps its immutable descriptor live.
        unsafe { Self::from_descriptor(rptadv_tone_descriptor(), frequency_hz, amplitude) }
    }

    /// Check the prefix before reading and caching oscillator callbacks.
    unsafe fn from_descriptor(
        descriptor: *const TonePrefix,
        frequency_hz: f32,
        amplitude: f32,
    ) -> Result<Self, MorseError> {
        // SAFETY: the caller provides either null or a readable descriptor prefix.
        let prefix = unsafe { descriptor.as_ref() }.ok_or(MorseError)?;
        if (prefix.struct_size as usize) < size_of::<ToneDescriptor>()
            || prefix.abi_version != 1
            || prefix.capability_name.is_null()
            || unsafe { CStr::from_ptr(prefix.capability_name) }.to_bytes() != b"rptadv.tone"
        {
            return Err(MorseError);
        }
        // SAFETY: the prefix proved enough bytes exist for every ABI 1 field.
        let api = unsafe { &*descriptor.cast::<ToneDescriptor>() };
        let (Some(create), Some(reset), Some(render), Some(destroy)) = (
            api.oscillator_create,
            api.oscillator_reset,
            api.oscillator_render,
            api.oscillator_destroy,
        ) else {
            return Err(MorseError);
        };
        let mut handle = core::ptr::null_mut();
        // SAFETY: `handle` is writable and the released ABI validates setup arguments.
        if unsafe { create(frequency_hz, amplitude, &mut handle) } != 0 {
            return Err(MorseError);
        }
        Ok(Self {
            handle: NonNull::new(handle).ok_or(MorseError)?,
            reset,
            render,
            destroy,
        })
    }

    /// Reset phase at a keyed Morse element boundary.
    pub(crate) fn reset(&mut self) {
        // SAFETY: this renderer owns a valid oscillator handle exclusively.
        let _ = unsafe { (self.reset)(self.handle.as_ptr()) };
    }

    /// Fill a keyed span without allocating or locking.
    pub(crate) fn render(&mut self, output: &mut [f32]) {
        // SAFETY: output is a writable span and this renderer owns the handle.
        let _ = unsafe { (self.render)(self.handle.as_ptr(), output.as_mut_ptr(), output.len()) };
    }
}

impl Drop for Oscillator {
    fn drop(&mut self) {
        // SAFETY: the last callback user has stopped before renderer destruction.
        unsafe { (self.destroy)(self.handle.as_ptr()) };
    }
}

#[cfg(test)]
#[cfg_attr(coverage, coverage(off))]
mod tests {
    use super::*;

    unsafe extern "C" fn create_null(
        _frequency_hz: f32,
        _amplitude: f32,
        out: *mut *mut ToneOscillator,
    ) -> i32 {
        // SAFETY: test supplies a writable output handle.
        unsafe { *out = core::ptr::null_mut() };
        0
    }

    #[test]
    fn rejects_incompatible_or_incomplete_tone_descriptor() {
        // SAFETY: the published library provides a complete ABI 1 descriptor.
        let real = unsafe { *rptadv_tone_descriptor().cast::<ToneDescriptor>() };
        let mut candidate = real;
        let pointer = &candidate.prefix as *const TonePrefix;
        assert!(unsafe { Oscillator::from_descriptor(core::ptr::null(), 1000.0, 0.5) }.is_err());

        candidate.prefix.struct_size = 0;
        assert!(unsafe { Oscillator::from_descriptor(pointer, 1000.0, 0.5) }.is_err());
        candidate = real;
        candidate.prefix.abi_version = 0;
        assert!(unsafe { Oscillator::from_descriptor(pointer, 1000.0, 0.5) }.is_err());
        candidate = real;
        candidate.prefix.capability_name = core::ptr::null();
        assert!(unsafe { Oscillator::from_descriptor(pointer, 1000.0, 0.5) }.is_err());
        candidate = real;
        candidate.prefix.capability_name = c"wrong.tone".as_ptr();
        assert!(unsafe { Oscillator::from_descriptor(pointer, 1000.0, 0.5) }.is_err());

        candidate = real;
        candidate.oscillator_create = None;
        assert!(unsafe { Oscillator::from_descriptor(pointer, 1000.0, 0.5) }.is_err());
        candidate = real;
        candidate.oscillator_reset = None;
        assert!(unsafe { Oscillator::from_descriptor(pointer, 1000.0, 0.5) }.is_err());
        candidate = real;
        candidate.oscillator_render = None;
        assert!(unsafe { Oscillator::from_descriptor(pointer, 1000.0, 0.5) }.is_err());
        candidate = real;
        candidate.oscillator_destroy = None;
        assert!(unsafe { Oscillator::from_descriptor(pointer, 1000.0, 0.5) }.is_err());

        candidate = real;
        candidate.oscillator_create = Some(create_null);
        assert!(unsafe { Oscillator::from_descriptor(pointer, 1000.0, 0.5) }.is_err());
        assert!(Oscillator::new(-1.0, 0.5).is_err());
    }
}
