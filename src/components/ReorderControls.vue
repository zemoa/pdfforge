<script setup lang="ts">
import { NButton } from "naive-ui";
import { useI18n } from "vue-i18n";

import LineIcon from "./LineIcon.vue";

defineProps<{
  position: number;
  count: number;
  name: string;
  disabled: boolean;
}>();
const emit = defineEmits<{
  move: [direction: -1 | 1];
}>();
const { t } = useI18n();
</script>

<template>
  <div class="reorder-controls">
    <NButton
      quaternary
      size="tiny"
      :disabled="disabled || position <= 1"
      :aria-label="t('merge.moveUp', { name })"
      :title="t('merge.moveUp', { name })"
      @click="emit('move', -1)"
      ><LineIcon name="chevronUp" :size="16"
    /></NButton>
    <NButton
      quaternary
      size="tiny"
      :disabled="disabled || position >= count"
      :aria-label="t('merge.moveDown', { name })"
      :title="t('merge.moveDown', { name })"
      @click="emit('move', 1)"
      ><LineIcon name="chevronDown" :size="16"
    /></NButton>
  </div>
</template>

<style scoped>
.reorder-controls {
  align-items: center;
  display: flex;
  flex-direction: column;
}
.reorder-controls .n-button {
  height: 1.25rem;
  padding: 0;
  width: 1.5rem;
}
</style>
