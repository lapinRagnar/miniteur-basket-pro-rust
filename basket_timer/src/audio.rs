use anyhow::Result;
use rodio::{Decoder, OutputStream, Sink, source::SineWave};
use rodio::Source;
use std::fs::File;
use std::io::BufReader;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use std::time::Duration;
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
        println!("🔊 Voix: {}", text);
        if let Ok(mut tts) = self.tts.lock() {
            tts.speak(&text, false)?;
        }
        Ok(())
    }
    
    pub fn play_siren(&self) -> Result<()> {
        println!("🚨 SIRENE !!!");
        let mut siren_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        siren_path.push("assets/sirene.wav");
        
        if siren_path.exists() {
            println!("Fichier trouvé: {:?}", siren_path);
            if let Ok(mut sink_guard) = self.sink.lock() {
                if let Some(sink) = sink_guard.as_mut() {
                    match File::open(&siren_path) {
                        Ok(file) => {
                            match Decoder::new(BufReader::new(file)) {
                                Ok(source) => {
                                    sink.append(source);
                                    sink.sleep_until_end(); // attend la fin de la sirène
                                    println!("Sirène jouée avec succès");
                                    return Ok(());
                                },
                                Err(e) => println!("Erreur décodage: {}", e),
                            }
                        },
                        Err(e) => println!("Erreur ouverture fichier: {}", e),
                    }
                }
            }
            println!("Échec de lecture de la sirène");
        } else {
            println!("Fichier sirène introuvable à {:?}", siren_path);
        }
        
        // Fallback : bip aigu
        println!("Jouer bip de secours");
        if let Ok(mut sink_guard) = self.sink.lock() {
            if let Some(sink) = sink_guard.as_mut() {
                let source = SineWave::new(880.0).take_duration(Duration::from_secs(1));
                sink.append(source);
                sink.sleep_until_end();
            }
        }
        Ok(())
    }
}

impl Default for AudioManager {
    fn default() -> Self {
        Self::new().expect("Failed to initialize audio manager")
    }
}

unsafe impl Send for AudioManager {}
unsafe impl Sync for AudioManager {}