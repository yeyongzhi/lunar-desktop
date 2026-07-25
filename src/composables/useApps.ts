import { computed, ref } from 'vue'
import {
    extractAppIcon,
    getInstalledApps,
    getSystemApps,
} from '@/commands'
import type { AppInfo } from '@/commands'

const ICON_LOAD_CONCURRENCY = 4

const systemApps = ref<AppInfo[]>([])
const installedApps = ref<AppInfo[]>([])
const loading = ref(false)
const loaded = ref(false)
const loadError = ref('')

const iconQueue: AppInfo[] = []
const queuedIcons = new Set<string>()
const activeIcons = new Set<string>()
const failedIcons = new Set<string>()
let activeIconLoads = 0
let loadPromise: Promise<void> | null = null

const allApps = computed(() => [
    ...systemApps.value,
    ...installedApps.value,
])

function getIconKey(app: AppInfo) {
    return `${app.icon_source ?? ''}:${app.icon_index ?? ''}`
}

function setIconPath(key: string, iconPath: string) {
    for (const app of allApps.value) {
        if (getIconKey(app) === key) app.icon_path = iconPath
    }
}

function drainIconQueue() {
    while (activeIconLoads < ICON_LOAD_CONCURRENCY && iconQueue.length) {
        const app = iconQueue.shift()
        if (!app?.icon_source) continue

        const key = getIconKey(app)
        queuedIcons.delete(key)
        activeIcons.add(key)
        activeIconLoads += 1

        extractAppIcon(app.icon_source, app.icon_index)
            .then((iconPath) => setIconPath(key, iconPath))
            .catch((error) => {
                failedIcons.add(key)
                console.warn(`无法加载“${app.name}”的图标`, error)
            })
            .finally(() => {
                activeIconLoads -= 1
                activeIcons.delete(key)
                drainIconQueue()
            })
    }
}

function ensureIcons(apps: AppInfo[], prioritize = false) {
    const priorityKeys = new Set(apps.map(getIconKey))
    const additions: AppInfo[] = []

    for (const app of apps) {
        if (app.icon_path || !app.icon_source) continue
        const key = getIconKey(app)
        if (
            queuedIcons.has(key) ||
            activeIcons.has(key) ||
            failedIcons.has(key)
        ) {
            continue
        }
        queuedIcons.add(key)
        additions.push(app)
    }

    if (prioritize) {
        iconQueue.unshift(...additions)
        iconQueue.sort(
            (left, right) =>
                Number(priorityKeys.has(getIconKey(right))) -
                Number(priorityKeys.has(getIconKey(left))),
        )
    } else {
        iconQueue.push(...additions)
    }
    drainIconQueue()
}

async function fetchApps() {
    loading.value = true
    loadError.value = ''

    const [systemResult, installedResult] = await Promise.allSettled([
        getSystemApps(),
        getInstalledApps(),
    ])

    const errors: string[] = []
    if (systemResult.status === 'fulfilled') {
        systemApps.value = systemResult.value
    } else {
        errors.push('系统应用')
    }
    if (installedResult.status === 'fulfilled') {
        installedApps.value = installedResult.value
    } else {
        errors.push('已安装软件')
    }

    if (errors.length) {
        loadError.value = `${errors.join('、')}加载失败`
    }
    loaded.value = true
    loading.value = false
}

function loadApps() {
    if (loaded.value) return Promise.resolve()
    if (!loadPromise) {
        loadPromise = fetchApps().finally(() => {
            loadPromise = null
        })
    }
    return loadPromise
}

async function retryLoadApps() {
    loaded.value = false
    systemApps.value = []
    installedApps.value = []
    iconQueue.length = 0
    queuedIcons.clear()
    failedIcons.clear()
    await loadApps()
}

export function useApps() {
    return {
        systemApps,
        installedApps,
        allApps,
        loading,
        loaded,
        loadError,
        loadApps,
        retryLoadApps,
        ensureIcons,
    }
}
