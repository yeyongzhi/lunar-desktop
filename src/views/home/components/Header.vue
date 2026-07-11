<script setup lang="ts">
import { useDateTime } from "@/composables/useDateTime";

const { dateTime } = useDateTime();
</script>

<template>
    <div class="w-full flex gap-x-4 items-center justify-between text-base">
        <div class="flex items-center font-bold">
            {{ dateTime.date }}
            <span class="mx-2">·</span>
            {{ dateTime.weekDay }}
        </div>
    </div>
</template>
