use std::path::PathBuf;
use std::process::ExitCode;
use std::time::Duration;

use cahill_core as core;
use core::resolver::Fs;
use core::source::{Clip, Folder, List, Source, Track};
use core::timeline::{flatten, Playable};

fn main() -> ExitCode {
    let mut args = std::env::args().skip(1);
    let Some(command) = args.next() else {
        eprintln!("usage: cahill <path> | list <paths...> | clip <src> <start> <end?> | play <path>");
        return ExitCode::FAILURE;
    };
    if command == "play" {
        return play(args);
    }
    let build = match command.as_str() {
        "list" => {
            let children = args.map(path_source).collect::<Vec<_>>();
            if children.is_empty() {
                eprintln!("list needs at least one path");
                return ExitCode::FAILURE;
            }
            Source::List(List { children })
        }
        "clip" => {
            let (Some(src), Some(start), end) =
                (args.next(), args.next(), args.next()) else {
                eprintln!("clip needs <src> <start> <end?>");
                return ExitCode::FAILURE;
            };
            let start = match parse_secs(&start) {
                Some(value) => value,
                None => return ExitCode::FAILURE,
            };
            let end = match end {
                Some(raw) => match parse_secs(&raw) {
                    Some(value) => Some(value),
                    None => return ExitCode::FAILURE,
                },
                None => None,
            };
            Source::Clip(Clip {
                inner: Box::new(path_source(src)),
                start,
                end,
            })
        }
        path => path_source(path.to_string()),
    };
    match flatten(&build, &Fs) {
        Ok(timeline) => {
            let plays = timeline
                .events
                .iter()
                .map(|event| event.play.clone())
                .collect::<Vec<_>>();
            print_timeline(&plays);
            match timeline.total {
                Some(total) => println!("total: {}", fmt(total)),
                None => println!("total: unknown"),
            }
            ExitCode::SUCCESS
        }
        Err(error) => {
            eprintln!("error: {}", error);
            ExitCode::FAILURE
        }
    }
}

fn play(mut args: std::iter::Skip<std::env::Args>) -> ExitCode {
    let Some(path) = args.next() else {
        eprintln!("play needs <path>");
        return ExitCode::FAILURE;
    };
    let source = path_source(path);
    match cahill_engine::resolve(&source, &Fs) {
        Ok(resolved) => {
            println!("total: {}", fmt(resolved.total));
            for (index, event) in resolved.events.iter().enumerate() {
                let end = event.play.end.map(fmt).unwrap_or_else(|| "end".to_string());
                println!(
                    "{}. {} [{}..{}]",
                    index,
                    event.play.path.display(),
                    fmt(event.play.start),
                    end,
                );
            }
            match cahill_engine::play(&resolved) {
                Ok(player) => {
                    std::thread::sleep(player.total + Duration::from_secs(1));
                    ExitCode::SUCCESS
                }
                Err(error) => {
                    eprintln!("play error: {}", error);
                    ExitCode::FAILURE
                }
            }
        }
        Err(error) => {
            eprintln!("error: {}", error);
            ExitCode::FAILURE
        }
    }
}

fn path_source(path: String) -> Source {
    let path = PathBuf::from(&path);
    if path.is_dir() {
        Source::Folder(Folder {
            path,
            recursive: true,
            extensions: audio_extensions(),
        })
    } else {
        Source::Track(Track {
            path,
            len: None,
            segments: Vec::new(),
        })
    }
}

fn parse_secs(raw: &str) -> Option<Duration> {
    let seconds = raw.parse::<f64>().ok()?;
    if seconds.is_finite() && seconds >= 0.0 {
        Some(Duration::from_secs_f64(seconds))
    } else {
        None
    }
}

fn print_timeline(plays: &[Playable]) {
    for (index, play) in plays.iter().enumerate() {
        let end = play
            .end
            .map(fmt)
            .unwrap_or_else(|| "end".to_string());
        println!("{}. {} [{}..{}]", index, play.path.display(), fmt(play.start), end);
    }
}

fn fmt(duration: Duration) -> String {
    let total = duration.as_secs();
    format!("{}:{:02}", total / 60, total % 60)
}

fn audio_extensions() -> Vec<String> {
    ["mp3", "flac", "wav", "ogg", "opus", "m4a", "aac", "aiff", "wma"]
        .iter()
        .map(|ext| ext.to_string())
        .collect()
}
