use colored::*;
use rodio::{Decoder, OutputStream};
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;
use std::time::Duration;
use tokio::task;
use tokio::time::sleep;
use glob::glob;

fn parse_lrc(file_path: &Path) -> Vec<(u64, String)> {
    let file = File::open(file_path).expect("LRC file not found");
    let reader = BufReader::new(file);

    let mut lyrics = Vec::new();

    for line in reader.lines() {
        if let Ok(line) = line {
            if let Some(close_bracket) = line.find(']') {
                let time_str = &line[1..close_bracket];
                let parts: Vec<&str> = time_str.split(':').collect();
                if parts.len() == 2 {
                    let mins: u64 = parts[0].parse().unwrap_or(0);
                    let secs_parts: Vec<&str> = parts[1].split('.').collect();
                    let secs: u64 = secs_parts[0].parse().unwrap_or(0);
                    let millis: u64 = if secs_parts.len() > 1 {
                        secs_parts[1].parse().unwrap_or(0) * 10
                    } else { 0 };
                    let total_ms = mins * 60_000 + secs * 1000 + millis;
                    let text = line[(close_bracket + 1)..].to_string();
                    lyrics.push((total_ms, text));
                }
            }
        }
    }

    lyrics
}


fn play_music(path: &Path) {
    let (_stream, stream_handle) = OutputStream::try_default().unwrap();
    let file = File::open(path).unwrap();
    let source = Decoder::new(BufReader::new(file)).unwrap();
    stream_handle.play_raw(source.convert_samples()).unwrap();

   
    std::thread::sleep(Duration::from_secs(600));
}


async fn scroll_lyrics_approx(lyrics: &str) {
    for line in lyrics.lines() {
        println!("{}", line.cyan().bold());
        sleep(Duration::from_millis(3000)).await; 
    }
}

#[tokio::main]
async fn main() {
  
    let mut songs = Vec::new();
    for entry in glob("songs/*.mp3").unwrap() {
        if let Ok(path) = entry {
            songs.push(path);
        }
    }

    if songs.is_empty() {
        println!("{}", "No songs found in songs/ folder!".red().bold());
        return;
    }

    println!("{}", "Available songs:".green().bold());
    for (i, song) in songs.iter().enumerate() {
        println!("{}: {}", i + 1, song.file_name().unwrap().to_string_lossy());
    }

    println!("{}", "\nEnter song number to play:".yellow().bold());
    let mut input = String::new();
    std::io::stdin().read_line(&mut input).unwrap();
    let choice: usize = input.trim().parse().unwrap_or(1);
    let song_path = &songs[choice - 1];
    println!("{}", format!("Playing: {}", song_path.display()).green().bold());

    
    let lrc_path = song_path.with_extension("lrc");
    let lyrics = if lrc_path.exists() {
        parse_lrc(&lrc_path)
    } else {
        vec![]
    };

    
    let music_handle = task::spawn_blocking(move || {
        play_music(song_path);
    });


    if !lyrics.is_empty() {
        let start_time = tokio::time::Instant::now();
        for (time_ms, line) in lyrics {
            let elapsed = start_time.elapsed().as_millis() as u64;
            if time_ms > elapsed {
                sleep(Duration::from_millis(time_ms - elapsed)).await;
            }
            println!("{}", line.cyan().bold());
        }
    } else {
        println!("{}", "No LRC found, scrolling lyrics approximately...".yellow());
       
        scroll_lyrics_approx("Your song lyrics here...").await;
    }

    music_handle.await.unwrap();
    println!("{}", "🎵 End of Karaoke 🎵".green().bold());
}
