<script setup lang="ts">
import type { NDateLocale, NLocale, GlobalThemeOverrides } from 'naive-ui'
import { dateZhCN, zhCN } from 'naive-ui'
import { ref } from 'vue'

const locale = ref<NLocale | null>(zhCN)
const dateLocale = ref<NDateLocale | null>(dateZhCN)

const themeOverrides: GlobalThemeOverrides = {
    common: {
        fontSize: '1rem'
    }
}

</script>

<template>
    <n-config-provider :theme-overrides="themeOverrides" class="w-full h-full" :locale="locale"
        :date-locale="dateLocale" inline-theme-disabled>
        <n-el class="w-full h-full">
            <router-view />
        </n-el>
    </n-config-provider>
</template>