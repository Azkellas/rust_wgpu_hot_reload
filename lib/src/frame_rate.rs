use std::collections::VecDeque;

/// Sliding window to give a smooth framerate.
/// Sum the last `window_size` `frame_duration` to estimate the framerate.
#[derive(Debug)]
pub struct FrameRate {
    /// Size of the sliding window.
    window_size: usize,
    /// Store the last frame durations.
    window: VecDeque<f32>,
}

impl FrameRate {
    /// Create a new slicing window with the given size.
    pub const fn new(window_size: usize) -> Self {
        Self {
            window_size,
            window: VecDeque::new(),
        }
    }

    /// Add the latest `frame_duration` to the window.
    /// Drop the oldest frame duration if needed.
    pub fn update(&mut self, frame_duration: f32) {
        self.window.push_back(frame_duration);
        if self.window.len() > self.window_size {
            self.window.pop_front();
        }
    }

    /// Compute current `frame_rate`
    /// Since the mean of frame duration is `sum(window) / window_size`
    /// The number of frame per seconds is `1 / sum(window) / window_size`
    /// ie `window_size / sum(window)`
    pub fn get(&self) -> f32 {
        self.window_size as f32 / self.window.iter().sum::<f32>()
    }
}

impl Default for FrameRate {
    /// Create a default `FrameRate` with a window size of 20.
    fn default() -> Self {
        Self::new(20)
    }
}
