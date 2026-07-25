use serde::Serialize;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::UNIX_EPOCH;
use tauri::Manager;
use winreg::enums::*;
use winreg::RegKey;

#[derive(Debug, Serialize, Clone)]
pub struct AppInfo {
    pub name: String,
    /// 应用来源，用于后端重新解析启动目标。
    pub category: String,
    pub can_launch: bool,
    /// 已生成的缓存图标路径。为空时由前端按需调用 extract_app_icon。
    pub icon_path: String,
    /// 用于提取图标的 PE 文件路径。
    pub icon_source: Option<String>,
    /// PE 资源索引；None 时提取文件关联图标。
    pub icon_index: Option<i32>,
    /// 可确认的应用程序可执行文件路径。
    pub path: Option<String>,
    pub install_date: Option<String>,
    pub install_location: Option<String>,
}

fn icon_output_path(
    app_handle: &tauri::AppHandle,
    source: &str,
    index: Option<i32>,
) -> Option<PathBuf> {
    let icon_dir = app_handle.path().app_cache_dir().ok()?.join("icons");
    std::fs::create_dir_all(&icon_dir).ok()?;
    Some(icon_dir.join(format!("{:016x}.png", icon_cache_key(source, index))))
}

fn cached_icon_path(
    app_handle: &tauri::AppHandle,
    source: &str,
    index: Option<i32>,
) -> Option<String> {
    let path = icon_output_path(app_handle, source, index)?;
    path.exists().then(|| path.to_string_lossy().into_owned())
}

fn icon_cache_key(source: &str, index: Option<i32>) -> u64 {
    // FNV-1a 避免直接把完整路径作为文件名导致冲突或超过 Windows 长度限制。
    let mut hash = 0xcbf29ce484222325_u64;
    let mut input = format!("{}|{}", source.to_lowercase(), index.unwrap_or(i32::MIN));

    // 文件更新后使用新的缓存键，避免长期显示旧图标。
    if let Ok(metadata) = std::fs::metadata(source) {
        input.push_str(&format!("|{}", metadata.len()));
        if let Ok(modified) = metadata.modified().and_then(|time| {
            time.duration_since(UNIX_EPOCH)
                .map_err(std::io::Error::other)
        }) {
            input.push_str(&format!("|{}", modified.as_nanos()));
        }
    }

    for byte in input.bytes() {
        hash ^= u64::from(byte);
        hash = hash.wrapping_mul(0x100000001b3);
    }
    hash
}

fn extract_associated_icon(app_handle: &tauri::AppHandle, exe_path: &str) -> Option<String> {
    let icon_file = icon_output_path(app_handle, exe_path, None)?;
    let icon_path = icon_file.to_string_lossy().into_owned();
    if icon_file.exists() {
        return Some(icon_path);
    }

    let script = format!(
        r#"Add-Type -AssemblyName System.Drawing
$icon = [System.Drawing.Icon]::ExtractAssociatedIcon('{source}')
if ($icon) {{
    $bitmap = $icon.ToBitmap()
    $bitmap.Save('{output}', [System.Drawing.Imaging.ImageFormat]::Png)
    $icon.Dispose()
    $bitmap.Dispose()
}}"#,
        source = escape_ps(exe_path),
        output = escape_ps(&icon_path),
    );

    run_powershell(&script)?;
    icon_file.exists().then_some(icon_path)
}

fn extract_indexed_icon(app_handle: &tauri::AppHandle, source: &str, index: i32) -> Option<String> {
    let icon_file = icon_output_path(app_handle, source, Some(index))?;
    let icon_path = icon_file.to_string_lossy().into_owned();
    if icon_file.exists() {
        return Some(icon_path);
    }

    let script = format!(
        r#"Add-Type -TypeDefinition @"
using System;
using System.Drawing;
using System.Runtime.InteropServices;
public class IconExtractor {{
    [DllImport("shell32.dll", CharSet = CharSet.Auto)]
    static extern uint ExtractIconEx(string file, int index, IntPtr[] large, IntPtr[] small, uint count);
    [DllImport("user32.dll")]
    static extern bool DestroyIcon(IntPtr handle);

    public static void Extract(string file, int index, string output) {{
        IntPtr[] large = new IntPtr[1], small = new IntPtr[1];
        ExtractIconEx(file, index, large, small, 1);
        IntPtr selected = large[0] != IntPtr.Zero ? large[0] : small[0];
        if (selected != IntPtr.Zero) {{
            using (var icon = Icon.FromHandle(selected))
            using (var bitmap = icon.ToBitmap())
                bitmap.Save(output, System.Drawing.Imaging.ImageFormat.Png);
        }}
        if (large[0] != IntPtr.Zero) DestroyIcon(large[0]);
        if (small[0] != IntPtr.Zero) DestroyIcon(small[0]);
    }}
}}
"@
[IconExtractor]::Extract('{source}', {index}, '{output}')
"#,
        source = escape_ps(source),
        output = escape_ps(&icon_path),
    );

    run_powershell(&script)?;
    icon_file.exists().then_some(icon_path)
}

