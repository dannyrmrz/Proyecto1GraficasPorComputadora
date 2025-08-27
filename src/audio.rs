// audio.rs

use raylib::prelude::*;
use std::process::Command;
use std::thread;
use std::time::Duration;

pub struct AudioManager {
    has_music_file: bool,
    has_sound_file: bool,
    audio_working: bool,
    music_playing: bool,
    music_process: Option<std::process::Child>,
}

impl AudioManager {
    pub fn new(_rl: &mut RaylibHandle, _thread: &RaylibThread) -> Self {
        let mut has_music_file = false;
        let mut has_sound_file = false;
        let mut audio_working = false;

        // Check if audio files exist
        has_music_file = std::fs::metadata("assets/music.ogg").is_ok();
        has_sound_file = std::fs::metadata("assets/pickup.wav").is_ok();

        // Try to initialize audio system using system commands
        audio_working = Self::check_audio_system();

        if audio_working {
            println!("✅ Audio system initialized successfully using system commands");
        } else {
            println!("⚠️ Audio system not available, using fallback mode");
        }

        AudioManager {
            has_music_file,
            has_sound_file,
            audio_working,
            music_playing: false,
            music_process: None,
        }
    }

    fn check_audio_system() -> bool {
        // Check if we can use system audio commands
        let commands = ["paplay", "aplay", "ffplay", "mpv", "cvlc"];
        
        for cmd in commands.iter() {
            if let Ok(output) = Command::new("which").arg(cmd).output() {
                if output.status.success() {
                    println!("🎵 Found audio command: {}", cmd);
                    return true;
                }
            }
        }
        
        false
    }

    pub fn play_background_music(&mut self, _rl: &mut RaylibHandle) {
        if !self.has_music_file {
            println!("⚠️ No music file available");
            return;
        }

        if self.music_playing {
            println!("🎵 Music already playing");
            return;
        }

        println!("🎵 Starting background music...");
        
        // Try to play music using available system commands
        if let Some(process) = Self::play_music_file() {
            self.music_process = Some(process);
            self.music_playing = true;
            println!("✅ Background music started and process stored");
        } else {
            println!("❌ Failed to start background music");
        }
    }

    fn play_music_file() -> Option<std::process::Child> {
        let commands: Vec<(&str, Vec<&str>)> = vec![
            ("paplay", vec!["assets/music.ogg"]),
            ("aplay", vec!["assets/music.ogg"]),
            ("ffplay", vec!["-nodisp", "-autoexit", "-loop", "0", "assets/music.ogg"]),
            ("mpv", vec!["--no-video", "--loop", "assets/music.ogg"]),
            ("cvlc", vec!["--intf", "dummy", "--repeat", "assets/music.ogg"]),
        ];

        for (cmd, args) in commands.iter() {
            if let Ok(output) = Command::new("which").arg(cmd).output() {
                if output.status.success() {
                    println!("🎵 Trying to play music with: {}", cmd);
                    let result = Command::new(cmd).args(args).spawn();
                    match result {
                        Ok(process) => {
                            println!("✅ Music started with {}", cmd);
                            return Some(process);
                        }
                        Err(e) => {
                            println!("❌ Failed to start music with {}: {}", cmd, e);
                        }
                    }
                }
            }
        }
        
        println!("⚠️ No audio command available for music playback");
        None
    }

    pub fn update_music(&self, _rl: &mut RaylibHandle) {
        // Music is handled by system commands
        // This function is kept for compatibility
    }

    pub fn stop_music(&mut self) {
        if let Some(mut process) = self.music_process.take() {
            println!("🛑 Stopping background music...");
            
            // Try to terminate the process gracefully
            if let Err(e) = process.kill() {
                println!("❌ Failed to kill music process: {}", e);
            } else {
                println!("✅ Background music stopped");
            }
            
            self.music_playing = false;
        }
    }

    pub fn play_pickup_sound(&self, _rl: &mut RaylibHandle) {
        if !self.has_sound_file {
            println!("⚠️ No sound file available");
            return;
        }

        println!("🔊 Playing pickup sound...");
        
        // Play sound effect using system commands
        let _sound_thread = thread::spawn(|| {
            Self::play_sound_file();
        });

        println!("✅ Pickup sound thread started");
    }

    fn play_sound_file() {
        let commands: Vec<(&str, Vec<&str>)> = vec![
            ("paplay", vec!["assets/pickup.wav"]),
            ("aplay", vec!["assets/pickup.wav"]),
            ("ffplay", vec!["-nodisp", "-autoexit", "assets/pickup.wav"]),
            ("mpv", vec!["--no-video", "assets/pickup.wav"]),
            ("cvlc", vec!["--intf", "dummy", "assets/pickup.wav"]),
        ];

        for (cmd, args) in commands.iter() {
            if let Ok(output) = Command::new("which").arg(cmd).output() {
                if output.status.success() {
                    println!("🔊 Playing sound with: {}", cmd);
                    let result = Command::new(cmd).args(args).spawn();
                    match result {
                        Ok(_) => {
                            println!("✅ Sound played with {}", cmd);
                            return;
                        }
                        Err(e) => {
                            println!("❌ Failed to play sound with {}: {}", cmd, e);
                        }
                    }
                }
            }
        }
        
        println!("⚠️ No audio command available for sound playback");
    }

    pub fn has_background_music(&self) -> bool {
        self.has_music_file && self.audio_working
    }

    pub fn has_pickup_sound(&self) -> bool {
        self.has_sound_file && self.audio_working
    }

    pub fn is_audio_working(&self) -> bool {
        self.audio_working
    }
}

impl Drop for AudioManager {
    fn drop(&mut self) {
        println!("🔄 AudioManager being dropped, stopping music...");
        self.stop_music();
    }
}
