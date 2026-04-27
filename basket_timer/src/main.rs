mod timer;
mod audio;
mod settings;

use dioxus::prelude::*;
use timer::{BasketTimer, TimerState};
use audio::AudioManager;
use settings::AppSettings;
use std::sync::Arc;
use tokio::sync::Mutex;

fn main() {
    // Lancer l'application Dioxus
    dioxus::launch(App);
}

#[derive(Clone)]
struct AppState {
    timer: Arc<Mutex<BasketTimer>>,
    audio: Arc<Mutex<AudioManager>>,
    settings: AppSettings,
}

#[component]
fn App() -> Element {
    let mut current_time = use_signal(|| 12);
    let mut timer_state = use_signal(|| TimerState::Stopped);
    let mut show_settings = use_signal(|| false);
    
    // Charger les paramètres
    let settings = use_signal(|| AppSettings::load());
    
    // Initialiser l'état de l'app
    let mut app_state = use_signal(|| {
        let mut timer = BasketTimer::new();
        let loaded_settings = settings();
        timer.set_time(loaded_settings.start_seconds);
        timer.break_duration = loaded_settings.break_seconds;
        timer.loop_enabled = loaded_settings.loop_enabled;
        
        AppState {
            timer: Arc::new(Mutex::new(timer)),
            audio: Arc::new(Mutex::new(AudioManager::default())),
            settings: loaded_settings,
        }
    });
    
    // Timer task
    use_effect(move || {
        let timer_clone = app_state().timer.clone();
        let audio_clone = app_state().audio.clone();
        let time_signal = current_time;
        let state_signal = timer_state;
        
        tokio::spawn(async move {
            let mut timer_guard = timer_clone.lock().await;
            
            // Callback pour chaque tick
            let on_tick = Arc::new(move |seconds: u32| {
                let time_signal_async = time_signal.clone();
                let audio_clone_async = audio_clone.clone();
                let state_signal_async = state_signal.clone();
                
                tokio::spawn(async move {
                    // Mettre à jour l'affichage
                    time_signal_async.set(seconds);
                    
                    // Dire le nombre à haute voix (sauf si c'est 0)
                    if seconds > 0 {
                        if let Ok(mut audio) = audio_clone_async.try_lock() {
                            let _ = audio.say_number(seconds);
                        }
                    } else {
                        // Jouer la sirène à 0
                        if let Ok(mut audio) = audio_clone_async.try_lock() {
                            let _ = audio.play_siren();
                        }
                        state_signal_async.set(TimerState::Stopped);
                    }
                });
            });
            
            timer_guard.run(on_tick).await;
        });
    });
    
    rsx! {
        div {
            style { include_str!("../assets/styles.css") }
            
            if *show_settings {
                SettingsPanel {
                    settings: settings,
                    app_state: app_state,
                    on_close: move || show_settings.set(false),
                }
            } else {
                MainTimer {
                    current_time: *current_time,
                    timer_state: *timer_state,
                    app_state: app_state,
                    on_settings: move || show_settings.set(true),
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
    on_settings: EventHandler,
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
                        let timer_clone = app.timer.clone();
                        tokio::spawn(async move {
                            let mut timer = timer_clone.lock().await;
                            timer.state = TimerState::Running;
                            // La tâche de timer est déjà en cours dans le use_effect
                        });
                    },
                    "▶ Démarrer"
                }
                
                button {
                    class: "btn btn-yellow",
                    onclick: move |_| {
                        let app = app_state.read();
                        let timer_clone = app.timer.clone();
                        tokio::spawn(async move {
                            let mut timer = timer_clone.lock().await;
                            timer.state = TimerState::Paused;
                        });
                    },
                    "⏸ Pause"
                }
                
                button {
                    class: "btn btn-red",
                    onclick: move |_| {
                        let app = app_state.read();
                        let timer_clone = app.timer.clone();
                        tokio::spawn(async move {
                            let mut timer = timer_clone.lock().await;
                            timer.reset();
                        });
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
    settings: Signal<AppSettings>,
    app_state: Signal<AppState>,
    on_close: EventHandler,
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
                        // Sauvegarder les paramètres
                        let new_settings = AppSettings {
                            start_seconds: start_val(),
                            break_seconds: break_val(),
                            loop_enabled: loop_val(),
                        };
                        
                        // Mettre à jour le timer
                        let app = app_state.read();
                        let timer_clone = app.timer.clone();
                        tokio::spawn(async move {
                            let mut timer = timer_clone.lock().await;
                            timer.set_time(new_settings.start_seconds);
                            timer.break_duration = new_settings.break_seconds;
                            timer.loop_enabled = new_settings.loop_enabled;
                            timer.reset();
                        });
                        
                        // Sauvegarder dans le fichier
                        let _ = new_settings.save();
                        
                        // Mettre à jour le signal
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