fn run_powershell(script: &str) -> Option<()> {
    let executable = windows_dir().join(r"System32\WindowsPowerShell\v1.0\powershell.exe");
    let executable = if executable.exists() {
        executable
    } else {
        PathBuf::from("powershell")
    };

    let output = Command::new(executable)
        .args(["-NoProfile", "-NonInteractive", "-Command", script])
        .output()
        .ok()?;
    output.status.success().then_some(())
}

fn escape_ps(value: &str) -> String {
    value.replace('\'', "''")
}

fn windows_dir() -> PathBuf {
    std::env::var_os("WINDIR")
        .or_else(|| std::env::var_os("SystemRoot"))
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from(r"C:\Windows"))
}

struct SysAppDef {
    name: &'static str,
    icon_relative_path: &'static str,
    icon_index: Option<i32>,
    launch_program: &'static str,
    launch_args: &'static [&'static str],
}

const SYSTEM_APPS: &[SysAppDef] = &[
    SysAppDef {
        name: "此电脑",
        icon_relative_path: "explorer.exe",
        icon_index: None,
        launch_program: "explorer.exe",
        launch_args: &["shell:MyComputerFolder"],
    },
    SysAppDef {
        name: "回收站",
        icon_relative_path: r"System32\imageres.dll",
        icon_index: Some(55),
        launch_program: "explorer.exe",
        launch_args: &["shell:RecycleBinFolder"],
    },
    SysAppDef {
        name: "计算器",
        icon_relative_path: r"System32\calc.exe",
        icon_index: None,
        launch_program: "calc.exe",
        launch_args: &[],
    },
    SysAppDef {
        name: "控制面板",
        icon_relative_path: r"System32\control.exe",
        icon_index: None,
        launch_program: "control.exe",
        launch_args: &[],
    },
    SysAppDef {
        name: "记事本",
        icon_relative_path: r"System32\notepad.exe",
        icon_index: None,
        launch_program: "notepad.exe",
        launch_args: &[],
    },
    SysAppDef {
        name: "命令提示符",
        icon_relative_path: r"System32\cmd.exe",
        icon_index: None,
        launch_program: "cmd.exe",
        launch_args: &[],
    },
    SysAppDef {
        name: "任务管理器",
        icon_relative_path: r"System32\Taskmgr.exe",
        icon_index: None,
        launch_program: "Taskmgr.exe",
        launch_args: &[],
    },
    SysAppDef {
        name: "截图工具",
        icon_relative_path: r"System32\SnippingTool.exe",
        icon_index: None,
        launch_program: "SnippingTool.exe",
        launch_args: &[],
    },
    SysAppDef {
        name: "Windows 设置",
        icon_relative_path: r"System32\imageres.dll",
        icon_index: Some(114),
        launch_program: "explorer.exe",
        launch_args: &["ms-settings:"],
    },
];

#[tauri::command]
fn get_system_apps(app_handle: tauri::AppHandle) -> Vec<AppInfo> {
    SYSTEM_APPS
        .iter()
        .map(|definition| {
            let icon_source = windows_dir()
                .join(definition.icon_relative_path)
                .to_string_lossy()
                .into_owned();

            AppInfo {
                name: definition.name.to_string(),
                category: "system".to_string(),
                can_launch: true,
                icon_path: cached_icon_path(&app_handle, &icon_source, definition.icon_index)
                    .unwrap_or_default(),
                icon_source: Some(icon_source),
                icon_index: definition.icon_index,
                path: Some(definition.launch_program.to_string()),
                install_date: None,
                install_location: None,
            }
        })
        .collect()
}

