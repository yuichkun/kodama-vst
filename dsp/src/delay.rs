#[cfg(target_arch = "wasm32")]
use alloc::vec::Vec;

use libm::{floorf, powf, roundf, sqrtf};

const MAX_DELAY_SAMPLES: usize = 192000;
const MAX_VOICES: usize = 16;
const DECAY_PER_TAP: f32 = 0.75;
const DELAY_TIME_RAMP_MS: f32 = 5.0;
const FEEDBACK_RAMP_MS: f32 = 5.0;
const MIX_RAMP_MS: f32 = 5.0;
const VOICES_RAMP_MS: f32 = 10.0;
pub const WAVEFORM_BUFFER_SIZE: usize = 512;

#[derive(Clone, Copy)]
struct ParamSmoother {
    current: f32,
    target: f32,
    step: f32,
    remaining: usize,
}

impl ParamSmoother {
    fn new(value: f32) -> Self {
        Self {
            current: value,
            target: value,
            step: 0.0,
            remaining: 0,
        }
    }

    fn set_target(&mut self, value: f32, ramp_samples: usize) {
        self.target = value;
        if ramp_samples <= 1 {
            self.current = value;
            self.step = 0.0;
            self.remaining = 0;
        } else {
            self.step = (value - self.current) / ramp_samples as f32;
            self.remaining = ramp_samples;
        }
    }

    fn next_value(&mut self) -> f32 {
        if self.remaining > 0 {
            self.current += self.step;
            self.remaining -= 1;
            if self.remaining == 0 {
                self.current = self.target;
            }
        }
        self.current
    }
}

fn ramp_samples(sample_rate: f32, ramp_ms: f32) -> usize {
    (roundf(sample_rate * (ramp_ms / 1000.0)) as usize).max(1)
}

pub struct DelayProcessor {
    delay_buffer: Vec<f32>,
    write_position: usize,
    sample_rate: f32,
    delay_time: ParamSmoother,
    feedback: ParamSmoother,
    mix: ParamSmoother,
    voices_current: usize,
    voices_target: usize,
    voices_from: usize,
    voices_to: usize,
    voices_transition_remaining: usize,
    voices_transition_total: usize,
    delay_time_ramp_samples: usize,
    feedback_ramp_samples: usize,
    mix_ramp_samples: usize,
    voices_ramp_samples: usize,
    voice_waveforms: [[f32; WAVEFORM_BUFFER_SIZE]; MAX_VOICES],
    waveform_write_index: usize,
}

impl DelayProcessor {
    pub fn new(sample_rate: f32) -> Self {
        let delay_time_ramp_samples = ramp_samples(sample_rate, DELAY_TIME_RAMP_MS);
        let feedback_ramp_samples = ramp_samples(sample_rate, FEEDBACK_RAMP_MS);
        let mix_ramp_samples = ramp_samples(sample_rate, MIX_RAMP_MS);
        let voices_ramp_samples = ramp_samples(sample_rate, VOICES_RAMP_MS);

        Self {
            delay_buffer: vec![0.0; MAX_DELAY_SAMPLES],
            write_position: 0,
            sample_rate,
            delay_time: ParamSmoother::new(300.0),
            feedback: ParamSmoother::new(0.3),
            mix: ParamSmoother::new(0.5),
            voices_current: 1,
            voices_target: 1,
            voices_from: 1,
            voices_to: 1,
            voices_transition_remaining: 0,
            voices_transition_total: 0,
            delay_time_ramp_samples,
            feedback_ramp_samples,
            mix_ramp_samples,
            voices_ramp_samples,
            voice_waveforms: [[0.0; WAVEFORM_BUFFER_SIZE]; MAX_VOICES],
            waveform_write_index: 0,
        }
    }

    pub fn set_sample_rate(&mut self, sample_rate: f32) {
        self.sample_rate = sample_rate;
        self.delay_time_ramp_samples = ramp_samples(sample_rate, DELAY_TIME_RAMP_MS);
        self.feedback_ramp_samples = ramp_samples(sample_rate, FEEDBACK_RAMP_MS);
        self.mix_ramp_samples = ramp_samples(sample_rate, MIX_RAMP_MS);
        self.voices_ramp_samples = ramp_samples(sample_rate, VOICES_RAMP_MS);

        self.delay_time
            .set_target(self.delay_time.target, self.delay_time_ramp_samples);
        self.feedback
            .set_target(self.feedback.target, self.feedback_ramp_samples);
        self.mix.set_target(self.mix.target, self.mix_ramp_samples);

        if self.voices_transition_remaining > 0 {
            self.voices_transition_total = self.voices_ramp_samples;
            self.voices_transition_remaining = self.voices_transition_total;
        }
    }

