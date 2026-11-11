# Lumi Glossary

## Media session

A Windows System Media Transport Controls session published by a music client. Sessions expose metadata, playback state, timeline information and capability flags.

## Media session provider

The Rust boundary that enumerates sessions and dispatches playback commands. The production implementation uses Windows SMTC; tests use an in-memory provider.

## Playback state

The selected session's metadata, playback status, duration, position and supported controls.

## Quick bar

The always-compact, always-on-top, draggable window near the taskbar. It shows artwork, title, one synchronized lyric line and previous/play/next controls.

## Interaction lock

The safety state that prevents the quick bar from receiving pointer input. Locked means mouse-transparent; unlocked means interactive for a bounded period.

## Lyric match

A synchronized LRCLIB result accepted only after title, artist and duration checks. A manually imported LRC always overrides an automatic match.

## Source app

The music client that owns a media session. Lumi displays a friendly name when known and otherwise uses a safe generic label.

