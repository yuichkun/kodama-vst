use crate::delay::WAVEFORM_BUFFER_SIZE;
use crate::DelayProcessor;

#[repr(C)]
pub struct KodamaDspHandle {
    processor: DelayProcessor,
}

#[no_mangle]
pub extern "C" fn kodama_dsp_create(sample_rate: f32) -> *mut KodamaDspHandle {
    let handle = Box::new(KodamaDspHandle {
        processor: DelayProcessor::new(sample_rate),
    });
    Box::into_raw(handle)
}

/// # Safety
/// `handle` must be a valid pointer returned by `kodama_dsp_create`
#[no_mangle]
pub unsafe extern "C" fn kodama_dsp_destroy(handle: *mut KodamaDspHandle) {
    if !handle.is_null() {
        drop(Box::from_raw(handle));
    }
}

/// # Safety
/// `handle` must be a valid pointer returned by `kodama_dsp_create`
#[no_mangle]
pub unsafe extern "C" fn kodama_dsp_set_sample_rate(handle: *mut KodamaDspHandle, sample_rate: f32) {
    if let Some(h) = handle.as_mut() {
        h.processor.set_sample_rate(sample_rate);
    }
}

/// # Safety
/// `handle` must be a valid pointer returned by `kodama_dsp_create`
#[no_mangle]
pub unsafe extern "C" fn kodama_dsp_set_delay_time(handle: *mut KodamaDspHandle, ms: f32) {
    if let Some(h) = handle.as_mut() {
        h.processor.set_delay_time(ms);
    }
}

/// # Safety
/// `handle` must be a valid pointer returned by `kodama_dsp_create`
#[no_mangle]
pub unsafe extern "C" fn kodama_dsp_set_feedback(handle: *mut KodamaDspHandle, value: f32) {
    if let Some(h) = handle.as_mut() {
        h.processor.set_feedback(value);
    }
}

/// # Safety
/// `handle` must be a valid pointer returned by `kodama_dsp_create`
#[no_mangle]
pub unsafe extern "C" fn kodama_dsp_set_mix(handle: *mut KodamaDspHandle, value: f32) {
    if let Some(h) = handle.as_mut() {
        h.processor.set_mix(value);
    }
}

/// # Safety
/// `handle` must be a valid pointer returned by `kodama_dsp_create`
#[no_mangle]
pub unsafe extern "C" fn kodama_dsp_set_voices(handle: *mut KodamaDspHandle, value: u32) {
    if let Some(h) = handle.as_mut() {
        h.processor.set_voices(value as usize);
    }
}

/// # Safety
/// - `handle` must be a valid pointer returned by `kodama_dsp_create`
/// - All buffer pointers must be valid for `num_samples` elements
#[no_mangle]
pub unsafe extern "C" fn kodama_dsp_process(
    handle: *mut KodamaDspHandle,
    left_in: *const f32,
    right_in: *const f32,
    left_out: *mut f32,
    right_out: *mut f32,
    num_samples: usize,
) {
    if handle.is_null()
        || left_in.is_null()
        || right_in.is_null()
        || left_out.is_null()
        || right_out.is_null()
    {
        return;
    }

    let h = &mut *handle;
    let left_in_slice = std::slice::from_raw_parts(left_in, num_samples);
    let right_in_slice = std::slice::from_raw_parts(right_in, num_samples);
    let left_out_slice = std::slice::from_raw_parts_mut(left_out, num_samples);
    let right_out_slice = std::slice::from_raw_parts_mut(right_out, num_samples);

    h.processor
        .process_stereo(left_in_slice, right_in_slice, left_out_slice, right_out_slice);
}

/// # Safety
/// `handle` must be a valid pointer returned by `kodama_dsp_create`
#[no_mangle]
pub unsafe extern "C" fn kodama_dsp_reset(handle: *mut KodamaDspHandle) {
    if let Some(h) = handle.as_mut() {
        h.processor.reset();
    }
}

/// # Safety
/// `handle` must be a valid pointer returned by `kodama_dsp_create`
#[no_mangle]
pub unsafe extern "C" fn kodama_dsp_get_voice_count(handle: *mut KodamaDspHandle) -> u32 {
    if let Some(h) = handle.as_ref() {
        h.processor.get_voices() as u32
    } else {
        0
    }
}

#[no_mangle]
pub extern "C" fn kodama_dsp_get_waveform_size() -> usize {
    WAVEFORM_BUFFER_SIZE
}

/// # Safety
/// - `handle` must be a valid pointer returned by `kodama_dsp_create`
/// - `out_ptr` must be valid for WAVEFORM_BUFFER_SIZE elements
#[no_mangle]
pub unsafe extern "C" fn kodama_dsp_get_voice_waveform(
    handle: *mut KodamaDspHandle,
    voice_index: u32,
    out_ptr: *mut f32,
) {
    if handle.is_null() || out_ptr.is_null() {
        return;
    }

    let h = &*handle;
    let waveform = h.processor.get_voice_waveform(voice_index as usize);
    let write_idx = h.processor.get_waveform_write_index();
    let out_slice = std::slice::from_raw_parts_mut(out_ptr, WAVEFORM_BUFFER_SIZE);

    for i in 0..WAVEFORM_BUFFER_SIZE {
        let read_idx = (write_idx + i) % WAVEFORM_BUFFER_SIZE;
        out_slice[i] = waveform[read_idx];
    }
}
