mod timer;
mod audio;
mod settings;

use dioxus::prelude::*;
use timer::{BasketTimer, TimerState};
use audio::AudioManager;
use settings::AppSettings;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

fn main() {
    dioxus::launch(App);
}

#[derive(Clone)]
struct AppState {
    timer: Arc<Mutex<BasketTimer>>,
    audio: AudioManager,
    settings: AppSettings,
}

#[component]
fn App() -> Element {
    let mut current_time = use_signal(|| 12);
    let mut timer_state = use_signal(|| TimerState::Stopped);
    let mut show_settings = use_signal(|| false);
    
    let settings = use_signal(|| AppSettings::load());
    
    let app_state = use_signal(|| {
        let mut timer = BasketTimer::new();
        let loaded_settings = settings();
        timer.set_time(loaded_settings.start_seconds);
        timer.break_duration = loaded_settings.break_seconds;
        timer.loop_enabled = loaded_settings.loop_enabled;
        
        AppState {
            timer: Arc::new(Mutex::new(timer)),
            audio: AudioManager::default(),
            settings: loaded_settings,
        }
    });
    
    let timer_clone = app_state().timer.clone();
    let audio_clone = app_state().audio.clone();
    let time_signal = current_time;
    let state_signal = timer_state;
    
    use_effect(move || {
        thread::spawn(move || {
            loop {
                let should_run = {
                    let timer = timer_clone.lock().unwrap();
                    timer.state == TimerState::Running
                };
                
                if !should_run {
                    thread::sleep(Duration::from_millis(100));
                    continue;
                }
                
                let current_seconds = {
                    let mut timer = timer_clone.lock().unwrap();
                    let secs = timer.current_seconds;
                    
                    if secs > 0 {
                        timer.current_seconds -= 1;
                    }
                    
                    secs
                };
                
                if current_seconds > 0 {
                    // Mettre à jour l'affichage
                    time_signal.set(current_seconds);
                    // Dire le nombre
                    audio_clone.say_number(current_seconds).ok();
                } else if current_seconds == 0 {
                    // Jouer la sirène
                    audio_clone.play_siren().ok();
                    
                    // Gérer la boucle
                    let mut timer = timer_clone.lock().unwrap();
                    if timer.loop_enabled {
                        timer.current_seconds = timer.initial_seconds;
                        timer.state = TimerState::OnBreak;
                        drop(timer);
                        
                        // Pause
                        thread::sleep(Duration::from_secs(
                            timer_clone.lock().unwrap().break_duration as u64
                        ));
                        
                        let mut timer = timer_clone.lock().unwrap();
                        timer.state = TimerState::Running;
                    } else {
                        timer.state = TimerState::Stopped;
                        state_signal.set(TimerState::Stopped);
                    }
                }
                
                thread::sleep(Duration::from_secs(1));
            }
        });
    });
    
    rsx! {
        div {
            if show_settings() {
                SettingsPanel {
                    settings: settings,
                    app_state: app_state,
                    on_close: move |_| show_settings.set(false),
                }
            } else {
                MainTimer {
                    current_time: current_time(),
                    timer_state: timer_state(),
                    app_state: app_state,
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
                {
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
}

#[component]
fn SettingsPanel(
    settings: Signal<AppSettings>,
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
                        let new_settings = AppSettings {
                            start_seconds: start_val(),
                            break_seconds: break_val(),
                            loop_enabled: loop_val(),
                        };
                        
                        let app = app_state.read();
                        let mut timer = app.timer.lock().unwrap();
                        timer.set_time(new_settings.start_seconds);
                        timer.break_duration = new_settings.break_seconds;
                        timer.loop_enabled = new_settings.loop_enabled;
                        timer.reset();
                        
                        let _ = new_settings.save();
                        settings.set(new_settings);
                        
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