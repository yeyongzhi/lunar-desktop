<script setup lang="ts">
import XIcon from '@/components/ui/XIcon/index.vue'

const searchWord = defineModel<string>({ default: '' })
</script>

<template>
    <n-input
        v-model:value="searchWord"
        type="text"
        round
        clearable
        placeholder="搜索应用"
    >
        <template #prefix>
            <XIcon name="Search" />
        </template>
    </n-input>
</template>
