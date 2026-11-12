use std::sync::OnceLock;

use regex::Regex;
use thiserror::Error;

use crate::contracts::LyricLine;

#[derive(Clone, Debug, Error, Eq, PartialEq)]
pub enum LyricsParseError {
    #[error("LRC 中没有带时间标签的歌词")]
    NoTimedLines,
}

pub fn parse_lrc(input: &str) -> Result<Vec<LyricLine>, LyricsParseError> {
    let mut offset_ms = 0i64;
    let mut lines = Vec::new();
    let timestamp_pattern = timestamp_pattern();
    let offset_pattern = offset_pattern();

    for raw_line in input.lines() {
        let line = raw_line.trim();
        if line.is_empty() {
            continue;
        }
        if let Some(captures) = offset_pattern.captures(line) {
            offset_ms = captures
                .get(1)
                .and_then(|value| value.as_str().parse().ok())
                .unwrap_or(0);
            continue;
        }

        let mut cursor = 0;
        let mut timestamps = Vec::new();
        for captures in timestamp_pattern.captures_iter(line) {
            let matched = captures.get(0).expect("full timestamp capture");
            if matched.start() != cursor {
                break;
            }
            cursor = matched.end();
            while line.as_bytes().get(cursor) == Some(&b' ') {
                cursor += 1;
            }
            let minutes: i64 = captures
                .get(1)
                .expect("minute capture")
                .as_str()
                .parse()
                .unwrap_or(0);
            let seconds: i64 = captures
                .get(2)
                .expect("second capture")
                .as_str()
                .parse()
                .unwrap_or(0);
            let fraction_ms = captures
                .get(3)
                .map(|fraction| parse_fraction_ms(fraction.as_str()))
                .unwrap_or(0);
            timestamps.push(minutes * 60_000 + seconds * 1_000 + fraction_ms);
        }

        if timestamps.is_empty() {
            continue;
        }
        let text = line[cursor..].trim().to_owned();
        for timestamp in timestamps {
            lines.push(LyricLine {
                time_ms: (timestamp + offset_ms).max(0),
                text: text.clone(),
            });
        }
    }

    if lines.is_empty() {
        return Err(LyricsParseError::NoTimedLines);
    }
    lines.sort_by_key(|line| line.time_ms);
    Ok(lines)
}

fn parse_fraction_ms(value: &str) -> i64 {
    match value.len() {
        1 => value.parse::<i64>().unwrap_or(0) * 100,
        2 => value.parse::<i64>().unwrap_or(0) * 10,
        _ => value.parse::<i64>().unwrap_or(0),
    }
}

fn timestamp_pattern() -> &'static Regex {
    static PATTERN: OnceLock<Regex> = OnceLock::new();
    PATTERN.get_or_init(|| {
        Regex::new(r"\[(\d{1,3}):(\d{2})(?:[.:](\d{1,3}))?\]").expect("valid LRC timestamp regex")
    })
}

fn offset_pattern() -> &'static Regex {
    static PATTERN: OnceLock<Regex> = OnceLock::new();
    PATTERN.get_or_init(|| {
        Regex::new(r"(?i)^\[offset:\s*([+-]?\d+)\s*\]$").expect("valid LRC offset regex")
    })
}
