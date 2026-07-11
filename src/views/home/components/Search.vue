<script setup lang="ts">
import { ref } from 'vue';
import XIcon from '@/components/ui/XIcon/index.vue'

const searchWord = ref("")

</script>

<template>
    <div class="">
        <n-input v-model:value="searchWord" type="text" round placeholder="请输入应用名称或关键字进行搜索">
            <template #suffix>
                <XIcon name="Search" />
            </template>
        </n-input>
    </div>
</template>
