<script setup lang="ts">
withDefaults(
  defineProps<{
    size?: number;
  }>(),
  { size: 24 },
);
</script>

<template>
  <svg
    aria-hidden="true"
    class="brand-mark"
    focusable="false"
    :height="size"
    viewBox="0 0 1024 1024"
    :width="size"
  >
    <path
      class="brand-mark__body"
      d="M172 720V325c0-115 93-209 209-209h224c235 0 419 171 419 383 0 211-176 365-402 365H172z"
    />
    <path
      class="brand-mark__body"
      d="M548 703h74c-8 62-61 105-127 105-70 0-120 51-120 117v12c0 45-36 81-81 81s-81-36-81-81c0-177 107-274 275-274z"
    />
    <path
      class="brand-mark__fold"
      d="M637 406c119 77 190 175 190 283 0 99-83 162-215 162h-79c-92 0-163 58-163 145 0 28 13 54 34 71-78-22-134-92-134-177 0-177 107-275 280-275h72c119 0 210-69 210-189 0-14-2-29-5-43z"
    />
    <path
      class="brand-mark__paper"
      d="M515 211c128 80 191 174 191 278 0 113-80 198-218 250-134 51-217 131-270 239V734c41-100 135-189 263-282 103-75 151-148 151-224 0-48-10-89-30-124-28 38-56 72-87 107z"
    />
  </svg>
</template>

<style scoped>
.brand-mark {
  display: block;
  flex: none;
}

.brand-mark__body {
  fill: var(--accent);
}

.brand-mark__fold {
  fill: color-mix(in srgb, var(--accent) 74%, #000);
}

.brand-mark__paper {
  fill: #fff;
}
</style>
