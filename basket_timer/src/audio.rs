use anyhow::Result;
use rodio::{Decoder, OutputStream, Sink, Source};
use std::fs::File;
use std::io::BufReader;
use std::path::PathBuf;
use tts::Tts;

pub struct AudioManager {
    _stream: OutputStream,
    sink: Sink,
    tts: Tts,
}

impl AudioManager {
    pub fn new() -> Result<Self> {
        let (_stream, stream_handle) = OutputStream::try_default()?;
        let sink = Sink::try_new(&stream_handle)?;
        
        // Initialiser le TTS
        let tts = Tts::default()?;
        
        Ok(Self {
            _stream,
            sink,
            tts,
        })
    }
    
    pub fn say_number(&mut self, number: u32) -> Result<()> {
        // Dire le nombre à haute voix
        let text = number.to_string();
        println!("🔊 saying: {}", text);
        
        // Sur les plateformes supportées, ça fonctionne très bien
        self.tts.speak(&text, false)?;
        
        Ok(())
    }
    
    pub fn play_siren(&mut self) -> Result<()> {
        println!("🚨 SIRENE !!! 🚨");
        
        // Essayer de charger la sirène depuis assets/
        let mut siren_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        siren_path.push("assets/sirene.wav");
        
        if siren_path.exists() {
            println!("Playing siren from: {:?}", siren_path);
            let file = File::open(siren_path)?;
            let source = Decoder::new(BufReader::new(file))?;
            
            // Jouer la sirène (non bloquant)
            self.sink.append(source);
            self.sink.sleep_until_end();
        } else {
            // Fallback : utiliser le TTS pour dire "ZERO" avec effet
            println!("Siren file not found, using TTS fallback");
            self.tts.speak("ZERO! ZERO! GAME OVER!", false)?;
        }
        
        Ok(())
    }
    
    pub fn stop_all_sounds(&mut self) {
        self.sink.stop();
    }
}

impl Default for AudioManager {
    fn default() -> Self {
        Self::new().expect("Failed to initialize audio manager")
    }
}