    pub fn set_delay_time(&mut self, ms: f32) {
        let target = ms.clamp(0.0, 2000.0);
        self.delay_time
            .set_target(target, self.delay_time_ramp_samples);
    }

    pub fn set_feedback(&mut self, value: f32) {
        let target = value.clamp(0.0, 1.0);
        self.feedback
            .set_target(target, self.feedback_ramp_samples);
    }

    pub fn set_mix(&mut self, value: f32) {
        let target = value.clamp(0.0, 1.0);
        self.mix.set_target(target, self.mix_ramp_samples);
    }

    pub fn set_voices(&mut self, value: usize) {
        let target = value.clamp(1, MAX_VOICES);
        if target == self.voices_target && self.voices_transition_remaining == 0 {
            return;
        }

        self.voices_target = target;
        if self.voices_current == self.voices_target && self.voices_transition_remaining == 0 {
            return;
        }

        let from = if self.voices_transition_remaining > 0 {
            self.voices_to
        } else {
            self.voices_current
        };

        self.voices_from = from;
        self.voices_to = self.voices_target;
        self.voices_transition_total = self.voices_ramp_samples;
        self.voices_transition_remaining = self.voices_transition_total;

        if self.voices_transition_total <= 1 {
            self.voices_current = self.voices_to;
            self.voices_transition_remaining = 0;
            self.voices_transition_total = 0;
        }
    }

    pub fn get_delay_time(&self) -> f32 {
        self.delay_time.target
    }

    pub fn get_feedback(&self) -> f32 {
        self.feedback.target
    }

    pub fn get_mix(&self) -> f32 {
        self.mix.target
    }

    pub fn get_voices(&self) -> usize {
        self.voices_target
    }

    /// Tap gain = distance_decay^tap_index / sqrt(sum of squared decays)
    fn calculate_tap_gain(&self, tap_index: usize, voices: usize) -> f32 {
        let raw_gain = powf(DECAY_PER_TAP, tap_index as f32);
        let total_power: f32 = (0..voices)
            .map(|i| powf(DECAY_PER_TAP, (i * 2) as f32))
            .sum();
        raw_gain / sqrtf(total_power)
    }

    /// Tap delay with phase offset: base_delay*(tap+1) + (base_delay/voices)*tap
    fn calculate_tap_delay(&self, tap_index: usize, voices: usize, base_delay_samples: f32) -> f32 {
        let base_tap_delay = base_delay_samples * (tap_index + 1) as f32;
        let offset = if voices > 1 {
            (base_delay_samples * tap_index as f32) / voices as f32
        } else {
            0.0
        };
        (base_tap_delay + offset).min((MAX_DELAY_SAMPLES - 1) as f32)
    }

    fn read_delay_sample(&self, delay_samples: f32) -> f32 {
        let buffer_len = MAX_DELAY_SAMPLES as f32;
        let mut read_pos = self.write_position as f32 - delay_samples;
        while read_pos < 0.0 {
            read_pos += buffer_len;
        }
        while read_pos >= buffer_len {
            read_pos -= buffer_len;
        }

        let index0 = floorf(read_pos) as usize;
        let index1 = (index0 + 1) % MAX_DELAY_SAMPLES;
        let frac = read_pos - index0 as f32;
        let sample0 = self.delay_buffer[index0];
        let sample1 = self.delay_buffer[index1];
        sample0 + (sample1 - sample0) * frac
    }

    fn calculate_wet(
        &self,
        voices: usize,
        base_delay_samples: f32,
        voice_samples: &mut [f32; MAX_VOICES],
    ) -> (f32, f32) {
        let mut wet = 0.0_f32;
        let mut last_tap_output = 0.0_f32;

        for tap in 0..voices {
            let tap_delay = self.calculate_tap_delay(tap, voices, base_delay_samples);
            let tap_output = self.read_delay_sample(tap_delay);
            let gain = self.calculate_tap_gain(tap, voices);
            let voice_sample = tap_output * gain;

            voice_samples[tap] = voice_sample;
            wet += voice_sample;

            if tap == voices - 1 {
                last_tap_output = tap_output;
            }
        }

        for tap in voices..MAX_VOICES {
            voice_samples[tap] = 0.0;
        }

        (wet, last_tap_output)
    }

