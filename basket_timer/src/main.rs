//! Application principale d'un minuteur de basket avec interface Dioxus.

#![allow(rust_2024_compatibility)]

mod timer;
mod audio;
mod settings;

use dioxus::prelude::*;
use timer::{BasketTimer, TimerState};
use audio::AudioManager;
use settings::AppSettings;
use std::sync::{Arc, Mutex};
use tokio::time::{sleep, Duration, interval};

fn main() {
    dioxus::launch(App);
}

/// État global partagé entre les composants.
#[derive(Clone)]
struct AppState {
    timer: Arc<Mutex<BasketTimer>>,
    audio: AudioManager,
}

/// Composant racine de l'application.
#[component]
fn App() -> Element {
    // Signaux partagés
    let mut current_time = use_signal(|| 12);
    let mut timer_state = use_signal(|| TimerState::Stopped);
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

    // Future asynchrone gérant le compte à rebours
    use_future(move || {
        let timer = timer_clone.clone();
        let audio = audio_clone.clone();
        async move {
            let mut break_remaining = 0;
            let mut ticker = interval(Duration::from_secs(1));
            ticker.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);

            loop {
                ticker.tick().await;

                let (state, current_secs, loop_enabled, break_duration, initial_seconds) = {
                    let guard = timer.lock().unwrap();
                    (guard.state, guard.current_seconds, guard.loop_enabled, guard.break_duration, guard.initial_seconds)
                };

                // Pause automatique entre cycles
                if break_remaining > 0 {
                    state_signal.set(TimerState::OnBreak);
                    break_remaining -= 1;
                    if break_remaining == 0 {
                        let mut guard = timer.lock().unwrap();
                        guard.current_seconds = initial_seconds;
                        guard.state = TimerState::Running;
                        state_signal.set(TimerState::Running);
                    }
                    continue;
                }

                match state {
                    TimerState::Running => {
                        if current_secs > 0 {
                            time_signal.set(current_secs);
                            let audio2 = audio.clone();
                            let secs = current_secs;
                            tokio::task::spawn_blocking(move || {
                                audio2.play_number(secs);
                            });
                            {
                                let mut guard = timer.lock().unwrap();
                                if guard.current_seconds > 0 {
                                    guard.current_seconds -= 1;
                                }
                            }
                        } else {
                            // Arrivé à zéro
                            time_signal.set(0);
                            let audio2 = audio.clone();
                            tokio::task::spawn_blocking(move || {
                                audio2.play_siren();
                            });
                            if loop_enabled {
                                if break_duration == 0 {
                                    let mut guard = timer.lock().unwrap();
                                    guard.current_seconds = initial_seconds;
                                } else {
                                    break_remaining = break_duration;
                                    let mut guard = timer.lock().unwrap();
                                    guard.state = TimerState::OnBreak;
                                    state_signal.set(TimerState::OnBreak);
                                }
                            } else {
                                let mut guard = timer.lock().unwrap();
                                guard.state = TimerState::Stopped;
                                state_signal.set(TimerState::Stopped);
                                time_signal.set(initial_seconds);
                            }
                        }
                    }
                    TimerState::Paused => {
                        state_signal.set(TimerState::Paused);
                        sleep(Duration::from_millis(100)).await;
                    }
                    TimerState::Stopped => {
                        state_signal.set(TimerState::Stopped);
                        sleep(Duration::from_millis(100)).await;
                    }
                    TimerState::OnBreak => {
                        sleep(Duration::from_millis(100)).await;
                    }
                }
            }
        }
    });

    // Réinitialisation
    let mut on_reset = move |_| {
        let app = app_state.read();
        let mut timer = app.timer.lock().unwrap();
        timer.reset();
        current_time.set(timer.current_seconds);
        timer_state.set(TimerState::Stopped);
    };

    // Bascule Démarrer/Pause (met à jour immédiatement l'UI)
    let on_toggle = move |_| {
        let app = app_state.read();
        let mut timer = app.timer.lock().unwrap();
        match timer.state {
            TimerState::Running => {
                timer.state = TimerState::Paused;
                timer_state.set(TimerState::Paused);
            }
            _ => {
                timer.state = TimerState::Running;
                timer_state.set(TimerState::Running);
            }
        }
    };

    // Interface utilisateur (CSS intégré)
    rsx! {
        div {
            style { "
                * {{ margin: 0; padding: 0; box-sizing: border-box; }}
                body {{ background: #0a0e1a; font-family: 'Segoe UI', Tahoma, Geneva, Verdana, sans-serif; min-height: 100vh; display: flex; justify-content: center; align-items: center; padding: 20px; }}
                .app-container {{ max-width: 800px; width: 100%; margin: 0 auto; background: #111827; border-radius: 48px; box-shadow: 0 25px 50px -12px rgba(0, 0, 0, 0.5); padding: 40px 30px 30px 30px; }}
                .status {{ text-align: center; font-size: 1.5rem; font-weight: 600; letter-spacing: 1px; margin-bottom: 20px; color: #9ca3af; text-transform: uppercase; }}
                .timer-display {{ text-align: center; font-size: 8rem; font-weight: 900; font-family: 'Courier New', monospace; color: #10b981; background: #1f2937; padding: 40px 20px; border-radius: 40px; margin: 20px 0 40px 0; letter-spacing: 8px; text-shadow: 0 0 10px rgba(16, 185, 129, 0.3); }}
                .controls {{ display: flex; justify-content: center; gap: 20px; margin-bottom: 25px; flex-wrap: wrap; }}
                .bottom-controls {{ display: flex; justify-content: center; gap: 15px; margin-top: 20px; }}
                .btn {{ padding: 14px 28px; font-size: 1.1rem; font-weight: 600; border: none; border-radius: 60px; cursor: pointer; transition: all 0.2s ease; color: white; box-shadow: 0 2px 5px rgba(0,0,0,0.2); }}
                .btn-green {{ background: #10b981; }}
                .btn-green:hover {{ background: #059669; transform: scale(1.02); }}
                .btn-yellow {{ background: #f59e0b; }}
                .btn-yellow:hover {{ background: #d97706; transform: scale(1.02); }}
                .btn-red {{ background: #ef4444; }}
                .btn-red:hover {{ background: #dc2626; transform: scale(1.02); }}
                .btn-blue {{ background: #3b82f6; }}
                .btn-blue:hover {{ background: #2563eb; transform: scale(1.02); }}
                .btn-gray {{ background: #4b5563; }}
                .btn-gray:hover {{ background: #374151; transform: scale(1.02); }}
                .btn:disabled {{ opacity: 0.5; cursor: not-allowed; transform: none; }}
                .settings-container {{ background: #1f2937; border-radius: 32px; padding: 30px; color: white; }}
                .settings-form label {{ color: #9ca3af; margin-bottom: 8px; display: block; font-weight: 500; }}
                .settings-form input {{ width: 100%; padding: 12px; border-radius: 20px; border: none; background: #111827; color: white; font-size: 1rem; margin-bottom: 20px; }}
                .settings-buttons {{ display: flex; gap: 15px; justify-content: center; margin-top: 20px; }}
                h2 {{ color: #f3f4f6; text-align: center; margin-bottom: 30px; }}
                .error-msg {{ color: #ef4444; font-size: 0.85rem; margin-top: -10px; margin-bottom: 10px; }}
            "}

            if show_settings() {
                div { class: "app-container",
                    SettingsPanel {
                        settings,
                        app_state: app_state.clone(),
                        on_close: move |_| show_settings.set(false),
                    }
                }
            } else {
                div { class: "app-container",
                    div { class: "status",
                        match timer_state() {
                            TimerState::Running => "⏲ EN COURS",
                            TimerState::Paused => "⏸ EN PAUSE",
                            TimerState::Stopped => "⏹ ARRÊTÉ",
                            TimerState::OnBreak => "☕ PAUSE",
                        }
                    }
                    div { class: "timer-display", {
                        let minutes = current_time() / 60;
                        let seconds = current_time() % 60;
                        format!("{:02}:{:02}", minutes, seconds)
                    }}
                    div { class: "controls",
                        button {
                            class: if matches!(timer_state(), TimerState::Running) { "btn btn-yellow" } else { "btn btn-green" },
                            onclick: on_toggle,
                            if matches!(timer_state(), TimerState::Running) { "⏸ Pause" } else { "▶ Démarrer" }
                        }
                        button {
                            class: "btn btn-red",
                            disabled: matches!(timer_state(), TimerState::Stopped) && current_time() == 0,
                            onclick: move |_| on_reset(()),
                            "🔄 Reset"
                        }
                        button {
                            class: "btn btn-blue",
                            onclick: move |_| show_settings.set(true),
                            "⚙ Paramètres"
                        }
                    }
                    div { class: "bottom-controls",
                        button {
                            class: "btn btn-gray",
                            onclick: move |_| std::process::exit(0),
                            "🚪 Quitter"
                        }
                    }
                }
            }
        }
    }
}

/// Panneau de configuration (paramètres)
#[component]
fn SettingsPanel(
    mut settings: Signal<AppSettings>,
    app_state: Signal<AppState>,
    on_close: EventHandler<()>,
) -> Element {
    let mut start_val = use_signal(|| settings().start_seconds);
    let mut break_val = use_signal(|| settings().break_seconds);
    let mut loop_val = use_signal(|| settings().loop_enabled);
    let mut error_msg = use_signal(|| String::new());

    let on_save = move |_| {
        let new_start = start_val();
        let new_break = break_val();
        if new_start < 4 || new_start > 12 {
            error_msg.set("Le temps de départ doit être compris entre 4 et 12 secondes".to_string());
            return;
        }
        if new_break < 4 || new_break > 12 {
            error_msg.set("La durée de pause doit être comprise entre 4 et 12 secondes".to_string());
            return;
        }
        error_msg.set(String::new());
        let new = AppSettings {
            start_seconds: new_start,
            break_seconds: new_break,
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
    };

    rsx! {
        div { class: "settings-container",
            h2 { "Paramètres" }
            div { class: "settings-form",
                label { "Temps de départ (secondes) :" }
                input {
                    r#type: "number",
                    value: "{start_val}",
                    oninput: move |e| {
                        if let Ok(val) = e.value().parse::<u32>() {
                            start_val.set(val);
                            error_msg.set(String::new());
                        }
                    },
                    min: "4",
                    max: "12",
                    step: "1",
                }
                label { "Durée de pause (secondes) :" }
                input {
                    r#type: "number",
                    value: "{break_val}",
                    oninput: move |e| {
                        if let Ok(val) = e.value().parse::<u32>() {
                            break_val.set(val);
                            error_msg.set(String::new());
                        }
                    },
                    min: "4",
                    max: "12",
                    step: "1",
                }
                label { "Répéter en boucle :" }
                input {
                    r#type: "checkbox",
                    checked: "{loop_val}",
                    oninput: move |e| {
                        loop_val.set(e.value() == "true");
                    },
                }
                if !error_msg().is_empty() {
                    div { class: "error-msg", "{error_msg}" }
                }
            }
            div { class: "settings-buttons",
                button {
                    class: "btn btn-green",
                    onclick: on_save,
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