use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::process::Command;
use winreg::enums::*;
use winreg::RegKey;

// ── Data Types ──────────────────────────────────────────────────────────

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AppInfo {
    pub name: String,
    /// Local file path to the extracted icon PNG.
    /// Convert to an asset URL on the frontend with `convertFileSrc()`.
    pub icon_path: String,
    /// Executable path (for launching).
    pub path: Option<String>,
    /// Installation date string from the registry.
    pub install_date: Option<String>,
    /// Installation directory.
    pub install_location: Option<String>,
}

// ── Icon Extraction ─────────────────────────────────────────────────────

/// Extract the associated icon from an executable and save as PNG.
/// Returns the path to the cached PNG file.
fn extract_associated_icon(app_handle: &tauri::AppHandle, exe_path: &str) -> Option<String> {
    let icon_dir = app_handle.path().app_cache_dir().ok()?.join("icons");
    std::fs::create_dir_all(&icon_dir).ok()?;

    let safe_name = sanitise_filename(exe_path);
    let icon_file = icon_dir.join(format!("{}.png", safe_name));
    let icon_path_str = icon_file.to_string_lossy().to_string();

    // Use cached icon if already extracted
    if icon_file.exists() {
        return Some(icon_path_str);
    }

    let script = format!(
        r#"Add-Type -AssemblyName System.Drawing
$icon = [System.Drawing.Icon]::ExtractAssociatedIcon('{}')
if ($icon) {{
    $bmp = $icon.ToBitmap()
    $bmp.Save('{}', [System.Drawing.Imaging.ImageFormat]::Png)
    $icon.Dispose()
    $bmp.Dispose()
}}"#,
        escape_ps(exe_path),
        escape_ps(&icon_path_str),
    );

    run_powershell(&script)?;
    icon_file.exists().then_some(icon_path_str)
}

/// Extract an icon from a DLL at a specific resource index.
fn extract_dll_icon(app_handle: &tauri::AppHandle, dll_path: &str, index: i32) -> Option<String> {
    let icon_dir = app_handle.path().app_cache_dir().ok()?.join("icons");
    std::fs::create_dir_all(&icon_dir).ok()?;

    let safe_name = sanitise_filename(&format!("{}_{}", dll_path, index));
    let icon_file = icon_dir.join(format!("{}.png", safe_name));
    let icon_path_str = icon_file.to_string_lossy().to_string();

    if icon_file.exists() {
        return Some(icon_path_str);
    }

    let script = format!(
        r#"Add-Type -TypeDefinition @"
using System;
using System.Drawing;
using System.Runtime.InteropServices;
public class IconExtractor {{
    [DllImport("shell32.dll", CharSet = CharSet.Auto)]
    static extern IntPtr ExtractIconEx(string file, int idx, IntPtr[] large, IntPtr[] small, uint count);
    [DllImport("user32.dll")]
    static extern bool DestroyIcon(IntPtr h);
    public static void Extract(string file, int idx, string outPath) {{
        IntPtr[] large = new IntPtr[1], small = new IntPtr[1];
        ExtractIconEx(file, idx, large, small, 1);
        if (large[0] != IntPtr.Zero) {{
            using (var icon = Icon.FromHandle(large[0]))
            using (var bmp = icon.ToBitmap())
                bmp.Save(outPath, System.Drawing.Imaging.ImageFormat.Png);
            DestroyIcon(large[0]);
        }}
    }}
}}
"@
[IconExtractor]::Extract('{dll}', {idx}, '{out}')
"#,
        dll = escape_ps(dll_path),
        idx = index,
        out = escape_ps(&icon_path_str),
    );

    run_powershell(&script)?;
    icon_file.exists().then_some(icon_path_str)
}

fn run_powershell(script: &str) -> Option<()> {
    let output = Command::new("powershell")
        .args(["-NoProfile", "-NonInteractive", "-Command", script])
        .output()
        .ok()?;
    output.status.success().then_some(())
}

fn escape_ps(s: &str) -> String {
    s.replace('\'', "''")
}

fn sanitise_filename(s: &str) -> String {
    s.replace(['\\', '/', ':', '*', '?', '"', '<', '>', '|', ' '], "_")
}

// ── System Apps ──────────────────────────────────────────────────────────

struct SysAppDef {
    name: &'static str,
    /// Icon source: executable or DLL
    icon_source: &'static str,
    /// If Some, extract icon from DLL at this resource index instead of using ExtractAssociatedIcon
    dll_index: Option<i32>,
    /// Path for launching (None for shell namespace objects like Recycle Bin)
    exec_path: Option<&'static str>,
}