    pub fn process_stereo(
        &mut self,
        left_in: &[f32],
        right_in: &[f32],
        left_out: &mut [f32],
        right_out: &mut [f32],
    ) {
        let num_samples = left_in
            .len()
            .min(right_in.len())
            .min(left_out.len())
            .min(right_out.len());

        let max_base_delay = (MAX_DELAY_SAMPLES / MAX_VOICES - 1) as f32;

        for i in 0..num_samples {
            let input_mono = (left_in[i] + right_in[i]) * 0.5;
            let delay_time_ms = self.delay_time.next_value();
            let feedback = self.feedback.next_value();
            let mix = self.mix.next_value();
            let base_delay_samples = ((delay_time_ms / 1000.0) * self.sample_rate)
                .clamp(1.0, max_base_delay);

            let mut voice_samples_from = [0.0_f32; MAX_VOICES];
            let mut voice_samples_to = [0.0_f32; MAX_VOICES];

            let (wet, last_tap_output) = if self.voices_transition_remaining > 0 {
                let total = self.voices_transition_total.max(1);
                let index = total - self.voices_transition_remaining;
                let blend = if total <= 1 {
                    1.0
                } else {
                    index as f32 / (total - 1) as f32
                };

                self.voices_transition_remaining -= 1;
                if self.voices_transition_remaining == 0 {
                    self.voices_current = self.voices_to;
                }

                let (wet_from, last_from) =
                    self.calculate_wet(self.voices_from, base_delay_samples, &mut voice_samples_from);
                let (wet_to, last_to) =
                    self.calculate_wet(self.voices_to, base_delay_samples, &mut voice_samples_to);

                for tap in 0..MAX_VOICES {
                    let blended = voice_samples_from[tap] * (1.0 - blend)
                        + voice_samples_to[tap] * blend;
                    self.voice_waveforms[tap][self.waveform_write_index] = blended;
                }

                (
                    wet_from * (1.0 - blend) + wet_to * blend,
                    last_from * (1.0 - blend) + last_to * blend,
                )
            } else {
                let (wet, last_tap_output) =
                    self.calculate_wet(self.voices_current, base_delay_samples, &mut voice_samples_from);

                for tap in 0..MAX_VOICES {
                    self.voice_waveforms[tap][self.waveform_write_index] = voice_samples_from[tap];
                }

                (wet, last_tap_output)
            };

            self.delay_buffer[self.write_position] = input_mono + last_tap_output * feedback;

            left_out[i] = left_in[i] * (1.0 - mix) + wet * mix;
            right_out[i] = right_in[i] * (1.0 - mix) + wet * mix;

            self.write_position = (self.write_position + 1) % MAX_DELAY_SAMPLES;
            self.waveform_write_index = (self.waveform_write_index + 1) % WAVEFORM_BUFFER_SIZE;
        }
    }

    pub fn reset(&mut self) {
        self.delay_time.set_target(self.delay_time.target, 1);
        self.feedback.set_target(self.feedback.target, 1);
        self.mix.set_target(self.mix.target, 1);
        self.voices_current = self.voices_target;
        self.voices_from = self.voices_target;
        self.voices_to = self.voices_target;
        self.voices_transition_remaining = 0;
        self.voices_transition_total = 0;

        self.delay_buffer.fill(0.0);
        self.write_position = 0;
        for voice in &mut self.voice_waveforms {
            voice.fill(0.0);
        }
        self.waveform_write_index = 0;
    }

    pub fn get_voice_waveform(&self, voice_index: usize) -> &[f32; WAVEFORM_BUFFER_SIZE] {
        &self.voice_waveforms[voice_index.min(MAX_VOICES - 1)]
    }

