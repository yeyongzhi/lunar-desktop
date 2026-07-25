<script setup lang="ts" name="XIcon">
import { computed } from "vue";
import type { Component } from "vue";
import {
  AppWindow,
  ArrowUpRight,
  ChevronDown,
  ChevronUp,
  Search,
} from "@lucide/vue";

const iconMap = {
  AppWindow,
  ArrowUpRight,
  ChevronDown,
  ChevronUp,
  Search,
} satisfies Record<string, Component>;

type IconName = keyof typeof iconMap;

const props = withDefaults(
  defineProps<{
    name: IconName;
    size?: number | string;
    color?: string;
  }>(),
  {
    size: 20,
  }
);

const iconComponent = computed<Component | null>(() => {
  return iconMap[props.name];
});
</script>

<template>
  <n-icon v-if="iconComponent" :size="size" :color="color">
    <component :is="iconComponent" />
  </n-icon>
</template>
