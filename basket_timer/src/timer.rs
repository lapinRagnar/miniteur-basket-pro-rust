#[derive(Clone, Copy, PartialEq)]
pub enum TimerState {
    Stopped,
    Running,
    Paused,
    OnBreak,
}

pub struct BasketTimer {
    pub initial_seconds: u32,
    pub current_seconds: u32,
    pub break_duration: u32,
    pub loop_enabled: bool,
    pub state: TimerState,
}

impl BasketTimer {
    pub fn new() -> Self {
        Self {
            initial_seconds: 12,
            current_seconds: 12,
            break_duration: 5,
            loop_enabled: false,
            state: TimerState::Stopped,
        }
    }

    pub fn set_time(&mut self, seconds: u32) {
        self.initial_seconds = seconds;
        self.current_seconds = seconds;
    }

    pub fn reset(&mut self) {
        self.current_seconds = self.initial_seconds;
        self.state = TimerState::Stopped;
    }
}

impl Default for BasketTimer {
    fn default() -> Self {
        Self::new()
    }
}