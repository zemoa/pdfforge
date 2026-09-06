<script setup lang="ts">
import { NAlert, NButton, NModal, NProgress, NSpace, NText } from "naive-ui";
import { computed, watch } from "vue";
import { useI18n } from "vue-i18n";

import { useUpdateStore } from "../stores/update/useUpdateStore";

const show = defineModel<boolean>("show", { required: true });
const { locale, t } = useI18n();
const update = useUpdateStore();
const downloadProgress = computed(() => update.progressPercent ?? 0);

watch(show, (opened) => {
  if (opened) void update.initialize();
});
</script>

<template>
  <NModal
    v-model:show="show"
    preset="card"
    :title="t('update.title')"
    style="width: min(92vw, 34rem)"
  >
    <p>{{ t("update.installed", { version: update.installedVersion || "—" }) }}</p>
    <NAlert v-if="update.updatedTo" type="success" :title="t('update.updated')">
      {{ t("update.updatedBody", { version: update.updatedTo }) }}
    </NAlert>
    <NAlert v-if="update.errorMessage" type="error" :title="t('update.error')">
      {{ update.errorMessage }}
    </NAlert>
    <NAlert v-if="update.manualDownloadVersion" type="info" :title="t('update.manualTitle')">
      {{ t("update.manualBody", { version: update.manualDownloadVersion }) }}
    </NAlert>
    <NAlert v-if="update.result?.kind === 'upToDate'" type="success">
      {{ t("update.upToDate") }}
    </NAlert>
    <NAlert v-if="update.result?.kind === 'unsupported'" type="warning">
      {{ t("update.unsupported") }}
    </NAlert>
    <section v-if="update.result?.kind === 'available'" class="available-update">
      <NText strong>{{ t("update.available", { version: update.result.version }) }}</NText>
      <p v-if="update.result.notes" class="update-notes">{{ update.result.notes }}</p>
    </section>
    <section v-if="update.phase === 'downloading'" class="update-progress">
      <NText>{{ t("update.downloading") }}</NText>
      <NProgress :percentage="downloadProgress" :indicator-placement="'inside'" processing />
    </section>
    <template #action>
      <NSpace justify="space-between">
        <NButton
          v-if="update.rollbackAvailable"
          quaternary
          :disabled="update.phase !== 'idle'"
          @click="update.restore"
        >
          {{ t("update.restore") }}
        </NButton>
        <span v-else />
        <NSpace>
          <NButton :disabled="update.phase === 'downloading'" @click="show = false">{{
            t("update.close")
          }}</NButton>
          <NButton
            v-if="update.phase === 'downloading'"
            type="error"
            secondary
            @click="update.cancel"
          >
            {{ t("update.cancel") }}
          </NButton>
          <NButton
            :loading="update.phase === 'checking'"
            :disabled="update.phase === 'downloading'"
            @click="update.check(locale)"
          >
            {{ t("update.check") }}
          </NButton>
          <NButton
            v-if="update.result?.kind === 'available'"
            type="primary"
            :disabled="update.phase !== 'idle'"
            @click="update.install"
            >{{ t("update.install") }}</NButton
          >
        </NSpace>
      </NSpace>
    </template>
  </NModal>
</template>

<style scoped>
.available-update,
.update-progress {
  display: grid;
  gap: 0.7rem;
  margin-top: 1rem;
}

.update-notes {
  margin: 0;
  white-space: pre-wrap;
}
</style>
