# Lumi Engineering Guide

## Product

流光（Lumi）是 Windows 10/11 音乐控制中枢。它通过 Windows 系统媒体会话控制已安装的音乐客户端，不实现本地音频播放或客户端专有适配。

## Commands

```powershell
pnpm install
pnpm test
pnpm build
cargo test --manifest-path src-tauri/Cargo.toml
cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets -- -D warnings
pnpm tauri build --bundles nsis
```

## Engineering Rules

- 所有业务行为先写失败测试，再写最小实现。
- 前端与 Rust 之间的数据边界使用 `src/contracts.ts` 与 `src-tauri/src/contracts.rs` 中的等价类型。
- SMTC 访问集中在 Rust 媒体模块，React 不直接调用 Windows API。
- LRCLIB 请求只由 Rust 后端发起，只发送标题、歌手、专辑和时长。
- 默认锁定便利条并启用鼠标穿透；只有显式解锁时允许交互或拖动。
- 不记录账号、播放历史或遥测。

## Agent skills

### Issue tracker

Issues live in GitHub Issues for `zhuangkinpan-stack/Music_window`. See `docs/agents/issue-tracker.md`.

### Triage labels

Use the default five-role vocabulary. See `docs/agents/triage-labels.md`.

### Domain docs

This is a single-context repository. See `docs/agents/domain.md`.

