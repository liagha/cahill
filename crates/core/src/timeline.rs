use std::path::{Path, PathBuf};
use std::time::Duration;

use crate::error::{Error, Result};
use crate::resolver::Resolver;
use crate::source::{Clip, List, Segment, Source, Track};

pub struct Timeline {
    pub events: Vec<Event>,
    pub total: Option<Duration>,
}

pub struct Event {
    pub play: Playable,
}

#[derive(Debug, Clone)]
pub struct Playable {
    pub path: PathBuf,
    pub start: Duration,
    pub end: Option<Duration>,
}

impl Playable {
    pub fn len(&self) -> Option<Duration> {
        self.end.map(|end| end - self.start)
    }
}

pub fn flatten(source: &Source, resolver: &dyn Resolver) -> Result<Timeline> {
    let (events, total) = inner(source, resolver)?;
    Ok(Timeline { events, total })
}

fn inner(source: &Source, resolver: &dyn Resolver) -> Result<(Vec<Event>, Option<Duration>)> {
    match source {
        Source::Track(track) => track_span(track),
        Source::List(list) => list_span(list, resolver),
        Source::Folder(folder) => {
            let tracks = resolver
                .list(folder)?
                .into_iter()
                .map(Source::Track)
                .collect::<Vec<_>>();
            list_span(&List { children: tracks }, resolver)
        }
        Source::Clip(clip) => clip_span(clip, resolver),
    }
}

fn track_span(track: &Track) -> Result<(Vec<Event>, Option<Duration>)> {
    if track.segments.is_empty() {
        let play = Playable {
            path: track.path.clone(),
            start: Duration::ZERO,
            end: track.len,
        };
        let total = play.len();
        return Ok((vec![Event { play }], total));
    }
    segments_span(&track.path, &track.segments)
}

fn segments_span(path: &Path, segments: &[Segment]) -> Result<(Vec<Event>, Option<Duration>)> {
    let mut events = Vec::new();
    let mut total = Duration::ZERO;
    for segment in segments {
        let len = segment.end.map(|end| end - segment.start);
        if segment.end.is_none() {
            events.push(Event {
                play: Playable {
                    path: path.to_path_buf(),
                    start: segment.start,
                    end: None,
                },
            });
            return Ok((events, None));
        }
        events.push(Event {
            play: Playable {
                path: path.to_path_buf(),
                start: segment.start,
                end: segment.end,
            },
        });
        total += len.unwrap();
    }
    Ok((events, Some(total)))
}

fn list_span(list: &List, resolver: &dyn Resolver) -> Result<(Vec<Event>, Option<Duration>)> {
    let mut events = Vec::new();
    let mut total = Some(Duration::ZERO);
    for child in &list.children {
        let (mut child_events, child_total) = inner(child, resolver)?;
        events.append(&mut child_events);
        match child_total {
            Some(len) => {
                if let Some(acc) = total.as_mut() {
                    *acc += len;
                }
            }
            None => total = None,
        }
    }
    Ok((events, total))
}

fn clip_span(clip: &Clip, resolver: &dyn Resolver) -> Result<(Vec<Event>, Option<Duration>)> {
    let (events, inner_total) = inner(&clip.inner, resolver)?;
    if clip.start > clip.end.unwrap_or(Duration::MAX) {
        return Err(Error::Reversed {
            start: clip.start,
            end: clip.end.unwrap_or(Duration::MAX),
        });
    }
    if inner_total.is_none() {
        return Err(Error::Clip {
            start: clip.start,
            end: clip.end,
        });
    }
    let timeline = Timeline { events, total: inner_total };
    let windowed = window(&timeline, clip.start, clip.end);
    let total = windowed
        .iter()
        .try_fold(Duration::ZERO, |acc, event| {
            event.play.len().map(|len| acc + len)
        });
    Ok((windowed, total))
}

