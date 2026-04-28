#![allow(dependency_on_unit_never_type_fallback)]

mod timer;
mod audio;
mod settings;

use dioxus::prelude::*;
use timer::{BasketTimer, TimerState};
use audio::AudioManager;
use settings::AppSettings;
use std::sync::{Arc, Mutex};
use tokio::time::{sleep, Duration, interval};
use std::process;

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
                                let _ = audio2.play_number(secs);
                            });
                            {
                                let mut guard = timer.lock().unwrap();
                                if guard.current_seconds > 0 {
                                    guard.current_seconds -= 1;
                                }
                            }
                        } else {
                            let audio2 = audio.clone();
                            tokio::task::spawn_blocking(move || {
                                let _ = audio2.play_siren();
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

    rsx! {
        div {
            // Style CSS avec accolades échappées
            style { "
                * {{ margin: 0; padding: 0; box-sizing: border-box; }}
                body {{ background: #0a0e1a; font-family: 'Segoe UI', Tahoma, Geneva, Verdana, sans-serif; min-height: 100vh; display: flex; justify-content: center; align-items: center; padding: 20px; }}
                .app-container {{ max-width: 800px; width: 100%; margin: 0 auto; background: #111827; border-radius: 48px; box-shadow: 0 25px 50px -12px rgba(0, 0, 0, 0.5); padding: 40px 30px 30px 30px; transition: all 0.3s ease; }}
                .status {{ text-align: center; font-size: 1.5rem; font-weight: 600; letter-spacing: 1px; margin-bottom: 20px; color: #9ca3af; text-transform: uppercase; }}
                .timer-display {{ text-align: center; font-size: 8rem; font-weight: 900; font-family: 'Courier New', monospace; color: #10b981; background: #1f2937; padding: 40px 20px; border-radius: 40px; margin: 20px 0 40px 0; letter-spacing: 8px; text-shadow: 0 0 10px rgba(16, 185, 129, 0.3); box-shadow: inset 0 2px 5px rgba(0,0,0,0.2), 0 5px 15px rgba(0,0,0,0.1); }}
                .controls {{ display: flex; justify-content: center; gap: 20px; margin-bottom: 25px; flex-wrap: wrap; }}
                .bottom-controls {{ display: flex; justify-content: center; margin-top: 10px; }}
                .btn {{ padding: 14px 28px; font-size: 1.1rem; font-weight: 600; border: none; border-radius: 60px; cursor: pointer; transition: all 0.2s ease; color: white; box-shadow: 0 2px 5px rgba(0,0,0,0.2); }}
                .btn-green {{ background: #10b981; }}
                .btn-green:hover {{ background: #059669; transform: scale(1.02); box-shadow: 0 5px 15px rgba(16, 185, 129, 0.4); }}
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
                            class: "btn btn-green",
                            disabled: matches!(timer_state(), TimerState::Running | TimerState::OnBreak),
                            onclick: move |_| {
                                let app = app_state.read();
                                let mut timer = app.timer.lock().unwrap();
                                timer.state = TimerState::Running;
                            },
                            "▶ Démarrer"
                        }
                        button {
                            class: "btn btn-yellow",
                            disabled: !matches!(timer_state(), TimerState::Running),
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
                            onclick: move |_| show_settings.set(true),
                            "⚙ Paramètres"
                        }
                    }
                    div { class: "bottom-controls",
                        button {
                            class: "btn btn-gray",
                            onclick: move |_| process::exit(0),
                            "🚪 Quitter"
                        }
                    }
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