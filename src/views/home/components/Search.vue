<script setup lang="ts">
import { ref, computed } from 'vue'
import XIcon from '@/components/ui/XIcon/index.vue'
import { getSystemApps, getInstalledApps, toAssetUrl } from '@/commands'
import type { AppInfo } from '@/commands'

const searchWord = ref('')
const showPopover = ref(false)
const wrapperRef = ref<HTMLElement | null>(null)
const popoverWidth = ref(0)

// ── 应用数据 ──────────────────────────────────────────────────────────────

const systemApps = ref<AppInfo[]>([])
const installedApps = ref<AppInfo[]>([])
const loaded = ref(false)

async function handleFocus() {
    if (wrapperRef.value) {
        popoverWidth.value = wrapperRef.value.offsetWidth
    }
    showPopover.value = true

    // 首次聚焦时拉取数据
    if (!loaded.value) {
        loaded.value = true
        const [sys, inst] = await Promise.all([
            getSystemApps().catch(() => [] as AppInfo[]),
            getInstalledApps().catch(() => [] as AppInfo[]),
        ])
        systemApps.value = sys
        installedApps.value = inst
    }
}

function handleBlur() {
    // 由 onPopoverMousedown 阻止关闭, 此处不做处理
}

function onPopoverMousedown(e: MouseEvent) {
    e.preventDefault()
}
</script>

<template>
    <div ref="wrapperRef" class="">
        <n-popover
            :show="showPopover"
            trigger="manual"
            placement="bottom"
            :width="popoverWidth"
            raw
        >
            <template #trigger>
                <n-input
                    v-model:value="searchWord"
                    type="text"
                    round
                    placeholder="请输入应用名称或关键字进行搜索"
                    @focus="handleFocus"
                    @blur="handleBlur"
                >
                    <template #prefix>
                        <XIcon name="Search" />
                    </template>
                    <template #suffix>
                        <XIcon :name="showPopover ? 'ChevronUp' : 'ChevronDown'" />
                    </template>
                </n-input>
            </template>

            <!-- popover 弹出层内容 -->
            <div @mousedown="onPopoverMousedown" class="bg-white rounded-lg overflow-hidden">
                <!-- 系统常用应用 -->
                <div v-if="systemApps.length" class="py-2">
                    <div class="text-xs text-n-text-3 px-4 py-1">系统应用</div>
                    <div
                        v-for="app in systemApps"
                        :key="app.name"
                        class="flex items-center gap-x-3 px-4 py-1.5 hover:bg-n-fill-2 cursor-pointer transition-colors"
                    >
                        <img
                            v-if="app.icon_path"
                            :src="toAssetUrl(app.icon_path)"
                            class="w-5 h-5 flex-shrink-0"
                            alt=""
                        />
                        <span class="text-sm text-n-text-2 truncate">{{ app.name }}</span>
                        <span class="text-xs text-n-text-3 truncate ml-auto">{{ app.path }}</span>
                    </div>
                </div>

                <!-- 已安装软件 -->
                <div v-if="installedApps.length" class="py-2 border-t">
                    <div class="text-xs text-n-text-3 px-4 py-1">已安装软件</div>
                    <div
                        v-for="app in installedApps"
                        :key="app.name"
                        class="flex items-center gap-x-3 px-4 py-1.5 hover:bg-n-fill-2 cursor-pointer transition-colors"
                    >
                        <img
                            v-if="app.icon_path"
                            :src="toAssetUrl(app.icon_path)"
                            class="w-5 h-5 flex-shrink-0"
                            alt=""
                        />
                        <span class="text-sm text-n-text-2 truncate">{{ app.name }}</span>
                        <span v-if="app.install_location" class="text-xs text-n-text-3 truncate ml-auto">
                            {{ app.install_location }}
                        </span>
                    </div>
                </div>

                <!-- 空态 -->
                <div v-if="loaded && !systemApps.length && !installedApps.length" class="py-8 text-center text-sm text-n-text-3">
                    暂无数据
                </div>
            </div>
        </n-popover>
    </div>
</template>
