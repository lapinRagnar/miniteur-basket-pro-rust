mod timer;
mod audio;
mod settings;

use dioxus::prelude::*;
use timer::{BasketTimer, TimerState};
use audio::AudioManager;
use settings::AppSettings;
use std::sync::{Arc, Mutex};
use tokio::time::{sleep, Duration};

fn main() {
    dioxus::launch(App);
}

#[derive(Clone)]
struct AppState {
    timer: Arc<Mutex<BasketTimer>>,
    audio: AudioManager,
}

#[component]
fn App() -> Element {
    let current_time = use_signal(|| 12);
    let timer_state = use_signal(|| TimerState::Stopped);
    let mut show_settings = use_signal(|| false);
    let settings = use_signal(|| AppSettings::load());
    
    let app_state = use_signal(|| {
        let mut timer = BasketTimer::new();
        let loaded = settings();
        timer.set_time(loaded.start_seconds);
        timer.break_duration = loaded.break_seconds;
        timer.loop_enabled = loaded.loop_enabled;
        AppState {
            timer: Arc::new(Mutex::new(timer)),
            audio: AudioManager::default(),
        }
    });
    
    let timer_clone = app_state().timer.clone();
    let audio_clone = app_state().audio.clone();
    let mut time_signal = current_time;
    let mut state_signal = timer_state;
    
    // Tâche asynchrone – on clone à l'intérieur de la closure extérieure
    use_future(move || {
        let timer = timer_clone.clone();
        let audio = audio_clone.clone();
        async move {
            loop {
                let should_run = {
                    let timer_guard = timer.lock().unwrap();
                    timer_guard.state == TimerState::Running
                };
                if !should_run {
                    sleep(Duration::from_millis(100)).await;
                    continue;
                }
                
                let is_zero = {
                    let mut timer_guard = timer.lock().unwrap();
                    if timer_guard.current_seconds > 0 {
                        timer_guard.current_seconds -= 1;
                        false
                    } else {
                        true
                    }
                };
                
                if !is_zero {
                    let secs = { timer.lock().unwrap().current_seconds };
                    time_signal.set(secs);
                    audio.say_number(secs).ok();
                } else {
                    audio.play_siren().ok();
                    let mut timer_guard = timer.lock().unwrap();
                    if timer_guard.loop_enabled {
                        timer_guard.current_seconds = timer_guard.initial_seconds;
                        timer_guard.state = TimerState::OnBreak;
                        drop(timer_guard);
                        sleep(Duration::from_secs(
                            timer.lock().unwrap().break_duration as u64
                        )).await;
                        let mut timer_guard = timer.lock().unwrap();
                        timer_guard.state = TimerState::Running;
                    } else {
                        timer_guard.state = TimerState::Stopped;
                        state_signal.set(TimerState::Stopped);
                    }
                }
                
                sleep(Duration::from_secs(1)).await;
            }
        }
    });
    
    rsx! {
        div {
            if show_settings() {
                SettingsPanel {
                    settings,
                    app_state,
                    on_close: move |_| show_settings.set(false),
                }
            } else {
                MainTimer {
                    current_time: current_time(),
                    timer_state: timer_state(),
                    app_state,
                    on_settings: move |_| show_settings.set(true),
                }
            }
        }
    }
}

#[component]
fn MainTimer(
    current_time: u32,
    timer_state: TimerState,
    app_state: Signal<AppState>,
    on_settings: EventHandler<()>,
) -> Element {
    let minutes = current_time / 60;
    let seconds = current_time % 60;
    let time_display = format!("{:02}:{:02}", minutes, seconds);
    
    rsx! {
        div { class: "timer-container",
            div { class: "timer-display", "{time_display}" }
            div { class: "controls",
                button {
                    class: "btn btn-green",
                    disabled: timer_state == TimerState::Running,
                    onclick: move |_| {
                        let app = app_state.read();
                        let mut timer = app.timer.lock().unwrap();
                        timer.state = TimerState::Running;
                    },
                    "▶ Démarrer"
                }
                button {
                    class: "btn btn-yellow",
                    onclick: move |_| {
                        let app = app_state.read();
                        let mut timer = app.timer.lock().unwrap();
                        timer.state = TimerState::Paused;
                    },
                    "⏸ Pause"
                }
                button {
                    class: "btn btn-red",
                    onclick: move |_| {
                        let app = app_state.read();
                        let mut timer = app.timer.lock().unwrap();
                        timer.reset();
                        // Le signal `current_time` sera mis à jour dès que la future repasse
                    },
                    "🔄 Reset"
                }
                button {
                    class: "btn btn-blue",
                    onclick: move |_| on_settings.call(()),
                    "⚙ Paramètres"
                }
            }
            div { class: "timer-status",
                match timer_state {
                    TimerState::Running => "⏲ En cours...",
                    TimerState::Paused => "⏸ En pause",
                    TimerState::Stopped => "⏹ Arrêté",
                    TimerState::OnBreak => "☕ Pause",
                }
            }
        }
    }
}

#[component]
fn SettingsPanel(
    mut settings: Signal<AppSettings>,
    app_state: Signal<AppState>,
    on_close: EventHandler<()>,
) -> Element {
    let mut start_val = use_signal(|| settings().start_seconds);
    let mut break_val = use_signal(|| settings().break_seconds);
    let mut loop_val = use_signal(|| settings().loop_enabled);
    
    rsx! {
        div { class: "settings-container",
            h2 { "Paramètres" }
            div { class: "settings-form",
                label { "Temps de départ (secondes):" }
                input {
                    r#type: "number",
                    value: "{start_val}",
                    oninput: move |e| {
                        if let Ok(val) = e.value().parse::<u32>() {
                            start_val.set(val);
                        }
                    },
                    min: "1",
                    max: "3600",
                }
                label { "Durée de pause (secondes):" }
                input {
                    r#type: "number",
                    value: "{break_val}",
                    oninput: move |e| {
                        if let Ok(val) = e.value().parse::<u32>() {
                            break_val.set(val);
                        }
                    },
                    min: "0",
                    max: "300",
                }
                label { "Répéter en boucle:" }
                input {
                    r#type: "checkbox",
                    checked: "{loop_val}",
                    oninput: move |e| {
                        loop_val.set(e.value() == "true");
                    },
                }
            }
            div { class: "settings-buttons",
                button {
                    class: "btn btn-green",
                    onclick: move |_| {
                        let new = AppSettings {
                            start_seconds: start_val(),
                            break_seconds: break_val(),
                            loop_enabled: loop_val(),
                        };
                        let app = app_state.read();
                        let mut timer = app.timer.lock().unwrap();
                        timer.set_time(new.start_seconds);
                        timer.break_duration = new.break_seconds;
                        timer.loop_enabled = new.loop_enabled;
                        timer.reset();
                        let _ = new.save();
                        settings.set(new);
                        on_close.call(());
                    },
                    "💾 Sauvegarder"
                }
                button {
                    class: "btn btn-gray",
                    onclick: move |_| on_close.call(()),
                    "Annuler"
                }
            }
        }
    }
}