use std::time::Duration;

use cahill_core as core;
use core::error::{Error, Result};
use core::resolver::Resolver;
use core::source::{Clip, List, Source, Track};
use core::timeline::{flatten, Playable};

use crate::probe::probe;

#[derive(Debug, Clone)]
pub struct Resolved {
    pub events: Vec<ResolvedEvent>,
    pub total: Duration,
}

#[derive(Debug, Clone)]
pub struct ResolvedEvent {
    pub at: Duration,
    pub play: Playable,
}

pub fn resolve(source: &Source, resolver: &dyn Resolver) -> Result<Resolved> {
    let probed = probed(source, resolver)?;
    let timeline = flatten(&probed, resolver)?;
    let mut total = Duration::ZERO;
    let mut events = Vec::new();
    for event in &timeline.events {
        let len = event.play.len().ok_or_else(|| Error::Other("unknown length".into()))?;
        events.push(ResolvedEvent {
            at: total,
            play: event.play.clone(),
        });
        total += len;
    }
    Ok(Resolved { events, total })
}

fn probed(source: &Source, resolver: &dyn Resolver) -> Result<Source> {
    match source {
        Source::Track(track) => {
            let len = track.len.or_else(|| probe(&track.path));
            Ok(Source::Track(Track {
                path: track.path.clone(),
                len,
                segments: track.segments.clone(),
            }))
        }
        Source::List(list) => {
            let children = list
                .children
                .iter()
                .map(|child| probed(child, resolver))
                .collect::<Result<Vec<_>>>()?;
            Ok(Source::List(List { children }))
        }
        Source::Folder(folder) => {
            let tracks = resolver
                .list(folder)?
                .into_iter()
                .map(|mut track| {
                    if track.len.is_none() {
                        track.len = probe(&track.path);
                    }
                    Source::Track(track)
                })
                .collect();
            Ok(Source::List(List { children: tracks }))
        }
        Source::Clip(clip) => {
            let inner = probed(&clip.inner, resolver)?;
            Ok(Source::Clip(Clip {
                inner: Box::new(inner),
                start: clip.start,
                end: clip.end,
            }))
        }
    }
}
