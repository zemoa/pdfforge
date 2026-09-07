<script setup lang="ts">
import { NInput } from "naive-ui";

defineProps<{
  disabled: boolean;
  placeholder: string;
  value: string;
}>();

const emit = defineEmits<{
  paste: [pastedText: string];
  "update:value": [value: string];
}>();

function pasteDestination(event: ClipboardEvent) {
  emit("paste", event.clipboardData?.getData("text/plain") ?? "");
}
</script>

<template>
  <NInput
    :value="value"
    :disabled="disabled"
    :placeholder="placeholder"
    @update:value="emit('update:value', $event)"
    @paste.prevent="pasteDestination"
  />
</template>
