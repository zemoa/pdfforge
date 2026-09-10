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
    <NAlert v-if="update.errorCode" type="error" :title="t('update.error')">
      {{ t(`errors.${update.errorCode}`) }}
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
    <section v-if="update.result && update.result.kind !== 'upToDate'" class="available-update">
      <NText v-if="update.result.kind === 'available'" strong>
        {{ t("update.available", { version: update.result.version }) }}
      </NText>
      <div
        class="update-releases"
        role="region"
        :aria-label="t('update.releaseNotes')"
        tabindex="0"
      >
        <section v-for="release in update.result.releases" :key="release.version">
          <h3 class="update-version">PDFForge {{ release.version }}</h3>
          <p v-if="release.notes" class="update-notes">{{ release.notes }}</p>
        </section>
      </div>
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
  overflow-wrap: anywhere;
  white-space: pre-wrap;
}

.update-releases {
  display: grid;
  gap: 1.2rem;
  max-height: 40vh;
  overflow-y: auto;
}

.update-version {
  margin: 0 0 0.4rem;
  font-size: 1rem;
}
</style>
