<script setup lang="ts">
import { computed } from 'vue'
import { LayoutGrid, LayoutList } from '@lucide/vue'
import { useApps } from '@/composables/useApps'

const selectedGroup = defineModel<string>({ default: 'all' })
const viewMode = defineModel<'list' | 'grid'>('viewMode', { default: 'grid' })
const { systemApps, installedApps, allApps } = useApps()

const groups = computed(() => [
    { label: '全部应用', value: 'all', count: allApps.value.length },
    { label: '系统应用', value: 'system', count: systemApps.value.length },
    { label: '安装应用', value: 'installed', count: installedApps.value.length },
])

const viewModeTip = computed(() =>
    viewMode.value === 'list'
        ? '切换为网格模式'
        : '切换为列表模式',
)

function toggleViewMode() {
    viewMode.value = viewMode.value === 'list' ? 'grid' : 'list'
}
</script>

<template>
    <section class="flex items-center justify-between" aria-label="应用分组">
        <div class="flex-1 flex items-center gap-x-2">
            <n-button v-for="item in groups" :key="item.value" round
                :type="selectedGroup === item.value ? 'primary' : 'default'" @click="selectedGroup = item.value">
                {{ item.label }}
                <span class="ml-1 opacity-60">({{ item.count }})</span>
            </n-button>
        </div>

        <div>
            <n-tooltip>
                <template #trigger>
                    <n-button
                        circle
                        quaternary
                        :aria-label="viewModeTip"
                        @click="toggleViewMode"
                    >
                        <template #icon>
                            <n-icon>
                                <LayoutList v-if="viewMode === 'list'" />
                                <LayoutGrid v-else />
                            </n-icon>
                        </template>
                    </n-button>
                </template>
                {{ viewModeTip }}
            </n-tooltip>
        </div>
    </section>
</template>
