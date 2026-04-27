use chrono::Duration;
use std::sync::Arc;
use tokio::sync::Mutex;
use tokio::time::{sleep, Instant};

#[derive(Clone, PartialEq)]
pub enum TimerState {
    Stopped,
    Running,
    Paused,
    OnBreak,  // Pause entre les répétitions
}

pub struct BasketTimer {
    pub initial_seconds: u32,
    pub current_seconds: u32,
    pub break_duration: u32,
    pub loop_enabled: bool,
    pub state: TimerState,
    last_tick: Option<Instant>,
}

impl BasketTimer {
    pub fn new() -> Self {
        Self {
            initial_seconds: 12,
            current_seconds: 12,
            break_duration: 5,  // 5 secondes de pause par défaut
            loop_enabled: false,
            state: TimerState::Stopped,
            last_tick: None,
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

    pub async fn run(&mut self, on_tick: Arc<dyn Fn(u32) + Send + Sync>) {
        if self.state == TimerState::Running {
            return;
        }
        
        self.state = TimerState::Running;
        
        while self.state == TimerState::Running {
            let start = Instant::now();
            
            // Vérifier si on doit changer d'état
            if self.current_seconds == 0 {
                self.handle_zero_reached().await;
                continue;
            }
            
            // Appeler la fonction callback pour la seconde actuelle
            on_tick(self.current_seconds);
            
            self.current_seconds -= 1;
            
            // Attendre le prochain tick (1 seconde)
            let elapsed = start.elapsed();
            if elapsed < tokio::time::Duration::from_secs(1) {
                sleep(tokio::time::Duration::from_secs(1) - elapsed).await;
            }
        }
    }

    async fn handle_zero_reached(&mut self) {
        // Sirène jouée à 0
        if self.loop_enabled {
            self.state = TimerState::OnBreak;
            // Pause entre les répétitions
            sleep(tokio::time::Duration::from_secs(self.break_duration.into())).await;
            self.current_seconds = self.initial_seconds;
            self.state = TimerState::Running;
        } else {
            self.state = TimerState::Stopped;
        }
    }
}

impl Default for BasketTimer {
    fn default() -> Self {
        Self::new()
    }
}