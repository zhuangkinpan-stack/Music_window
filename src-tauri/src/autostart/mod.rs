use std::path::Path;

use windows_registry::{CURRENT_USER, LOCAL_MACHINE};

const RUN_KEY: &str = r"Software\Microsoft\Windows\CurrentVersion\Run";
const VALUE_NAME: &str = "Lumi";

pub fn set_enabled(enabled: bool) -> Result<(), String> {
    let executable =
        std::env::current_exe().map_err(|error| format!("无法定位 Lumi 可执行文件：{error}"))?;
    set_enabled_for_executable(enabled, &executable)
}

fn set_enabled_for_executable(enabled: bool, executable: &Path) -> Result<(), String> {
    if enabled {
        if !executable.is_absolute() || !executable.exists() {
            return Err(format!("Lumi 可执行文件不存在：{}", executable.display()));
        }
        CURRENT_USER
            .create(RUN_KEY)
            .and_then(|key| key.set_string(VALUE_NAME, run_value(executable)))
            .map_err(|error| format!("写入开机启动项失败：{error}"))?;
        remove_value(LOCAL_MACHINE, true)?;
    } else {
        remove_value(CURRENT_USER, false)?;
        remove_value(LOCAL_MACHINE, false)?;
    }
    Ok(())
}

fn remove_value(root: &windows_registry::Key, tolerate_access_denied: bool) -> Result<(), String> {
    match root
        .options()
        .read()
        .write()
        .open(RUN_KEY)
        .and_then(|key| key.remove_value(VALUE_NAME))
    {
        Ok(()) => Ok(()),
        Err(error) if error.code().0 as u32 & 0xffff == 2 => Ok(()),
        Err(error) if tolerate_access_denied && matches!(error.code().0 as u32 & 0xffff, 5 | 6) => {
            Ok(())
        }
        Err(error) => Err(format!("删除开机启动项失败：{error}")),
    }
}

fn run_value(executable: &Path) -> String {
    format!("\"{}\"", executable.display())
}

#[cfg(test)]
mod tests;
