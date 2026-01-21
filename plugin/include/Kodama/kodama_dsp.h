#ifndef KODAMA_DSP_H
#define KODAMA_DSP_H

#include <stddef.h>
#include <stdint.h>

#ifdef __cplusplus
extern "C" {
#endif

typedef struct KodamaDspHandle KodamaDspHandle;

KodamaDspHandle* kodama_dsp_create(float sample_rate);
void kodama_dsp_destroy(KodamaDspHandle* handle);
void kodama_dsp_set_sample_rate(KodamaDspHandle* handle, float sample_rate);
void kodama_dsp_set_delay_time(KodamaDspHandle* handle, float ms);
void kodama_dsp_set_feedback(KodamaDspHandle* handle, float value);
void kodama_dsp_set_mix(KodamaDspHandle* handle, float value);
void kodama_dsp_set_voices(KodamaDspHandle* handle, uint32_t value);
void kodama_dsp_process(
    KodamaDspHandle* handle,
    const float* left_in,
    const float* right_in,
    float* left_out,
    float* right_out,
    size_t num_samples
);
void kodama_dsp_reset(KodamaDspHandle* handle);
uint32_t kodama_dsp_get_voice_count(KodamaDspHandle* handle);
size_t kodama_dsp_get_waveform_size(void);
void kodama_dsp_get_voice_waveform(KodamaDspHandle* handle, uint32_t voice_index, float* out_ptr);

#ifdef __cplusplus
}
#endif

#endif
