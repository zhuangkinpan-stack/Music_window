# 流光（Lumi）

Windows 音乐控制中枢。通过系统媒体会话（SMTC）统一控制支持 Windows 媒体控制的音乐软件，并提供低误触的桌面便捷条。

## 开发

```powershell
pnpm install
pnpm test
pnpm build
pnpm tauri dev
```

需要 Windows 10/11、WebView2、Rust stable 和 MSVC C++ Build Tools。

## 首版边界

- 不播放本地文件，不登录第三方账号。
- 只支持正确发布 SMTC 会话的音乐软件。
- 暂不提供播放队列、收藏、搜索或历史记录。