    pub fn get_waveform_write_index(&self) -> usize {
        self.waveform_write_index
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_delay_processor_creation() {
        let processor = DelayProcessor::new(44100.0);
        assert_eq!(processor.get_delay_time(), 300.0);
        assert_eq!(processor.get_feedback(), 0.3);
        assert_eq!(processor.get_mix(), 0.5);
        assert_eq!(processor.get_voices(), 1);
    }

    #[test]
    fn test_parameter_clamping() {
        let mut processor = DelayProcessor::new(44100.0);

        processor.set_delay_time(-100.0);
        assert_eq!(processor.get_delay_time(), 0.0);

        processor.set_delay_time(5000.0);
        assert_eq!(processor.get_delay_time(), 2000.0);

        processor.set_feedback(1.5);
        assert_eq!(processor.get_feedback(), 1.0);

        processor.set_mix(-0.5);
        assert_eq!(processor.get_mix(), 0.0);

        processor.set_voices(0);
        assert_eq!(processor.get_voices(), 1);

        processor.set_voices(20);
        assert_eq!(processor.get_voices(), MAX_VOICES);
    }

    #[test]
    fn test_process_stereo_dry() {
        let mut processor = DelayProcessor::new(44100.0);
        processor.set_delay_time(0.0);
        processor.set_mix(0.0);

        let warmup = vec![0.0f32; 512];
        let mut warmup_out_l = vec![0.0f32; 512];
        let mut warmup_out_r = vec![0.0f32; 512];
        processor.process_stereo(&warmup, &warmup, &mut warmup_out_l, &mut warmup_out_r);

        let input = [1.0, 0.5, 0.0, -0.5, -1.0];
        let mut left_out = [0.0f32; 5];
        let mut right_out = [0.0f32; 5];

        processor.process_stereo(&input, &input, &mut left_out, &mut right_out);

        for i in 0..5 {
            assert!((left_out[i] - input[i]).abs() < 0.001);
            assert!((right_out[i] - input[i]).abs() < 0.001);
        }
    }

    #[test]
    fn test_voices_single_equals_original_behavior() {
        let mut processor = DelayProcessor::new(44100.0);
        processor.set_voices(1);
        processor.set_delay_time(100.0);
        processor.set_feedback(0.5);
        processor.set_mix(1.0);

        let warmup = vec![0.0f32; 1024];
        let mut warmup_out_l = vec![0.0f32; 1024];
        let mut warmup_out_r = vec![0.0f32; 1024];
        processor.process_stereo(&warmup, &warmup, &mut warmup_out_l, &mut warmup_out_r);

        let mut input_l = vec![0.0f32; 8820];
        let mut input_r = vec![0.0f32; 8820];
        input_l[0] = 1.0;
        input_r[0] = 1.0;

        let mut output_l = vec![0.0f32; 8820];
        let mut output_r = vec![0.0f32; 8820];

        processor.process_stereo(&input_l, &input_r, &mut output_l, &mut output_r);

        let delay_sample = 4410;
        assert!(output_l[delay_sample].abs() > 0.1, "Expected delay tap at 100ms");
        assert!((output_l[delay_sample] - output_r[delay_sample]).abs() < 0.001);
    }

    #[test]
    fn test_tap_delay_with_offset() {
        let mut processor = DelayProcessor::new(44100.0);
        processor.set_voices(4);

        let base_delay = 1000.0;

        let tap0 = processor.calculate_tap_delay(0, 4, base_delay);
        let tap1 = processor.calculate_tap_delay(1, 4, base_delay);
        let tap2 = processor.calculate_tap_delay(2, 4, base_delay);
        let tap3 = processor.calculate_tap_delay(3, 4, base_delay);

        assert!((tap0 - 1000.0).abs() < 0.001);
        assert!((tap1 - 2250.0).abs() < 0.001);
        assert!((tap2 - 3500.0).abs() < 0.001);
        assert!((tap3 - 4750.0).abs() < 0.001);

        let interval_01 = tap1 - tap0;
        let interval_12 = tap2 - tap1;
        let interval_23 = tap3 - tap2;
        assert!((interval_01 - interval_12).abs() < 0.001);
        assert!((interval_12 - interval_23).abs() < 0.001);
    }

    #[test]
    fn test_rms_roughly_constant_across_voice_counts() {
        let sample_rate = 44100.0;
        let test_duration_samples = 44100;

        let mut rms_values = Vec::new();

        for voice_count in [1, 2, 4, 8, 16] {
            let mut processor = DelayProcessor::new(sample_rate);
            processor.set_voices(voice_count);
            processor.set_delay_time(50.0);
            processor.set_feedback(0.0);
            processor.set_mix(1.0);

            let warmup = vec![0.0f32; 1024];
            let mut warmup_out_l = vec![0.0f32; 1024];
            let mut warmup_out_r = vec![0.0f32; 1024];
            processor.process_stereo(&warmup, &warmup, &mut warmup_out_l, &mut warmup_out_r);

            let mut input_l = vec![0.0f32; test_duration_samples];
            let mut input_r = vec![0.0f32; test_duration_samples];
            for i in (0..test_duration_samples).step_by(4410) {
                input_l[i] = 1.0;
                input_r[i] = 1.0;
            }

            let mut output_l = vec![0.0f32; test_duration_samples];
            let mut output_r = vec![0.0f32; test_duration_samples];

            processor.process_stereo(&input_l, &input_r, &mut output_l, &mut output_r);

            let sum_squares: f32 = output_l
                .iter()
                .zip(output_r.iter())
                .map(|(l, r)| l * l + r * r)
                .sum();
            let rms = (sum_squares / (2.0 * test_duration_samples as f32)).sqrt();
            rms_values.push((voice_count, rms));
        }

        let base_rms = rms_values[0].1;
        for (voices, rms) in &rms_values {
            let ratio = rms / base_rms;
            assert!(
                ratio > 0.5 && ratio < 2.0,
                "RMS for {} voices ({}) differs too much from base ({})",
                voices,
                rms,
                base_rms
            );
        }
    }
}
