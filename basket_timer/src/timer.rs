//! Module contenant la structure du minuteur et ses états.

/// États possibles du minuteur.
#[derive(Clone, Copy, PartialEq)]
pub enum TimerState {
    /// Arrêté (timer initialisé ou remis à zéro).
    Stopped,
    /// Compte à rebours en cours.
    Running,
    /// En pause manuelle.
    Paused,
    /// Pause automatique entre deux cycles (mode boucle).
    OnBreak,
}

/// Minuteur principal.
pub struct BasketTimer {
    /// Valeur initiale (secondes).
    pub initial_seconds: u32,
    /// Valeur courante (secondes restantes).
    pub current_seconds: u32,
    /// Durée de la pause entre cycles (secondes).
    pub break_duration: u32,
    /// Active la répétition en boucle.
    pub loop_enabled: bool,
    /// État actuel du minuteur.
    pub state: TimerState,
}

impl BasketTimer {
    /// Crée un minuteur avec des valeurs par défaut (12 secondes, pause 5s, boucle désactivée).
    pub fn new() -> Self {
        Self {
            initial_seconds: 12,
            current_seconds: 12,
            break_duration: 5,
            loop_enabled: false,
            state: TimerState::Stopped,
        }
    }

    /// Modifie le temps de départ et réinitialise le minuteur.
    pub fn set_time(&mut self, seconds: u32) {
        self.initial_seconds = seconds;
        self.current_seconds = seconds;
    }

    /// Remet le minuteur à sa valeur initiale et le passe à l'état `Stopped`.
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