struct RegistryEntry {
    name: String,
    icon_source: Option<String>,
    icon_index: Option<i32>,
    launch_path: Option<String>,
    install_date: Option<String>,
    install_location: Option<String>,
}

fn collect_registry_entries() -> Vec<RegistryEntry> {
    let mut entries = Vec::new();
    let uninstall_paths = [
        r"SOFTWARE\Microsoft\Windows\CurrentVersion\Uninstall",
        r"SOFTWARE\WOW6432Node\Microsoft\Windows\CurrentVersion\Uninstall",
    ];

    let hklm = RegKey::predef(HKEY_LOCAL_MACHINE);
    let hkcu = RegKey::predef(HKEY_CURRENT_USER);

    for path in uninstall_paths {
        read_uninstall_entries(&hklm, path, &mut entries);
    }
    read_uninstall_entries(
        &hkcu,
        r"SOFTWARE\Microsoft\Windows\CurrentVersion\Uninstall",
        &mut entries,
    );

    entries.sort_by_cached_key(|entry| entry.name.to_lowercase());
    entries.dedup_by(|left, right| left.name.eq_ignore_ascii_case(&right.name));
    entries
}

fn read_uninstall_entries(hive: &RegKey, path: &str, entries: &mut Vec<RegistryEntry>) {
    let key = match hive.open_subkey_with_flags(path, KEY_READ) {
        Ok(key) => key,
        Err(_) => return,
    };

    for subkey_name in key.enum_keys().filter_map(Result::ok) {
        let subkey = match key.open_subkey_with_flags(&subkey_name, KEY_READ) {
            Ok(subkey) => subkey,
            Err(_) => continue,
        };

        let name: String = match subkey.get_value("DisplayName") {
            Ok(name) => name,
            Err(_) => continue,
        };
        let system_component: u32 = subkey.get_value("SystemComponent").unwrap_or_default();
        if name.trim().is_empty()
            || system_component == 1
            || name.contains("Update for Microsoft")
            || name.contains("Security Update")
            || name.contains("Hotfix")
            || name.contains("Service Pack")
        {
            continue;
        }

        let display_icon: Option<String> = subkey.get_value("DisplayIcon").ok();
        let (icon_source, icon_index) = display_icon
            .as_deref()
            .map(parse_display_icon)
            .unwrap_or((None, None));
        let launch_path = icon_source
            .as_deref()
            .filter(|source| is_executable_file(source))
            .map(String::from);

        entries.push(RegistryEntry {
            name,
            icon_source,
            icon_index,
            launch_path,
            install_date: subkey.get_value("InstallDate").ok(),
            install_location: subkey.get_value("InstallLocation").ok(),
        });
    }
}

fn parse_display_icon(value: &str) -> (Option<String>, Option<i32>) {
    let expanded = expand_windows_env(value.trim());
    let (path, index) = if let Some(quoted) = expanded.strip_prefix('"') {
        match quoted.find('"') {
            Some(end) => {
                let path = &quoted[..end];
                let suffix = quoted[end + 1..].trim();
                let index = suffix
                    .strip_prefix(',')
                    .and_then(|value| value.trim().parse().ok());
                (path, index)
            }
            None => (expanded.trim_matches('"'), None),
        }
    } else if let Some((path, suffix)) = expanded.rsplit_once(',') {
        match suffix.trim().parse::<i32>() {
            Ok(index) => (path.trim(), Some(index)),
            Err(_) => (expanded.as_str(), None),
        }
    } else {
        (expanded.as_str(), None)
    };

    let path = path.trim().trim_matches('"');
    ((!path.is_empty()).then(|| path.to_string()), index)
}

fn expand_windows_env(value: &str) -> String {
    let mut output = String::with_capacity(value.len());
    let mut remaining = value;

    while let Some(start) = remaining.find('%') {
        output.push_str(&remaining[..start]);
        let after_start = &remaining[start + 1..];
        let Some(end) = after_start.find('%') else {
            output.push_str(&remaining[start..]);
            return output;
        };

        let variable = &after_start[..end];
        let replacement = std::env::vars_os().find_map(|(key, value)| {
            key.to_string_lossy()
                .eq_ignore_ascii_case(variable)
                .then(|| value.to_string_lossy().into_owned())
        });

        match replacement {
            Some(value) => output.push_str(&value),
            None => output.push_str(&remaining[start..start + end + 2]),
        }
        remaining = &after_start[end + 1..];
    }

    output.push_str(remaining);
    output
}

