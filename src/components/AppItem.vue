<script setup lang="ts">
import { computed, onUnmounted, ref, watch } from 'vue'
import XIcon from '@/components/ui/XIcon/index.vue'
import { launchApp, toAssetUrl } from '@/commands'
import type { AppInfo } from '@/commands'

const props = defineProps<{
    app: AppInfo
    layout?: 'list' | 'grid'
}>()

const emit = defineEmits<{
    opened: [app: AppInfo]
}>()

const opening = ref(false)
const openError = ref('')
const imageFailed = ref(false)
let errorTimer: ReturnType<typeof setTimeout> | null = null

const secondaryText = computed(
    () =>
        props.app.install_location ||
        props.app.path ||
        (props.app.category === 'system' ? 'Windows 系统应用' : '已安装软件'),
)

const title = computed(() => {
    if (openError.value) return openError.value
    if (!props.app.can_launch) return '未找到可用的启动文件'
    return `打开 ${props.app.name}`
})

async function handleOpen() {
    if (!props.app.can_launch || opening.value) return

    opening.value = true
    openError.value = ''
    try {
        await launchApp(props.app)
        emit('opened', props.app)
    } catch (error) {
        openError.value = String(error)
        if (errorTimer) clearTimeout(errorTimer)
        errorTimer = setTimeout(() => {
            openError.value = ''
        }, 4000)
    } finally {
        opening.value = false
    }
}

watch(
    () => props.app.icon_path,
    () => {
        imageFailed.value = false
    },
)

onUnmounted(() => {
    if (errorTimer) clearTimeout(errorTimer)
})
</script>

<template>
    <div
        class="min-w-0 rounded-lg border p-2"
        :class="[
            layout === 'grid'
                ? 'flex flex-col items-center gap-2 text-center'
                : 'flex items-center gap-3',
            app.can_launch && !opening ? 'cursor-pointer hover:bg-slate-100' : 'opacity-50',
        ]"
        :tabindex="app.can_launch && !opening ? 0 : -1" :aria-disabled="!app.can_launch || opening" :title="title"
        @click="handleOpen" @keydown.enter="handleOpen" @keydown.space.prevent="handleOpen">
        <div
            class="grid shrink-0 place-items-center overflow-hidden rounded-lg"
            :class="layout === 'grid' ? 'size-12' : 'size-10'"
        >
            <img
                v-if="app.icon_path && !imageFailed"
                :src="toAssetUrl(app.icon_path)"
                class="object-contain"
                :class="layout === 'grid' ? 'size-12' : 'size-10'"
                alt="" @error="imageFailed = true" />
            <XIcon v-else name="AppWindow" :size="layout === 'grid' ? 28 : 20" />
        </div>

        <div class="min-w-0" :class="layout === 'grid' ? 'w-full' : ''">
            <div class="truncate text-sm" :class="layout === 'grid' ? 'px-1' : ''">
                {{ app.name }}
            </div>
            <div
                v-if="layout !== 'grid'"
                class="mt-1 truncate text-xs"
                :class="openError ? 'text-red-600' : 'text-slate-400'"
            >
                {{ openError || secondaryText }}
            </div>
        </div>
    </div>
</template>
