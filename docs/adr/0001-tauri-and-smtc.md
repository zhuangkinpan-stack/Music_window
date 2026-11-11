# ADR 0001: Tauri and Windows SMTC

## Status

Accepted

## Context

Lumi needs a low-overhead Windows desktop UI, two independently styled windows, tray integration, global hotkeys and direct access to Windows media sessions. The app must remain usable beside games and must not depend on client-specific automation.

## Decision

- Use Tauri 2 with a React/TypeScript renderer.
- Implement Windows media integration in Rust using the Windows SMTC APIs.
- Keep all SMTC and network access behind typed Tauri commands and events.
- Use the system media-session contract as the compatibility boundary.
- Do not add per-client adapters in version 1.

## Consequences

The application stays smaller than an Electron equivalent and can enforce strict permissions. Development requires Rust and MSVC tooling on Windows. Clients that do not publish SMTC sessions are explicitly unsupported.

