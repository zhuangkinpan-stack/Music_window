//! 快捷方式自愈。
//!
//! 安装包按 `$LOCALAPPDATA` 解析安装目录，而 `$LOCALAPPDATA` 在某些机器上被
//! 重定向到非系统盘（本机就落在 F: 盘）。一旦程序被移动或盘符变化，
//! 桌面快捷方式与开机启动项都会指向不存在的路径。
//!
//! 应用启动时做一次低成本检查：
//! - 开机启动项由 `autostart` 模块在每次启动时按当前 exe 重写，天然自愈；
//! - 桌面快捷方式若指向的目标已不存在，则改回当前 exe（存在则不碰，尊重用户自定义）。

use std::{
    ffi::OsStr,
    os::windows::ffi::OsStrExt,
    path::{Path, PathBuf},
};

use ::windows::{
    core::{Interface, GUID, PCWSTR},
    Win32::{
        System::Com::{
            CoCreateInstance, CoInitializeEx, CoTaskMemFree, CoUninitialize, IPersistFile,
            CLSCTX_INPROC_SERVER, COINIT_APARTMENTTHREADED, STGM_READ,
        },
        UI::Shell::{FOLDERID_Desktop, IShellLinkW, SHGetKnownFolderPath, KF_FLAG_DEFAULT},
    },
};

const CLSID_SHELL_LINK: GUID = GUID::from_u128(0x00021401_0000_0000_c000_000000000046);

pub const SHORTCUT_NAME: &str = "流光.lnk";

/// 返回是否真的修复了快捷方式。任何失败都不影响应用运行。
pub fn repair_desktop_shortcut(shortcut_name: &str) -> bool {
    // 独立线程里自己初始化 COM，避免干扰 Tauri 主线程已建立的套间模型。
    let name = shortcut_name.to_owned();
    std::thread::spawn(move || repair_on_thread(&name))
        .join()
        .unwrap_or(false)
}

fn repair_on_thread(shortcut_name: &str) -> bool {
    let Ok(executable) = std::env::current_exe() else {
        return false;
    };
    let Some(desktop) = desktop_dir() else {
        return false;
    };
    let shortcut = desktop.join(shortcut_name);
    if !shortcut.is_file() {
        return false;
    }

    unsafe {
        if CoInitializeEx(None, COINIT_APARTMENTTHREADED).is_err() {
            return false;
        }
        let repaired = repair_inner(&shortcut, &executable).unwrap_or(false);
        CoUninitialize();
        repaired
    }
}

fn desktop_dir() -> Option<PathBuf> {
    unsafe {
        let raw = SHGetKnownFolderPath(&FOLDERID_Desktop, KF_FLAG_DEFAULT, None).ok()?;
        let path = String::from_utf16_lossy(raw.as_wide());
        CoTaskMemFree(Some(raw.0.cast()));
        (!path.is_empty()).then(|| PathBuf::from(path))
    }
}

unsafe fn repair_inner(shortcut: &Path, executable: &Path) -> ::windows::core::Result<bool> {
    unsafe {
        let link: IShellLinkW = CoCreateInstance(&CLSID_SHELL_LINK, None, CLSCTX_INPROC_SERVER)?;
        let persist: IPersistFile = link.cast()?;

        let shortcut_wide = wide(shortcut.as_os_str());
        persist.Load(PCWSTR(shortcut_wide.as_ptr()), STGM_READ)?;

        let mut buffer = [0u16; 1_024];
        link.GetPath(&mut buffer, std::ptr::null_mut(), 0)?;
        let current = wide_to_string(&buffer);

        if !shortcut_needs_repair(&current, executable) {
            return Ok(false);
        }

        let target = wide(executable.as_os_str());
        link.SetPath(PCWSTR(target.as_ptr()))?;
        if let Some(directory) = executable.parent() {
            let working = wide(directory.as_os_str());
            link.SetWorkingDirectory(PCWSTR(working.as_ptr()))?;
        }
        link.SetIconLocation(PCWSTR(target.as_ptr()), 0)?;
        persist.Save(PCWSTR(shortcut_wide.as_ptr()), true)?;
        Ok(true)
    }
}

fn wide(value: &OsStr) -> Vec<u16> {
    value.encode_wide().chain(std::iter::once(0)).collect()
}

fn wide_to_string(buffer: &[u16]) -> String {
    let end = buffer
        .iter()
        .position(|value| *value == 0)
        .unwrap_or(buffer.len());
    String::from_utf16_lossy(&buffer[..end])
}

fn same_path(left: &str, right: &Path) -> bool {
    let right = right.to_string_lossy();
    left.eq_ignore_ascii_case(&right)
}

fn shortcut_needs_repair(current_target: &str, executable: &Path) -> bool {
    !current_target.is_empty()
        && !same_path(current_target, executable)
        && !Path::new(current_target).exists()
}

#[cfg(test)]
mod tests {
    use std::fs;

    use tempfile::tempdir;

    use super::*;

    #[test]
    fn repairs_only_missing_non_current_targets() {
        let directory = tempdir().unwrap();
        let executable = directory.path().join("lumi.exe");
        let existing_target = directory.path().join("custom-player.exe");
        let missing_target = directory.path().join("missing-player.exe");
        fs::write(&executable, b"lumi").unwrap();
        fs::write(&existing_target, b"custom").unwrap();

        assert!(!shortcut_needs_repair("", &executable));
        assert!(!shortcut_needs_repair(
            &executable.to_string_lossy(),
            &executable
        ));
        assert!(!shortcut_needs_repair(
            &existing_target.to_string_lossy(),
            &executable
        ));
        assert!(shortcut_needs_repair(
            &missing_target.to_string_lossy(),
            &executable
        ));
    }
}
