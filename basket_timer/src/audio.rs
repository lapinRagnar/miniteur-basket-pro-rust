use anyhow::Result;
use rodio::{Decoder, OutputStream, Sink, source::SineWave};
use rodio::Source;
use std::fs::File;
use std::io::BufReader;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use std::time::Duration;
// use tokio::task;

#[derive(Clone)]
pub struct AudioManager {
    sink: Arc<Mutex<Option<Sink>>>,
    _stream: Arc<OutputStream>, // garder le stream vivant
}

impl AudioManager {
    pub fn new() -> Result<Self> {
        let (_stream, stream_handle) = OutputStream::try_default()?;
        let sink = Sink::try_new(&stream_handle)?;
        Ok(Self {
            sink: Arc::new(Mutex::new(Some(sink))),
            _stream: Arc::new(_stream),
        })
    }

    fn play_file(&self, path: &PathBuf) {
        let sink_arc = self.sink.clone();
        let path = path.clone();
        // On lance dans un thread séparé pour ne pas bloquer
        std::thread::spawn(move || {
            if let Ok(mut sink_guard) = sink_arc.lock() {
                if let Some(sink) = sink_guard.as_mut() {
                    if let Ok(file) = File::open(&path) {
                        if let Ok(source) = Decoder::new(BufReader::new(file)) {
                            sink.append(source);
                            sink.sleep_until_end(); // on attend la fin dans ce thread
                            return;
                        }
                    }
                }
            }
            eprintln!("❌ Erreur lecture: {:?}", path);
        });
    }

    pub fn play_number(&self, number: u32) {
        let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        path.push("assets/numbers");
        path.push(format!("{}.wav", number));
        self.play_file(&path);
    }

    pub fn play_siren(&self) {
        let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        path.push("assets/sirene.wav");
        if path.exists() {
            self.play_file(&path);
        } else {
            // bip fallback dans un thread
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

    // Test synchrone mais dans un thread séparé (pour ne pas bloquer l'UI)
    pub fn test_sound(&self) {
        let sink_arc = self.sink.clone();
        std::thread::spawn(move || {
            if let Ok(mut sink_guard) = sink_arc.lock() {
                if let Some(sink) = sink_guard.as_mut() {
                    let source = SineWave::new(440.0).take_duration(Duration::from_secs(1));
                    sink.append(source);
                    sink.sleep_until_end();
                    println!("✅ Test son terminé");
                } else {
                    eprintln!("❌ Aucun sink disponible");
                }
            } else {
                eprintln!("❌ Impossible de verrouiller le sink");
            }
        });
    }
}

impl Default for AudioManager {
    fn default() -> Self {
        Self::new().expect("Failed to initialize audio manager")
    }
}

unsafe impl Send for AudioManager {}
unsafe impl Sync for AudioManager {}