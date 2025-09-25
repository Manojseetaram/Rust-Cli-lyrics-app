use clap::Parser;
use colored::*;
use reqwest;
use serde::Deserialize;
use tokio::time::{sleep, Duration};

#[derive(Parser)]
#[command(author, version, about = "Terminal Lyrics Viewer")]
struct Args {
    /// Song name
    song: String,
    /// Artist name
    artist: String,
}

#[derive(Deserialize)]
struct LyricsResponse {
    lyrics: String,
}

async fn fetch_lyrics(artist: &str, song: &str) -> Result<String, reqwest::Error> {
    let url = format!("https://api.lyrics.ovh/v1/{}/{}", artist, song);
    let resp: LyricsResponse = reqwest::get(&url).await?.json().await?;
    Ok(resp.lyrics)
}

#[tokio::main]
async fn main() {
    let args = Args::parse();

    println!(
        "{}",
        format!("Fetching lyrics for '{}' by '{}'", args.song, args.artist)
            .yellow()
            .bold()
    );

    match fetch_lyrics(&args.artist, &args.song).await {
        Ok(lyrics) => {
            println!("{}", "\n=== Lyrics ===\n".green().bold());

            // Split lyrics by lines
            for line in lyrics.lines() {
                println!("{}", line.cyan());
                sleep(Duration::from_millis(500)).await; // Scroll effect
            }

            println!("{}", "\n=== End of Lyrics ===".green().bold());
        }
        Err(_) => {
            println!("{}", "Lyrics not found or API error!".red().bold());
        }
    }
}