fn window(timeline: &Timeline, start: Duration, end: Option<Duration>) -> Vec<Event> {
    let mut out = Vec::new();
    let mut pos = Duration::ZERO;
    for event in &timeline.events {
        let begin = pos;
        match event.play.len() {
            Some(len) => {
                let cease = begin + len;
                if cease <= start {
                    pos = cease;
                    continue;
                }
                if end.is_some_and(|end| begin >= end) {
                    break;
                }
                let lo = begin.max(start);
                let hi = end.map_or(cease, |end| cease.min(end));
                if hi > lo {
                    out.push(Event {
                        play: Playable {
                            path: event.play.path.clone(),
                            start: event.play.start + (lo - begin),
                            end: Some(event.play.start + (hi - begin)),
                        },
                    });
                }
                pos = cease;
            }
            None => {
                if end.is_some_and(|end| begin >= end) {
                    break;
                }
                let lo = begin.max(start);
                let offset = event.play.start + (lo - begin);
                out.push(Event {
                    play: Playable {
                        path: event.play.path.clone(),
                        start: offset,
                        end: end.map(|end| offset + (end - lo)),
                    },
                });
                break;
            }
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::resolver::Resolver;
    use crate::source::{Clip, Folder, List, Source, Track};

    struct Unit;

    impl Resolver for Unit {
        fn list(&self, _folder: &Folder) -> Result<Vec<Track>> {
            Ok(Vec::new())
        }
    }

    fn s(secs: u64) -> Duration {
        Duration::from_secs(secs)
    }

    fn track(name: &str, len: u64) -> Source {
        Source::Track(Track {
            path: PathBuf::from(name),
            len: Some(s(len)),
            segments: Vec::new(),
        })
    }

    fn names(timeline: &Timeline) -> Vec<String> {
        timeline
            .events
            .iter()
            .map(|event| {
                let end = event
                    .play
                    .end
                    .map(fmt)
                    .unwrap_or_else(|| "end".to_string());
                format!(
                    "{}[{}..{}]",
                    event.play.path.display(),
                    fmt(event.play.start),
                    end
                )
            })
            .collect()
    }

    fn fmt(duration: Duration) -> String {
        format!("{}s", duration.as_secs())
    }

    #[test]
    fn plain_track() {
        let timeline = flatten(&track("a", 10), &Unit).unwrap();
        assert_eq!(timeline.total, Some(s(10)));
        assert_eq!(names(&timeline), vec![r#"a[0s..10s]"#]);
    }

    #[test]
    fn segmented_track() {
        let src = Source::Track(Track {
            path: PathBuf::from("a"),
            len: None,
            segments: vec![
                crate::source::Segment { start: s(30), end: Some(s(40)) },
                crate::source::Segment { start: s(90), end: Some(s(100)) },
            ],
        });
        let timeline = flatten(&src, &Unit).unwrap();
        assert_eq!(timeline.total, Some(s(20)));
        assert_eq!(
            names(&timeline),
            vec![r#"a[30s..40s]"#, r#"a[90s..100s]"#]
        );
    }

    #[test]
    fn list_order() {
        let src = Source::List(List {
            children: vec![track("a", 10), track("b", 5)],
        });
        let timeline = flatten(&src, &Unit).unwrap();
        assert_eq!(timeline.total, Some(s(15)));
        assert_eq!(
            names(&timeline),
            vec![r#"a[0s..10s]"#, r#"b[0s..5s]"#]
        );
    }

    #[test]
    fn clip_over_segmented() {
        let src = Source::Track(Track {
            path: PathBuf::from("a"),
            len: None,
            segments: vec![
                crate::source::Segment { start: s(0), end: Some(s(40)) },
                crate::source::Segment { start: s(90), end: Some(s(100)) },
            ],
        });
        let clip = Source::Clip(Clip {
            inner: Box::new(src),
            start: s(5),
            end: Some(s(15)),
        });
        let timeline = flatten(&clip, &Unit).unwrap();
        assert_eq!(timeline.total, Some(s(10)));
        assert_eq!(names(&timeline), vec![r#"a[5s..15s]"#]);
    }

    #[test]
    fn clip_across_list() {
        let src = Source::List(List {
            children: vec![track("a", 10), track("b", 10)],
        });
        let clip = Source::Clip(Clip {
            inner: Box::new(src),
            start: s(8),
            end: Some(s(14)),
        });
        let timeline = flatten(&clip, &Unit).unwrap();
        assert_eq!(
            names(&timeline),
            vec![r#"a[8s..10s]"#, r#"b[0s..4s]"#]
        );
    }

    #[test]
    fn open_in_list_keeps_order() {
        let open = Source::Track(Track {
            path: PathBuf::from("a"),
            len: None,
            segments: Vec::new(),
        });
        let src = Source::List(List {
            children: vec![open, track("b", 5)],
        });
        let timeline = flatten(&src, &Unit).unwrap();
        assert_eq!(timeline.total, None);
        assert_eq!(names(&timeline), vec![r#"a[0s..end]"#, r#"b[0s..5s]"#]);
    }
}
