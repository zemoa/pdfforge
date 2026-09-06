<script setup lang="ts">
import LineIcon from "./LineIcon.vue";

type IconName =
  | "add"
  | "chevronLeft"
  | "chevronRight"
  | "document"
  | "folder"
  | "grid"
  | "merge"
  | "more"
  | "redact"
  | "settings"
  | "split"
  | "theme"
  | "trash";

defineProps<{
  name: IconName;
  size?: number;
  strokeWidth?: number;
}>();
</script>

<template>
  <LineIcon :name="name" :size="size" :stroke-width="strokeWidth" />
</template>
