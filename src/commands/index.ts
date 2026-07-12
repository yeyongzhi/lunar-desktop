import { invoke } from '@tauri-apps/api/core'
import { convertFileSrc } from '@tauri-apps/api/core'

// ── Types ────────────────────────────────────────────────────────────────

export interface AppInfo {
    name: string
    /** 图标文件的本地路径，配合 toAssetUrl() 转为可访问 URL */
    icon_path: string
    /** 可执行文件路径（用于拉起应用） */
    path?: string
    /** 安装日期（注册表中的原始字符串） */
    install_date?: string
    /** 安装目录 */
    install_location?: string
}

// ── Commands ─────────────────────────────────────────────────────────────

/** 获取系统常用应用：此电脑、回收站、计算器、控制面板等 */
export async function getSystemApps(): Promise<AppInfo[]> {
    return invoke<AppInfo[]>('get_system_apps')
}

/** 获取注册表已安装的软件列表：微信、钉钉等 */
export async function getInstalledApps(): Promise<AppInfo[]> {
    return invoke<AppInfo[]>('get_installed_apps')
}

// ── Helpers ──────────────────────────────────────────────────────────────

/** 将本地文件路径转为 Tauri 可访问的资源 URL（用于 img src） */
export function toAssetUrl(filePath: string): string {
    if (!filePath) return ''
    return convertFileSrc(filePath)
}
