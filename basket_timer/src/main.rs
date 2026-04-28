mod timer;
mod audio;
mod settings;

use dioxus::prelude::*;
use timer::{BasketTimer, TimerState};
use audio::AudioManager;
use settings::AppSettings;
use std::sync::{Arc, Mutex};
use tokio::time::{sleep, Duration};
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

    use_future(move || {
        let timer = timer_clone.clone();
        let audio = audio_clone.clone();
        async move {
            let mut break_remaining = 0u64;

            loop {
                let (state, current_secs, loop_enabled, break_duration, initial_seconds) = {
                    let guard = timer.lock().unwrap();
                    (guard.state, guard.current_seconds, guard.loop_enabled, guard.break_duration, guard.initial_seconds)
                };

                if break_remaining > 0 {
                    state_signal.set(TimerState::OnBreak);
                    sleep(Duration::from_secs(1)).await;
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
                                audio2.say_number(secs).ok();
                            });
                            {
                                let mut guard = timer.lock().unwrap();
                                if guard.current_seconds > 0 {
                                    guard.current_seconds -= 1;
                                }
                            }
                            sleep(Duration::from_secs(1)).await;
                        } else {
                            let audio2 = audio.clone();
                            tokio::task::spawn_blocking(move || {
                                audio2.play_siren().ok();
                            });
                            if loop_enabled {
                                break_remaining = break_duration as u64;
                                state_signal.set(TimerState::OnBreak);
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

    // Closure qui prend un argument () pour EventHandler
    let exit_app = move |_: ()| {
        process::exit(0);
    };

    rsx! {
        div {
            if show_settings() {
                SettingsPanel {
                    settings,
                    app_state: app_state.clone(),
                    on_close: move |_| show_settings.set(false),
                }
            } else {
                MainTimer {
                    current_time: current_time(),
                    timer_state: timer_state(),
                    app_state: app_state.clone(),
                    on_settings: move |_| show_settings.set(true),
                    on_exit: exit_app,
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
    on_exit: EventHandler<()>,
) -> Element {
    let minutes = current_time / 60;
    let seconds = current_time % 60;
    let time_display = format!("{:02}:{:02}", minutes, seconds);

    let start_disabled = matches!(timer_state, TimerState::Running | TimerState::OnBreak);
    let pause_disabled = !matches!(timer_state, TimerState::Running);
    let reset_disabled = matches!(timer_state, TimerState::Stopped) && current_time == 0;

    let status_text = match timer_state {
        TimerState::Running => "⏲ En cours...",
        TimerState::Paused => "⏸ En pause",
        TimerState::Stopped => "⏹ Chrono arrêté",
        TimerState::OnBreak => "☕ Pause de 5 secondes",
    };

    rsx! {
        div { class: "timer-container",
            div { class: "timer-display", "{time_display}" }
            div { class: "controls",
                button {
                    class: "btn btn-green",
                    disabled: start_disabled,
                    onclick: move |_| {
                        let app = app_state.read();
                        let mut timer = app.timer.lock().unwrap();
                        timer.state = TimerState::Running;
                    },
                    "▶ Démarrer"
                }
                button {
                    class: "btn btn-yellow",
                    disabled: pause_disabled,
                    onclick: move |_| {
                        let app = app_state.read();
                        let mut timer = app.timer.lock().unwrap();
                        timer.state = TimerState::Paused;
                    },
                    "⏸ Pause"
                }
                button {
                    class: "btn btn-red",
                    disabled: reset_disabled,
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
                button {
                    class: "btn btn-gray",
                    onclick: move |_| on_exit.call(()),
                    "🚪 Quitter"
                }
            }
            div { class: "timer-status", "{status_text}" }
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