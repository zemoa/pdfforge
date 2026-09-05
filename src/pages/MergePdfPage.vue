<script setup lang="ts">
import {
  NAlert,
  NButton,
  NButtonGroup,
  NCard,
  NEmpty,
  NInput,
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

import ToolWorkspaceShell from "../components/ToolWorkspaceShell.vue";
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
  <ToolWorkspaceShell
    active-tool="merge"
    :navigation-disabled="merge.phase === 'running'"
    :title="t('merge.heading')"
  >
    <template #left-panel>
      <section class="panel-section">
        <NText strong>{{ t("merge.sources") }}</NText>
        <NText depth="3" class="drop-hint">{{ t("merge.dropHint") }}</NText>
        <NButton
          type="primary"
          block
          :disabled="merge.phase === 'running'"
          @click="merge.choosePdfFiles"
          >{{ t("merge.addFiles") }}</NButton
        >
        <NButton block :disabled="merge.phase === 'running'" @click="merge.chooseSourceFolder">{{
          t("merge.addFolder")
        }}</NButton>
      </section>
    </template>

    <template #right-panel>
      <section class="panel-section">
        <NText strong>{{ t("merge.destination") }}</NText>
        <label
          >{{ t("merge.outputName")
          }}<NInput
            :value="merge.outputName"
            :disabled="merge.phase === 'running'"
            :placeholder="t('merge.outputPlaceholder')"
            @update:value="merge.renameOutput"
        /></label>
        <label
          >{{ t("merge.destinationPath")
          }}<NInput
            :value="merge.destination"
            :disabled="merge.phase === 'running'"
            :placeholder="t('merge.destinationPlaceholder')"
            @update:value="merge.chooseDestination"
        /></label>
        <NButton :disabled="merge.phase === 'running'" @click="merge.chooseDestinationFolder">{{
          t("merge.browse")
        }}</NButton>
      </section>
    </template>

    <section class="merge-workspace">
      <NCard v-if="merge.phase === 'running'" embedded class="process-card">
        <NThing :title="t('merge.processing')">
          <NText depth="3">{{ t("merge.progress", merge.progress) }}</NText>
        </NThing>
        <NProgress :percentage="merge.progress.percent" indicator-placement="inside" processing />
        <NButton type="error" @click="merge.cancelMerge">{{ t("merge.cancel") }}</NButton>
      </NCard>

      <template v-else>
        <NAlert v-if="merge.errorMessage" type="error" :title="t('merge.error')">{{
          merge.errorMessage
        }}</NAlert>
        <NAlert v-if="merge.ignoredNonPdfs.length" type="warning" :title="t('merge.ignored')">{{
          merge.ignoredNonPdfs.join(", ")
        }}</NAlert>

        <div class="workspace-caption">
          <NText depth="3">{{ t("merge.intro") }}</NText>
          <NText depth="3">{{ t("merge.count", { count: merge.sources.length }) }}</NText>
        </div>
        <NEmpty
          v-if="!merge.sources.length"
          class="workspace-empty"
          :description="t('merge.emptySources')"
        />
        <NList v-else bordered class="source-list">
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
                <NButton quaternary type="error" size="small" @click="merge.removeSource(index)">{{
                  t("merge.remove")
                }}</NButton>
              </NButtonGroup>
            </template>
          </NListItem>
        </NList>
      </template>
    </section>

    <template #footer>
      <NButton
        block
        type="primary"
        size="large"
        :disabled="merge.phase === 'running' || !merge.canRequestSummary"
        @click="openSummary"
        >{{ t("merge.review") }}</NButton
      >
    </template>
  </ToolWorkspaceShell>

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
.merge-workspace {
  box-sizing: border-box;
  display: flex;
  flex-direction: column;
  gap: 1rem;
  height: 100%;
  min-height: 0;
  padding: 1.25rem;
}

.panel-section {
  align-content: start;
  display: grid;
  gap: 0.75rem;
}

label {
  display: grid;
  gap: 0.4rem;
}
.drop-hint {
  display: block;
  line-height: 1.5;
}

.workspace-caption {
  align-items: center;
  display: flex;
  justify-content: space-between;
}

.workspace-empty {
  display: grid;
  flex: 1;
  place-items: center;
}

.source-list {
  overflow: auto;
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