const SYSTEM_APPS: &[SysAppDef] = &[
    SysAppDef {
        name: "此电脑",
        icon_source: "explorer.exe",
        dll_index: None,
        exec_path: Some("explorer.exe"),
    },
    SysAppDef {
        name: "回收站",
        // Recycle Bin icon lives in imageres.dll at index 55 (empty) / 56 (full)
        icon_source: r"C:\Windows\System32\imageres.dll",
        dll_index: Some(55),
        exec_path: Some("explorer.exe"),
    },
    SysAppDef {
        name: "计算器",
        icon_source: "calc.exe",
        dll_index: None,
        exec_path: Some("calc.exe"),
    },
    SysAppDef {
        name: "控制面板",
        icon_source: "control.exe",
        dll_index: None,
        exec_path: Some("control.exe"),
    },
    SysAppDef {
        name: "记事本",
        icon_source: "notepad.exe",
        dll_index: None,
        exec_path: Some("notepad.exe"),
    },
    SysAppDef {
        name: "命令提示符",
        icon_source: "cmd.exe",
        dll_index: None,
        exec_path: Some("cmd.exe"),
    },
    SysAppDef {
        name: "任务管理器",
        icon_source: "Taskmgr.exe",
        dll_index: None,
        exec_path: Some("Taskmgr.exe"),
    },
    SysAppDef {
        name: "截图工具",
        icon_source: "SnippingTool.exe",
        dll_index: None,
        exec_path: Some("SnippingTool.exe"),
    },
    SysAppDef {
        name: "Windows 设置",
        icon_source: r"C:\Windows\System32\imageres.dll",
        dll_index: Some(114), // settings gear icon
        exec_path: Some("ms-settings:"),
    },
];

#[tauri::command]
fn get_system_apps(app_handle: tauri::AppHandle) -> Vec<AppInfo> {
    SYSTEM_APPS
        .iter()
        .map(|def| {
            let icon_path = match def.dll_index {
                Some(idx) => extract_dll_icon(&app_handle, def.icon_source, idx),
                None => extract_associated_icon(&app_handle, def.icon_source),
            }
            .unwrap_or_default();

            AppInfo {
                name: def.name.to_string(),
                icon_path,
                path: def.exec_path.map(String::from),
                install_date: None,
                install_location: None,
            }
        })
        .collect()
}

// ── Installed Apps (Registry) ────────────────────────────────────────────

#[tauri::command]
fn get_installed_apps(app_handle: tauri::AppHandle) -> Vec<AppInfo> {
    let mut apps: Vec<AppInfo> = Vec::new();

    // Read from both 64-bit and 32-bit uninstall registry views
    let uninstall_paths = &[
        r"SOFTWARE\Microsoft\Windows\CurrentVersion\Uninstall",
        r"SOFTWARE\WOW6432Node\Microsoft\Windows\CurrentVersion\Uninstall",
    ];

    // Also check current-user installs
    let hklm = RegKey::predef(HKEY_LOCAL_MACHINE);
    let hkcu = RegKey::predef(HKEY_CURRENT_USER);

    for &reg_path in uninstall_paths {
        read_uninstall_key(&hklm, reg_path, &app_handle, &mut apps);
    }
    // Current user uninstall entries
    read_uninstall_key(
        &hkcu,
        r"SOFTWARE\Microsoft\Windows\CurrentVersion\Uninstall",
        &app_handle,
        &mut apps,
    );

    // Deduplicate by name
    apps.sort_by(|a, b| a.name.cmp(&b.name));
    apps.dedup_by(|a, b| a.name.eq_ignore_ascii_case(&b.name));

    apps
}

fn read_uninstall_key(
    hive: &RegKey,
    path: &str,
    app_handle: &tauri::AppHandle,
    apps: &mut Vec<AppInfo>,
) {
    let key = match hive.open_subkey_with_flags(path, KEY_READ) {
        Ok(k) => k,
        Err(_) => return,
    };

    for subkey_name in key.enum_keys().filter_map(|k| k.ok()) {
        let subkey = match key.open_subkey_with_flags(&subkey_name, KEY_READ) {
            Ok(k) => k,
            Err(_) => continue,
        };

        // Read DisplayName
        let name: String = match subkey.get_value("DisplayName") {
            Ok(n) => n,
            Err(_) => continue,
        };

        // Skip system components and updates
        if name.is_empty()
            || name.contains("Update for Microsoft")
            || name.contains("Security Update")
            || name.contains("Hotfix")
            || name.contains("Service Pack")
        {
            continue;
        }

        // Read optional fields
        let display_icon: Option<String> = subkey.get_value("DisplayIcon").ok();
        let install_date: Option<String> = subkey.get_value("InstallDate").ok();
        let install_location: Option<String> = subkey.get_value("InstallLocation").ok();

        // Determine icon source: prefer DisplayIcon, fall back to executable path
        let icon_source = display_icon
            .as_deref()
            .unwrap_or("")
            .trim_end_matches(",0")
            .to_string();

        let icon_path = if !icon_source.is_empty() {
            extract_associated_icon(app_handle, &icon_source)
        } else {
            None
        }
        .unwrap_or_default();

        apps.push(AppInfo {
            name,
            icon_path,
            path: Some(icon_source),
            install_date,
            install_location,
        });
    }
}

// ── Tauri Entry Point ────────────────────────────────────────────────────

#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            greet,
            get_system_apps,
            get_installed_apps
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
