import { invoke } from '@tauri-apps/api/core'
import { convertFileSrc } from '@tauri-apps/api/core'

// ── Types ────────────────────────────────────────────────────────────────

export interface AppInfo {
    name: string
    category: 'system' | 'installed'
    can_launch: boolean
    /** 已提取图标的本地缓存路径，配合 toAssetUrl() 转为可访问 URL */
    icon_path: string
    /** 用于按需提取图标的 PE 文件路径 */
    icon_source?: string | null
    /** PE 文件中的图标资源索引 */
    icon_index?: number | null
    /** 可确认的应用程序可执行文件路径 */
    path?: string | null
    /** 安装日期（注册表中的原始字符串） */
    install_date?: string | null
    /** 安装目录 */
    install_location?: string | null
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

/** 由后端根据应用分类和名称重新解析并启动，避免前端执行任意路径 */
export async function launchApp(app: AppInfo): Promise<void> {
    return invoke('launch_app', {
        category: app.category,
        name: app.name,
    })
}

// ── Helpers ──────────────────────────────────────────────────────────────

/** 按需提取单个应用图标 */
export async function extractAppIcon(
    iconSource: string,
    iconIndex?: number | null,
): Promise<string> {
    return invoke<string>('extract_app_icon', { iconSource, iconIndex })
}

/** 将本地文件路径转为 Tauri 可访问的资源 URL（用于 img src） */
export function toAssetUrl(filePath: string): string {
    if (!filePath) return ''
    return convertFileSrc(filePath)
}
