<script setup lang="ts">
import {
  NAlert,
  NButton,
  NButtonGroup,
  NCard,
  NEmpty,
  NInput,
  NLayout,
  NLayoutContent,
  NList,
  NListItem,
  NModal,
  NProgress,
  NSpace,
  NText,
  NThing,
  NTooltip,
} from "naive-ui";
import { computed, onBeforeUnmount, onMounted, ref } from "vue";
import { useI18n } from "vue-i18n";

import { useMergeStore } from "../stores/merge/useMergeStore";

const { t } = useI18n();
const merge = useMergeStore();
const showSummary = ref(false);
const dragIndex = ref<number | null>(null);
const showIncidents = computed(() => merge.pendingInspection !== null);

onMounted(() => {
  void merge.initialize();
  void merge.protectWindowClose(async () => window.confirm(t("merge.closeWhileRunning")));
});
onBeforeUnmount(() => merge.dispose());

async function openSummary() {
  if (await merge.requestSummary()) showSummary.value = true;
}

function dropSource(index: number) {
  if (dragIndex.value !== null) merge.reorderSource(dragIndex.value, index);
  dragIndex.value = null;
}
</script>

<template>
  <NLayout class="application-shell">
    <NLayoutContent content-style="padding: 2rem;">
      <main class="merge-page">
        <header>
          <h1>{{ t("merge.heading") }}</h1>
          <NText depth="3">{{ t("merge.intro") }}</NText>
        </header>

        <NCard v-if="merge.phase === 'running'" embedded>
          <NThing :title="t('merge.processing')">
            <NText depth="3">{{ t("merge.progress", merge.progress) }}</NText>
          </NThing>
          <NProgress :percentage="merge.progress.percent" indicator-placement="inside" processing />
          <NButton type="error" @click="merge.cancelMerge">{{ t("merge.cancel") }}</NButton>
        </NCard>

        <template v-else>
          <NAlert v-if="merge.errorMessage" type="error" :title="t('merge.error')">
            {{ merge.errorMessage }}
          </NAlert>
          <NAlert v-if="merge.ignoredNonPdfs.length" type="warning" :title="t('merge.ignored')">
            {{ merge.ignoredNonPdfs.join(", ") }}
          </NAlert>

          <NCard embedded :title="t('merge.sources')">
            <NSpace>
              <NButton type="primary" @click="merge.choosePdfFiles">{{
                t("merge.addFiles")
              }}</NButton>
              <NButton @click="merge.chooseSourceFolder">{{ t("merge.addFolder") }}</NButton>
            </NSpace>
            <NText depth="3" class="drop-hint">{{ t("merge.dropHint") }}</NText>
            <NEmpty v-if="!merge.sources.length" :description="t('merge.emptySources')" />
            <NList v-else bordered>
              <NListItem
                v-for="(source, index) in merge.sources"
                :key="`${source.path}-${index}`"
                draggable="true"
                @dragstart="dragIndex = index"
                @dragover.prevent
                @drop="dropSource(index)"
              >
                <NTooltip trigger="hover"
                  ><template #trigger
                    ><strong>{{ source.name }}</strong></template
                  >{{ source.path }}</NTooltip
                >
                <template #suffix>
                  <NButtonGroup class="row-actions">
                    <NButton
                      quaternary
                      size="small"
                      :disabled="index === 0"
                      @click="merge.moveSource(index, -1)"
                      >↑</NButton
                    >
                    <NButton
                      quaternary
                      size="small"
                      :disabled="index === merge.sources.length - 1"
                      @click="merge.moveSource(index, 1)"
                      >↓</NButton
                    >
                    <NButton
                      quaternary
                      type="error"
                      size="small"
                      @click="merge.removeSource(index)"
                      >{{ t("merge.remove") }}</NButton
                    >
                  </NButtonGroup>
                </template>
              </NListItem>
            </NList>
          </NCard>

          <NCard embedded :title="t('merge.destination')">
            <label
              >{{ t("merge.outputName")
              }}<NInput
                :value="merge.outputName"
                :placeholder="t('merge.outputPlaceholder')"
                @update:value="merge.renameOutput"
            /></label>
            <label
              >{{ t("merge.destinationPath")
              }}<NInput
                :value="merge.destination"
                :placeholder="t('merge.destinationPlaceholder')"
                @update:value="merge.chooseDestination"
            /></label>
            <NButton @click="merge.chooseDestinationFolder">{{ t("merge.browse") }}</NButton>
          </NCard>

          <NButton
            block
            type="primary"
            size="large"
            :disabled="!merge.canRequestSummary"
            @click="openSummary"
          >
            {{ t("merge.review") }}
          </NButton>
        </template>
      </main>
    </NLayoutContent>
  </NLayout>

  <NModal
    :show="showIncidents"
    :mask-closable="false"
    preset="card"
    :title="t('merge.invalidSources')"
    style="width: min(92vw, 34rem)"
  >
    <p>{{ t("merge.invalidSourcesBody") }}</p>
    <NList bordered
      ><NListItem v-for="incident in merge.pendingInspection?.incidents" :key="incident.path">{{
        incident.name
      }}</NListItem></NList
    >
    <template #action
      ><NSpace justify="end"
        ><NButton @click="merge.cancelPreparation">{{ t("merge.cancelPreparation") }}</NButton
        ><NButton type="primary" @click="merge.ignoreInvalidSources">{{
          t("merge.ignoreAndContinue")
        }}</NButton></NSpace
      ></template
    >
  </NModal>

  <NModal
    v-model:show="showSummary"
    preset="card"
    :title="t('merge.summaryTitle')"
    style="width: min(92vw, 38rem)"
  >
    <NList bordered
      ><NListItem v-for="source in merge.sources" :key="source.path">{{
        source.name
      }}</NListItem></NList
    >
    <p>
      <strong>{{ t("merge.output") }}</strong> {{ merge.outputPreview?.outputPath }}
    </p>
    <NAlert v-if="merge.warnings.length" type="warning" :title="t('merge.warningTitle')">{{
      t("merge.warningBody")
    }}</NAlert>
    <template #action
      ><NSpace justify="end"
        ><NButton @click="showSummary = false">{{ t("merge.back") }}</NButton
        ><NButton
          type="primary"
          @click="
            showSummary = false;
            merge.confirmMerge();
          "
          >{{ t("merge.confirm") }}</NButton
        ></NSpace
      ></template
    >
  </NModal>
</template>

<style scoped>
.application-shell {
  min-height: 100vh;
}
.merge-page {
  display: grid;
  gap: 1rem;
  margin: 0 auto;
  max-width: 52rem;
}
h1 {
  margin: 0;
}
label {
  display: grid;
  gap: 0.4rem;
  margin-bottom: 0.8rem;
}
.drop-hint {
  display: block;
  margin: 0.8rem 0;
}

:deep(.n-list-item) {
  min-height: 0;
  padding: 0.35rem 0.75rem;
}

:deep(.n-list-item__main) {
  padding: 0;
}

.row-actions {
  flex-wrap: nowrap;
}
</style>