fn is_executable_file(path: &str) -> bool {
    Path::new(path)
        .extension()
        .is_some_and(|extension| extension.eq_ignore_ascii_case("exe"))
        && Path::new(path).is_file()
}

#[tauri::command]
async fn get_installed_apps(app_handle: tauri::AppHandle) -> Result<Vec<AppInfo>, String> {
    tauri::async_runtime::spawn_blocking(move || {
        collect_registry_entries()
            .into_iter()
            .map(|entry| {
                let icon_path = entry
                    .icon_source
                    .as_deref()
                    .and_then(|source| cached_icon_path(&app_handle, source, entry.icon_index))
                    .unwrap_or_default();

                AppInfo {
                    name: entry.name,
                    category: "installed".to_string(),
                    can_launch: entry.launch_path.is_some(),
                    icon_path,
                    icon_source: entry.icon_source,
                    icon_index: entry.icon_index,
                    path: entry.launch_path,
                    install_date: entry.install_date,
                    install_location: entry.install_location,
                }
            })
            .collect()
    })
    .await
    .map_err(|error| format!("读取已安装应用失败: {error}"))
}

#[tauri::command]
async fn launch_app(category: String, name: String) -> Result<(), String> {
    tauri::async_runtime::spawn_blocking(move || match category.as_str() {
        "system" => {
            let definition = SYSTEM_APPS
                .iter()
                .find(|app| app.name.eq_ignore_ascii_case(&name))
                .ok_or_else(|| format!("未找到系统应用: {name}"))?;

            Command::new(definition.launch_program)
                .args(definition.launch_args)
                .spawn()
                .map(|_| ())
                .map_err(|error| format!("无法打开“{name}”: {error}"))
        }
        "installed" => {
            let entry = collect_registry_entries()
                .into_iter()
                .find(|app| app.name.eq_ignore_ascii_case(&name))
                .ok_or_else(|| format!("未找到已安装应用: {name}"))?;
            let path = entry
                .launch_path
                .ok_or_else(|| format!("“{name}”没有可用的启动路径"))?;

            Command::new(path)
                .spawn()
                .map(|_| ())
                .map_err(|error| format!("无法打开“{name}”: {error}"))
        }
        _ => Err(format!("不支持的应用分类: {category}")),
    })
    .await
    .map_err(|error| error.to_string())?
}

#[tauri::command]
async fn extract_app_icon(
    app_handle: tauri::AppHandle,
    icon_source: String,
    icon_index: Option<i32>,
) -> Result<String, String> {
    if !Path::new(&icon_source).is_file() {
        return Err(format!("图标源文件不存在: {icon_source}"));
    }

    tauri::async_runtime::spawn_blocking(move || {
        match icon_index {
            Some(index) => extract_indexed_icon(&app_handle, &icon_source, index),
            None => extract_associated_icon(&app_handle, &icon_source),
        }
        .ok_or_else(|| format!("无法提取图标: {icon_source}"))
    })
    .await
    .map_err(|error| error.to_string())?
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![
            get_system_apps,
            get_installed_apps,
            launch_app,
            extract_app_icon,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

#[cfg(test)]
mod tests {
    use super::{icon_cache_key, parse_display_icon};

    #[test]
    fn parses_quoted_display_icon_with_index() {
        let (path, index) = parse_display_icon(r#""C:\Apps, Inc\app.exe",-12"#);
        assert_eq!(path.as_deref(), Some(r"C:\Apps, Inc\app.exe"));
        assert_eq!(index, Some(-12));
    }

    #[test]
    fn parses_unquoted_display_icon() {
        let (path, index) = parse_display_icon(r"C:\Apps\app.exe,0");
        assert_eq!(path.as_deref(), Some(r"C:\Apps\app.exe"));
        assert_eq!(index, Some(0));
    }

    #[test]
    fn cache_key_includes_source_and_index() {
        assert_ne!(
            icon_cache_key(r"C:\Apps\app.exe", None),
            icon_cache_key(r"C:\Apps\app.exe", Some(0))
        );
        assert_ne!(
            icon_cache_key(r"C:\Apps\app.exe", None),
            icon_cache_key(r"C:\Other\app.exe", None)
        );
    }
}
