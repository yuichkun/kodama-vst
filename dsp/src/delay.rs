#[cfg(target_arch = "wasm32")]
use alloc::vec::Vec;

use libm::{powf, sqrtf};

const MAX_DELAY_SAMPLES: usize = 192000;
const MAX_VOICES: usize = 16;
const DECAY_PER_TAP: f32 = 0.75;

pub struct DelayProcessor {
    delay_buffer: Vec<f32>,
    write_position: usize,
    sample_rate: f32,
    delay_time_ms: f32,
    feedback: f32,
    mix: f32,
    voices: usize,
}

impl DelayProcessor {
    pub fn new(sample_rate: f32) -> Self {
        Self {
            delay_buffer: vec![0.0; MAX_DELAY_SAMPLES],
            write_position: 0,
            sample_rate,
            delay_time_ms: 300.0,
            feedback: 0.3,
            mix: 0.5,
            voices: 1,
        }
    }

    pub fn set_sample_rate(&mut self, sample_rate: f32) {
        self.sample_rate = sample_rate;
    }

    pub fn set_delay_time(&mut self, ms: f32) {
        self.delay_time_ms = ms.clamp(0.0, 2000.0);
    }

    pub fn set_feedback(&mut self, value: f32) {
        self.feedback = value.clamp(0.0, 1.0);
    }

    pub fn set_mix(&mut self, value: f32) {
        self.mix = value.clamp(0.0, 1.0);
    }

    pub fn set_voices(&mut self, value: usize) {
        self.voices = value.clamp(1, MAX_VOICES);
    }

    pub fn get_delay_time(&self) -> f32 {
        self.delay_time_ms
    }

    pub fn get_feedback(&self) -> f32 {
        self.feedback
    }

    pub fn get_mix(&self) -> f32 {
        self.mix
    }

    pub fn get_voices(&self) -> usize {
        self.voices
    }

    /// Tap gain = distance_decay^tap_index / sqrt(sum of squared decays)
    fn calculate_tap_gain(&self, tap_index: usize) -> f32 {
        let raw_gain = powf(DECAY_PER_TAP, tap_index as f32);
        let total_power: f32 = (0..self.voices)
            .map(|i| powf(DECAY_PER_TAP, (i * 2) as f32))
            .sum();
        raw_gain / sqrtf(total_power)
    }

    /// Tap delay with phase offset: base_delay*(tap+1) + (base_delay/voices)*tap
    fn calculate_tap_delay(&self, tap_index: usize, base_delay_samples: usize) -> usize {
        let base_tap_delay = base_delay_samples * (tap_index + 1);
        let offset = if self.voices > 1 {
            (base_delay_samples * tap_index) / self.voices
        } else {
            0
        };
        (base_tap_delay + offset).min(MAX_DELAY_SAMPLES - 1)
    }

    pub fn process_stereo(
        &mut self,
        left_in: &[f32],
        right_in: &[f32],
        left_out: &mut [f32],
        right_out: &mut [f32],
    ) {
        let base_delay_samples = ((self.delay_time_ms / 1000.0) * self.sample_rate) as usize;
        let base_delay_samples = base_delay_samples.clamp(1, MAX_DELAY_SAMPLES / MAX_VOICES - 1);

        let num_samples = left_in
            .len()
            .min(right_in.len())
            .min(left_out.len())
            .min(right_out.len());

        for i in 0..num_samples {
            let input_mono = (left_in[i] + right_in[i]) * 0.5;

            let mut wet = 0.0_f32;
            let mut last_tap_output = 0.0_f32;

            for tap in 0..self.voices {
                let tap_delay = self.calculate_tap_delay(tap, base_delay_samples);

                let read_pos = if self.write_position >= tap_delay {
                    self.write_position - tap_delay
                } else {
                    MAX_DELAY_SAMPLES - (tap_delay - self.write_position)
                };

                let tap_output = self.delay_buffer[read_pos];
                let gain = self.calculate_tap_gain(tap);

                wet += tap_output * gain;

                if tap == self.voices - 1 {
                    last_tap_output = tap_output;
                }
            }

            self.delay_buffer[self.write_position] = input_mono + last_tap_output * self.feedback;

            left_out[i] = left_in[i] * (1.0 - self.mix) + wet * self.mix;
            right_out[i] = right_in[i] * (1.0 - self.mix) + wet * self.mix;

            self.write_position = (self.write_position + 1) % MAX_DELAY_SAMPLES;
        }
    }

    pub fn reset(&mut self) {
        self.delay_buffer.fill(0.0);
        self.write_position = 0;
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

        let base_delay = 1000;

        let tap0 = processor.calculate_tap_delay(0, base_delay);
        let tap1 = processor.calculate_tap_delay(1, base_delay);
        let tap2 = processor.calculate_tap_delay(2, base_delay);
        let tap3 = processor.calculate_tap_delay(3, base_delay);

        assert_eq!(tap0, 1000);
        assert_eq!(tap1, 2250);
        assert_eq!(tap2, 3500);
        assert_eq!(tap3, 4750);

        let interval_01 = tap1 - tap0;
        let interval_12 = tap2 - tap1;
        let interval_23 = tap3 - tap2;
        assert_eq!(interval_01, interval_12);
        assert_eq!(interval_12, interval_23);
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
