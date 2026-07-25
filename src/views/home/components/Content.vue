<script setup lang="ts">
import { computed, onMounted, watch } from 'vue'
import AppItem from '@/components/AppItem.vue'
import { useApps } from '@/composables/useApps'

const props = withDefaults(
    defineProps<{
        searchWord?: string
        group?: string
        viewMode?: 'list' | 'grid'
    }>(),
    {
        searchWord: '',
        group: 'all',
        viewMode: 'grid',
    },
)

const {
    allApps,
    loading,
    loaded,
    loadError,
    loadApps,
    retryLoadApps,
    ensureIcons,
} = useApps()

const keyword = computed(() => props.searchWord.trim().toLocaleLowerCase())

const filteredApps = computed(() => {
    const apps =
        props.group === 'all'
            ? allApps.value
            : allApps.value.filter((app) => app.category === props.group)

    if (!keyword.value) return apps

    return apps.filter(
        (app) =>
            app.name.toLocaleLowerCase().includes(keyword.value) ||
            app.path?.toLocaleLowerCase().includes(keyword.value) ||
            app.install_location?.toLocaleLowerCase().includes(keyword.value),
    )
})

function queueVisibleIcons() {
    ensureIcons(filteredApps.value, true)
}

async function handleRetry() {
    await retryLoadApps()
    queueVisibleIcons()
}

watch(filteredApps, queueVisibleIcons)
onMounted(() => {
    void loadApps().then(queueVisibleIcons)
})
</script>

<template>
    <section class="min-h-0 w-full flex-1 overflow-hidden border">
        <div v-if="loadError"
            class="mb-2 flex items-center justify-between gap-3 rounded-xl bg-red-50 px-3 py-2 text-xs text-red-700">
            <span>{{ loadError }}</span>
            <button type="button" class="shrink-0 font-medium underline underline-offset-2" @click="handleRetry">
                重新加载
            </button>
        </div>

        <div v-if="loading" class="grid h-full place-items-center">
            <div class="text-center text-sm text-slate-400">
                <n-spin size="small" />
                <div class="mt-2">正在读取应用列表…</div>
            </div>
        </div>

        <div v-else-if="loaded && !filteredApps.length" class="grid h-full place-items-center text-center">
            <div>
                <div class="text-sm font-medium text-slate-600">
                    {{ keyword ? '没有匹配的应用' : '这个分组暂时为空' }}
                </div>
                <div class="mt-1 text-xs text-slate-400">
                    {{ keyword ? '换个名称或关键字试试' : '重新加载后再试' }}
                </div>
            </div>
        </div>

        <n-scrollbar v-else-if="loaded" class="h-full">
            <div
                class="grid auto-rows-min pb-2 pr-2"
                :class="
                    viewMode === 'list'
                        ? 'grid-cols-1 gap-2'
                        : 'grid-cols-8 gap-2'
                "
            >
                <AppItem
                    v-for="app in filteredApps"
                    :key="`${app.category}:${app.name}`"
                    :app="app"
                    :layout="viewMode"
                />
            </div>
        </n-scrollbar>
    </section>
</template>
