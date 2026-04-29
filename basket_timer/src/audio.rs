//! Module audio utilisant `rodio` pour jouer des fichiers WAV sans bloquer l'interface.

use anyhow::Result;
use rodio::{Decoder, OutputStream, Sink, source::SineWave};
use rodio::Source;
use std::fs::File;
use std::io::BufReader;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use std::time::Duration;

/// Structure responsable de la lecture des sons.
#[derive(Clone)]
pub struct AudioManager {
    sink: Arc<Mutex<Option<Sink>>>,
    _stream: Arc<OutputStream>, // maintenu vivant
}

impl AudioManager {
    /// Initialise le système audio (périphérique de sortie par défaut).
    pub fn new() -> Result<Self> {
        let (_stream, stream_handle) = OutputStream::try_default()?;
        let sink = Sink::try_new(&stream_handle)?;
        Ok(Self {
            sink: Arc::new(Mutex::new(Some(sink))),
            _stream: Arc::new(_stream),
        })
    }

    /// Joue un fichier WAV de manière non‑bloquante (dans un thread séparé).
    fn play_file_non_blocking(&self, path: &PathBuf) {
        let sink_arc = self.sink.clone();
        let path = path.clone();
        std::thread::spawn(move || {
            if let Ok(mut sink_guard) = sink_arc.lock() {
                if let Some(sink) = sink_guard.as_mut() {
                    if let Ok(file) = File::open(&path) {
                        if let Ok(source) = Decoder::new(BufReader::new(file)) {
                            sink.append(source);
                            sink.sleep_until_end(); // attend la fin dans le thread
                            return;
                        }
                    }
                }
            }
            eprintln!("❌ Erreur lecture: {:?}", path);
        });
    }

    /// Joue l’annonce d’un nombre (fichier assets/numbers/{number}.wav).
    pub fn play_number(&self, number: u32) {
        let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        path.push("assets/numbers");
        path.push(format!("{}.wav", number));
        if path.exists() {
            self.play_file_non_blocking(&path);
        } else {
            eprintln!("⚠️ Fichier audio manquant: {:?}", path);
        }
    }

    /// Joue la sirène (assets/sirene.wav) ou un bip de secours.
    pub fn play_siren(&self) {
        let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        path.push("assets/sirene.wav");
        if path.exists() {
            self.play_file_non_blocking(&path);
        } else {
            // Bip de secours (880 Hz, 1 seconde)
            let sink_arc = self.sink.clone();
            std::thread::spawn(move || {
                if let Ok(mut sink_guard) = sink_arc.lock() {
                    if let Some(sink) = sink_guard.as_mut() {
                        let source = SineWave::new(880.0).take_duration(Duration::from_secs(1));
                        sink.append(source);
                        sink.sleep_until_end();
                    }
                }
            });
        }
    }
}

impl Default for AudioManager {
    fn default() -> Self {
        Self::new().expect("Échec de l'initialisation audio")
    }
}

// Nécessaire pour partager AudioManager entre threads (car rodio n'est pas Send/Sync par défaut).
unsafe impl Send for AudioManager {}
unsafe impl Sync for AudioManager {}