#[cfg(target_arch = "wasm32")]
use alloc::vec::Vec;

const MAX_DELAY_SAMPLES: usize = 192000;

pub struct DelayProcessor {
    delay_buffer: [Vec<f32>; 2],
    write_position: usize,
    sample_rate: f32,
    delay_time_ms: f32,
    feedback: f32,
    mix: f32,
}

impl DelayProcessor {
    pub fn new(sample_rate: f32) -> Self {
        Self {
            delay_buffer: [
                vec![0.0; MAX_DELAY_SAMPLES],
                vec![0.0; MAX_DELAY_SAMPLES],
            ],
            write_position: 0,
            sample_rate,
            delay_time_ms: 300.0,
            feedback: 0.3,
            mix: 0.5,
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

    pub fn get_delay_time(&self) -> f32 {
        self.delay_time_ms
    }

    pub fn get_feedback(&self) -> f32 {
        self.feedback
    }

    pub fn get_mix(&self) -> f32 {
        self.mix
    }

    pub fn process_stereo(
        &mut self,
        left_in: &[f32],
        right_in: &[f32],
        left_out: &mut [f32],
        right_out: &mut [f32],
    ) {
        let delay_samples = ((self.delay_time_ms / 1000.0) * self.sample_rate) as usize;
        let delay_samples = delay_samples.clamp(1, MAX_DELAY_SAMPLES - 1);

        let num_samples = left_in
            .len()
            .min(right_in.len())
            .min(left_out.len())
            .min(right_out.len());

        for i in 0..num_samples {
            let read_pos = if self.write_position >= delay_samples {
                self.write_position - delay_samples
            } else {
                MAX_DELAY_SAMPLES - (delay_samples - self.write_position)
            };

            let delayed_l = self.delay_buffer[0][read_pos];
            let delayed_r = self.delay_buffer[1][read_pos];

            self.delay_buffer[0][self.write_position] = left_in[i] + delayed_l * self.feedback;
            self.delay_buffer[1][self.write_position] = right_in[i] + delayed_r * self.feedback;

            left_out[i] = left_in[i] * (1.0 - self.mix) + delayed_l * self.mix;
            right_out[i] = right_in[i] * (1.0 - self.mix) + delayed_r * self.mix;

            self.write_position = (self.write_position + 1) % MAX_DELAY_SAMPLES;
        }
    }

    pub fn reset(&mut self) {
        for buffer in &mut self.delay_buffer {
            buffer.fill(0.0);
        }
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
}
