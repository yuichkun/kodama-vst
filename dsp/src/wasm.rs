use alloc::vec::Vec;
use core::cell::UnsafeCell;
use core::marker::Sync;
use core::mem::{drop, forget};
use core::ops::FnOnce;
use core::option::Option::{self, None, Some};
use core::slice::{from_raw_parts, from_raw_parts_mut};

use crate::delay::WAVEFORM_BUFFER_SIZE;
use crate::DelayProcessor;

/// Global processor storage for WASM (single-threaded)
struct ProcessorHolder {
    inner: UnsafeCell<Option<DelayProcessor>>,
}

unsafe impl Sync for ProcessorHolder {}

static PROCESSOR: ProcessorHolder = ProcessorHolder {
    inner: UnsafeCell::new(None),
};

fn with_processor<F, R>(f: F) -> R
where
    F: FnOnce(Option<&mut DelayProcessor>) -> R,
{
    unsafe {
        let opt = &mut *PROCESSOR.inner.get();
        f(opt.as_mut())
    }
}

#[no_mangle]
pub extern "C" fn wasm_init(sample_rate: f32) {
    unsafe {
        *PROCESSOR.inner.get() = Some(DelayProcessor::new(sample_rate));
    }
}

#[no_mangle]
pub extern "C" fn wasm_set_delay_time(ms: f32) {
    with_processor(|p| {
        if let Some(proc) = p {
            proc.set_delay_time(ms);
        }
    });
}

#[no_mangle]
pub extern "C" fn wasm_set_feedback(value: f32) {
    with_processor(|p| {
        if let Some(proc) = p {
            proc.set_feedback(value);
        }
    });
}

#[no_mangle]
pub extern "C" fn wasm_set_mix(value: f32) {
    with_processor(|p| {
        if let Some(proc) = p {
            proc.set_mix(value);
        }
    });
}

#[no_mangle]
pub extern "C" fn wasm_set_voices(value: u32) {
    with_processor(|p| {
        if let Some(proc) = p {
            proc.set_voices(value as usize);
        }
    });
}

/// # Safety
/// All buffer pointers must be valid for `num_samples` elements
#[no_mangle]
pub unsafe extern "C" fn wasm_process(
    left_in: *const f32,
    right_in: *const f32,
    left_out: *mut f32,
    right_out: *mut f32,
    num_samples: usize,
) {
    if left_in.is_null() || right_in.is_null() || left_out.is_null() || right_out.is_null() {
        return;
    }

    with_processor(|p| {
        if let Some(proc) = p {
            let left_in_slice = from_raw_parts(left_in, num_samples);
            let right_in_slice = from_raw_parts(right_in, num_samples);
            let left_out_slice = from_raw_parts_mut(left_out, num_samples);
            let right_out_slice = from_raw_parts_mut(right_out, num_samples);

            proc.process_stereo(left_in_slice, right_in_slice, left_out_slice, right_out_slice);
        }
    });
}

#[no_mangle]
pub extern "C" fn wasm_reset() {
    with_processor(|p| {
        if let Some(proc) = p {
            proc.reset();
        }
    });
}

#[no_mangle]
pub extern "C" fn wasm_get_voice_count() -> u32 {
    with_processor(|p| {
        if let Some(proc) = p {
            proc.get_voices() as u32
        } else {
            0
        }
    })
}

#[no_mangle]
pub extern "C" fn wasm_get_waveform_size() -> u32 {
    WAVEFORM_BUFFER_SIZE as u32
}

/// # Safety
/// `out_ptr` must be valid for WAVEFORM_BUFFER_SIZE elements
#[no_mangle]
pub unsafe extern "C" fn wasm_get_voice_waveform(voice_index: u32, out_ptr: *mut f32) {
    if out_ptr.is_null() {
        return;
    }

    with_processor(|p| {
        if let Some(proc) = p {
            let waveform = proc.get_voice_waveform(voice_index as usize);
            let write_idx = proc.get_waveform_write_index();
            let out_slice = from_raw_parts_mut(out_ptr, WAVEFORM_BUFFER_SIZE);

            for i in 0..WAVEFORM_BUFFER_SIZE {
                let read_idx = (write_idx + i) % WAVEFORM_BUFFER_SIZE;
                out_slice[i] = waveform[read_idx];
            }
        }
    });
}

#[no_mangle]
pub extern "C" fn wasm_alloc(size: usize) -> *mut f32 {
    let mut buf = Vec::<f32>::with_capacity(size);
    buf.resize(size, 0.0);
    let ptr = buf.as_mut_ptr();
    forget(buf);
    ptr
}

/// # Safety
/// `ptr` must have been returned by `wasm_alloc` with the same `size`
#[no_mangle]
pub unsafe extern "C" fn wasm_free(ptr: *mut f32, size: usize) {
    if !ptr.is_null() && size > 0 {
        drop(Vec::from_raw_parts(ptr, size, size));
    }
}
