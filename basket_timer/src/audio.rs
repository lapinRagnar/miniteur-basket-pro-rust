use anyhow::Result;
use rodio::{Decoder, OutputStream, Sink};
use std::fs::File;
use std::io::BufReader;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use tts::Tts;

#[derive(Clone)]
pub struct AudioManager {
    sink: Arc<Mutex<Option<Sink>>>,
    tts: Arc<Mutex<Tts>>,
}

impl AudioManager {
    pub fn new() -> Result<Self> {
        let (_stream, stream_handle) = OutputStream::try_default()?;
        let sink = Sink::try_new(&stream_handle)?;
        let tts = Tts::default()?;
        Ok(Self {
            sink: Arc::new(Mutex::new(Some(sink))),
            tts: Arc::new(Mutex::new(tts)),
        })
    }
    
    pub fn say_number(&self, number: u32) -> Result<()> {
        let text = number.to_string();
        println!("🔊 saying: {}", text);
        if let Ok(mut tts) = self.tts.lock() {
            let _ = tts.speak(&text, false);
        }
        Ok(())
    }
    
    pub fn play_siren(&self) -> Result<()> {
        println!("🚨 SIRENE !!! 🚨");
        let mut siren_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        siren_path.push("assets/sirene.wav");
        
        if siren_path.exists() {
            if let Ok(mut sink_guard) = self.sink.lock() {
                if let Some(sink) = sink_guard.as_mut() {
                    if let Ok(file) = File::open(&siren_path) {
                        if let Ok(source) = Decoder::new(BufReader::new(file)) {
                            sink.append(source);
                            sink.sleep_until_end();
                        }
                    }
                }
            }
        } else {
            if let Ok(mut tts) = self.tts.lock() {
                let _ = tts.speak("ZERO! ZERO! GAME OVER!", false);
            }
        }
        Ok(())
    }
    
    pub fn stop_all_sounds(&self) {
        if let Ok(mut sink_guard) = self.sink.lock() {
            if let Some(sink) = sink_guard.as_mut() {
                sink.stop();
            }
        }
    }
}

impl Default for AudioManager {
    fn default() -> Self {
        Self::new().expect("Failed to initialize audio manager")
    }
}

// Nécessaire car Rodio n'est pas Send/Sync par défaut
unsafe impl Send for AudioManager {}
unsafe impl Sync for AudioManager {}