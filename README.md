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

## 使用

1. 在网易云音乐、Spotify 或 QQ 音乐等支持 Windows 系统媒体控制的客户端中开始播放。
2. 打开流光，主窗口会自动列出可控制的媒体会话。
3. 便捷条默认锁定并鼠标穿透。按 `Ctrl+Alt+L` 临时解锁控制，5 秒后自动恢复。
4. 按 `Ctrl+Alt+M` 显示或隐藏便捷条。
5. 解锁后拖动便捷条可调整位置；宽度可在设置中调整。

同步歌词来自 LRCLIB，并在本地缓存。匹配失败时可以在主窗口导入本地 `.lrc` 文件。

## 安装包

构建后的 NSIS 单用户安装包位于 `artifacts/流光_0.1.0_x64-setup.exe`。安装包未签名，Windows 首次运行时可能显示 SmartScreen 提示。

