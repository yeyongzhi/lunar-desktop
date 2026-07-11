<script setup lang="ts" name="XIcon">
import { computed } from "vue";
import type { Component } from "vue";
import * as LucideIcons from "@lucide/vue";

const props = withDefaults(
  defineProps<{
    name: string;
    size?: number | string;
    color?: string;
  }>(),
  {
    size: 20,
  }
);

const iconComponent = computed<Component | null>(() => {
  // Lucide icon names are PascalCase (e.g. "ArrowRight", "ChevronDown")
  const icon = LucideIcons[props.name as keyof typeof LucideIcons];
  // Filter out non-component exports (e.g. LUCIDE_CONTEXT, createLucideIcon)
  if (!icon || (typeof icon !== "function" && typeof icon !== "object")) {
    console.warn(`[XIcon] Icon "${props.name}" not found in @lucide/vue`);
    return null;
  }
  return icon as unknown as Component;
});
</script>

<template>
  <n-icon v-if="iconComponent" :size="size" :color="color">
    <component :is="iconComponent" />
  </n-icon>
</template>
