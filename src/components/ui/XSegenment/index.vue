<script setup lang="ts" name="XSegenment">
export interface SegmentOption {
    label: string
    value: string | number
    disabled?: boolean
}

withDefaults(
    defineProps<{
        /** v-model 当前选中值 */
        modelValue?: string | number
        /** 选项列表 */
        options: SegmentOption[]
        /** 尺寸 */
        size?: 'small' | 'medium' | 'large'
    }>(),
    {
        size: 'medium',
    },
)

const emit = defineEmits<{
    'update:modelValue': [value: string | number]
}>()

function handleUpdateValue(value: string | number) {
    emit('update:modelValue', value)
}
</script>

<template>
    <n-tabs
        type="segment"
        animated
        :value="modelValue"
        :size="size"
        @update:value="handleUpdateValue"
    >
        <n-tab
            v-for="option in options"
            :key="option.value"
            :name="option.value"
            :disabled="option.disabled"
        >
            {{ option.label }}
        </n-tab>
    </n-tabs>
</template>
