<script setup lang="ts">
import {
  NAlert,
  NButton,
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
import { computed, onBeforeUnmount, onMounted, ref, watch } from "vue";
import { useI18n } from "vue-i18n";

import DestinationFolderInput from "../components/DestinationFolderInput.vue";
import LineIcon from "../components/LineIcon.vue";
import ReorderControls from "../components/ReorderControls.vue";
import ToolWorkspaceShell from "../components/ToolWorkspaceShell.vue";
import { usePointerReorder } from "../composables/usePointerReorder";
import { useMergeStore } from "../stores/merge/useMergeStore";

const { t } = useI18n();
const merge = useMergeStore();
const showSummary = ref(false);
const documentStage = ref<HTMLElement | null>(null);
const showIncidents = computed(() => merge.pendingInspection !== null);
const reorderDisabled = computed(
  () => merge.phase === "running" || showSummary.value || showIncidents.value,
);
const reorder = usePointerReorder({
  entryIds: () => merge.sources.map((source) => source.entryId),
  enabled: () => !reorderDisabled.value,
  container: () => documentStage.value,
  move: (from, to) => merge.reorderSource(from, to),
});
const totalPages = computed(() =>
  merge.sources.reduce((total, source) => total + source.pageCount, 0),
);

onMounted(() => {
  void merge.initialize();
  void merge.protectWindowClose(async () => window.confirm(t("merge.closeWhileRunning")));
});
onBeforeUnmount(() => merge.dispose());

async function openSummary() {
  if (await merge.requestSummary()) showSummary.value = true;
}

watch(
  () => merge.outputPreview,
  (preview) => {
    if (!preview) showSummary.value = false;
  },
  { flush: "sync" },
);
</script>

<template>
  <ToolWorkspaceShell
    active-tool="merge"
    :navigation-disabled="merge.phase === 'running'"
    :title="t('merge.heading')"
  >
    <template #left-panel>
      <section class="documents-panel">
        <div class="panel-heading">
          <span class="panel-eyebrow">{{ t("common.documents") }}</span>
          <NButton
            class="add-pdf-button"
            :disabled="merge.phase === 'running'"
            @click="merge.choosePdfFiles"
            ><LineIcon name="add" :size="14" /> {{ t("merge.addFiles") }}</NButton
          >
          <NButton
            quaternary
            size="small"
            :disabled="merge.phase === 'running'"
            @click="merge.chooseSourceFolder"
            ><LineIcon name="folder" :size="14" /> {{ t("merge.addFolder") }}</NButton
          >
        </div>

        <div v-if="merge.sources.length" class="document-list">
          <div v-for="(source, index) in merge.sources" :key="source.entryId" class="document-row">
            <ReorderControls
              :position="index + 1"
              :count="merge.sources.length"
              :name="source.name"
              :disabled="reorderDisabled"
              @move="merge.moveSource(index, $event)"
            />
            <NTooltip trigger="hover"
              ><template #trigger
                ><div class="document-row__copy">
                  <strong>{{ source.name }}</strong
                  ><span>{{ t("common.pageCount", { count: source.pageCount }) }}</span>
                </div></template
              >{{ source.path }}</NTooltip
            >
            <NButton
              quaternary
              class="document-row__remove"
              :aria-label="t('merge.remove')"
              :disabled="merge.phase === 'running'"
              @click="merge.removeSource(index)"
              ><LineIcon name="trash" :size="14"
            /></NButton>
          </div>
        </div>
        <NText v-else depth="3" class="panel-hint">{{ t("merge.dropHint") }}</NText>

        <div class="output-panel">
          <span class="panel-eyebrow">{{ t("common.output") }}</span>
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
            }}<DestinationFolderInput
              :value="merge.destination"
              :disabled="merge.phase === 'running'"
              :placeholder="t('merge.destinationPlaceholder')"
              @update:value="merge.chooseDestination"
              @paste="merge.pasteDestination"
          /></label>
          <NButton
            quaternary
            size="small"
            :disabled="merge.phase === 'running'"
            @click="merge.chooseDestinationFolder"
            ><LineIcon name="folder" :size="14" /> {{ t("merge.browse") }}</NButton
          >
          <span class="originals-note">{{ t("common.originalsSafe") }}</span>
        </div>
      </section>
    </template>

    <section class="merge-workspace">
      <template v-if="merge.phase === 'running'">
        <div class="process-state">
          <NThing :title="t('merge.processing')"
            ><NText depth="3">{{ t("merge.progress", merge.progress) }}</NText></NThing
          ><NProgress
            :percentage="merge.progress.percent"
            indicator-placement="inside"
            processing
          /><NButton type="error" secondary @click="merge.cancelMerge">{{
            t("merge.cancel")
          }}</NButton>
        </div>
      </template>
      <template v-else>
        <NAlert v-if="merge.errorCode" type="error" :title="t('merge.error')">{{
          t(`errors.${merge.errorCode}`)
        }}</NAlert>
        <NAlert v-if="merge.ignoredNonPdfs.length" type="warning" :title="t('merge.ignored')">{{
          merge.ignoredNonPdfs.join(", ")
        }}</NAlert>
        <div class="workspace-toolbar">
          <div>
            <span class="toolbar-title">{{ t("merge.order") }}</span
            ><span class="toolbar-meta"
              >{{ t("merge.count", { count: merge.sources.length }) }} ·
              {{ t("common.pageCount", { count: totalPages }) }}</span
            >
          </div>
        </div>
        <NText v-if="merge.sources.length > 1" depth="3" class="panel-hint">{{
          t("merge.reorderHint")
        }}</NText>
        <NEmpty
          v-if="!merge.sources.length"
          class="workspace-empty"
          :description="t('merge.emptySources')"
        />
        <div v-else ref="documentStage" class="document-stage">
          <template v-for="(source, index) in merge.sources" :key="source.entryId">
            <article
              class="pdf-document"
              :data-reorder-id="source.entryId"
              :class="[
                reorder.itemClasses(source.entryId),
                { 'pdf-document--movable': !reorderDisabled && merge.sources.length > 1 },
              ]"
              :aria-label="t('merge.dragSource', { name: source.name })"
              tabindex="-1"
              @pointerdown="reorder.start($event, source.entryId)"
              @dragstart.prevent
            >
              <div class="pdf-paper">
                <div class="pdf-paper__header">
                  <LineIcon name="document" :size="15" /><span>PDF</span>
                  <span
                    class="pdf-document__position"
                    :aria-label="
                      t('merge.position', { position: index + 1, count: merge.sources.length })
                    "
                    >{{ index + 1 }}</span
                  >
                </div>
                <div class="pdf-paper__title">{{ source.name.replace(/\.pdf$/i, "") }}</div>
                <span
                  v-for="line in 5"
                  :key="line"
                  class="pdf-paper__line"
                  :style="{ width: `${82 - line * 7}%` }"
                />
                <div class="pdf-paper__footer">
                  {{ source.pageCount }} {{ t("common.pagesShort") }}
                </div>
              </div>
              <div class="pdf-document__caption" :title="source.path">
                <strong>{{ source.name }}</strong
                ><span>{{ t("common.pageCount", { count: source.pageCount }) }}</span>
              </div>
            </article>
          </template>
        </div>
      </template>
    </section>

    <template #footer>
      <div class="footer-copy">
        {{ merge.outputName || t("merge.outputPlaceholder") }}.pdf
        <span>{{ t("common.creationNote") }}</span>
      </div>
      <NSpace size="small"
        ><NButton
          quaternary
          :disabled="merge.phase === 'running'"
          @click="merge.cancelPreparation"
          >{{ t("common.reset") }}</NButton
        ><NButton
          class="create-button"
          :disabled="merge.phase === 'running' || !merge.canRequestSummary"
          @click="openSummary"
          >{{ t("merge.review") }} <i /></NButton
      ></NSpace>
    </template>
  </ToolWorkspaceShell>

  <NModal
    :show="showIncidents"
    :mask-closable="false"
    preset="card"
    :title="t('merge.invalidSources')"
    style="width: min(92vw, 34rem)"
    ><p>{{ t("merge.invalidSourcesBody") }}</p>
    <NList bordered
      ><NListItem v-for="incident in merge.pendingInspection?.incidents" :key="incident.path">{{
        incident.name
      }}</NListItem></NList
    ><template #action
      ><NSpace justify="end"
        ><NButton @click="merge.cancelPreparation">{{ t("merge.cancelPreparation") }}</NButton
        ><NButton type="primary" @click="merge.ignoreInvalidSources">{{
          t("merge.ignoreAndContinue")
        }}</NButton></NSpace
      ></template
    ></NModal
  >
  <NModal
    v-model:show="showSummary"
    preset="card"
    :title="t('merge.summaryTitle')"
    style="width: min(92vw, 38rem)"
    ><NList bordered
      ><NListItem v-for="(source, index) in merge.sources" :key="source.entryId">{{
        `${index + 1}. ${source.name}`
      }}</NListItem></NList
    >
    <p>
      <strong>{{ t("merge.output") }}</strong> {{ merge.outputPreview?.outputPath }}
    </p>
    <NAlert v-if="merge.warnings.length" type="warning" :title="t('merge.warningTitle')">{{
      t("merge.warningBody")
    }}</NAlert
    ><template #action
      ><NSpace justify="end"
        ><NButton @click="showSummary = false">{{ t("merge.back") }}</NButton
        ><NButton
          type="primary"
          :disabled="!merge.outputPreview || merge.phase === 'running'"
          @click="
            showSummary = false;
            merge.confirmMerge();
          "
          >{{ t("merge.confirm") }}</NButton
        ></NSpace
      ></template
    ></NModal
  >
</template>

<style scoped>
.documents-panel {
  display: flex;
  flex-direction: column;
  height: 100%;
  min-height: 0;
}
.panel-heading,
.output-panel {
  display: grid;
  gap: 0.6rem;
}
.panel-eyebrow {
  color: var(--text-tertiary);
  font-size: 0.625rem;
  font-weight: 700;
  letter-spacing: 0.1em;
  text-transform: uppercase;
}
.add-pdf-button {
  justify-content: flex-start;
}
.panel-hint,
.originals-note {
  font-size: 0.6875rem;
  line-height: 1.45;
}
.document-list {
  align-content: start;
  display: grid;
  flex: 1;
  gap: 0.5rem;
  margin: 0.85rem 0;
  min-height: 6rem;
  overflow: auto;
  padding: 0.25rem;
}
.document-row {
  align-items: center;
  border-radius: var(--radius-sm);
  display: grid;
  gap: 0.55rem;
  grid-template-columns: auto minmax(0, 1fr) auto;
  padding: 0.45rem 0.35rem;
  position: relative;
}
.document-row:hover {
  background: var(--surface);
}
.document-row__copy {
  display: grid;
  gap: 0.15rem;
  min-width: 0;
}
.document-row__copy strong {
  font-size: 0.72rem;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}
.document-row__copy span {
  color: var(--text-tertiary);
  font-size: 0.625rem;
}
.document-row__remove {
  color: var(--text-tertiary);
  height: 1.5rem;
  padding: 0;
  width: 1.5rem;
}
.output-panel {
  border-top: 1px solid var(--border);
  margin-top: auto;
  padding-top: 1rem;
  flex-shrink: 0;
}
label {
  color: var(--text-secondary);
  display: grid;
  font-size: 0.655rem;
  gap: 0.3rem;
}
.originals-note {
  color: var(--text-tertiary);
  margin-top: 0.1rem;
}
.merge-workspace {
  box-sizing: border-box;
  display: flex;
  flex-direction: column;
  gap: 1rem;
  height: 100%;
  min-height: 0;
  padding: 1rem 1.25rem 1.25rem;
}
.workspace-toolbar {
  align-items: center;
  display: flex;
  justify-content: space-between;
}
.workspace-toolbar > div {
  align-items: baseline;
  display: flex;
  gap: 0.65rem;
  flex-wrap: wrap;
}
.toolbar-title {
  color: var(--text);
  font-size: 0.75rem;
  font-weight: 650;
}
.toolbar-meta {
  color: var(--text-tertiary);
  font-size: 0.6875rem;
}
.workspace-empty {
  display: grid;
  flex: 1;
  place-items: center;
}
.document-stage {
  align-content: start;
  align-items: start;
  display: flex;
  flex: 1;
  flex-wrap: wrap;
  gap: 1rem;
  justify-content: center;
  min-height: 0;
  overflow: auto;
  padding: 1rem;
}
.pdf-document {
  max-width: 12.5rem;
  min-width: 9.5rem;
  position: relative;
  width: min(29%, 12rem);
}
.pdf-document--movable {
  cursor: grab;
  touch-action: none;
  user-select: none;
}
.pdf-document--movable:active {
  cursor: grabbing;
}
.pdf-document__position {
  font-variant-numeric: tabular-nums;
  margin-left: auto;
}
.reorder-dragging {
  background: var(--accent-soft);
  border-radius: var(--radius-sm);
  outline: 1px dashed var(--accent);
}
.reorder-before::before,
.reorder-after::after {
  background: var(--accent);
  border-radius: 2px;
  content: "";
  pointer-events: none;
  position: absolute;
  z-index: 2;
}
.pdf-document.reorder-before::before,
.pdf-document.reorder-after::after {
  bottom: 0;
  top: 0;
  width: 3px;
}
.pdf-document.reorder-before::before {
  left: -0.6rem;
}
.pdf-document.reorder-after::after {
  right: -0.6rem;
}
.pdf-paper {
  background: #fff;
  border: 1px solid #d8d8d8;
  border-radius: 2px;
  box-shadow: 0 8px 18px rgb(0 0 0 / 5.5%);
  box-sizing: border-box;
  color: #444;
  display: flex;
  flex-direction: column;
  gap: 0.5rem;
  min-height: 14rem;
  padding: 0.9rem;
  transition:
    box-shadow 130ms ease,
    transform 130ms ease;
}
.pdf-document:hover .pdf-paper {
  box-shadow:
    0 0 0 2px var(--accent),
    0 10px 22px rgb(0 0 0 / 6.5%);
  transform: translateY(-2px);
}
.pdf-paper__header {
  align-items: center;
  color: #777;
  display: flex;
  font-size: 0.6rem;
  gap: 0.35rem;
  letter-spacing: 0.08em;
}
.pdf-paper__title {
  color: #222;
  font-size: 0.82rem;
  font-weight: 650;
  margin: 0.35rem 0;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}
.pdf-paper__line {
  background: #dedede;
  display: block;
  height: 1px;
}
.pdf-paper__footer {
  color: #999;
  font-size: 0.6rem;
  margin-top: auto;
}
.pdf-document__caption {
  display: grid;
  gap: 0.2rem;
  padding: 0.55rem 0.1rem 0;
}
.pdf-document__caption strong {
  font-size: 0.7rem;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}
.pdf-document__caption span {
  color: var(--text-tertiary);
  font-size: 0.63rem;
}
.process-state {
  background: var(--surface);
  border: 1px solid var(--border);
  border-radius: var(--radius-panel);
  display: grid;
  gap: 1rem;
  margin: auto;
  max-width: 30rem;
  padding: 1.25rem;
  width: min(100%, 30rem);
}
.footer-copy {
  color: var(--text-secondary);
  flex: 1;
  font-size: 0.7rem;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}
.footer-copy span {
  color: var(--text-tertiary);
}
.create-button {
  background: var(--text);
  border-color: var(--text);
  color: var(--surface);
  min-width: 9rem;
}
.create-button i {
  background: var(--accent);
  border-radius: 50%;
  height: 0.35rem;
  margin-left: 0.35rem;
  width: 0.35rem;
}
@media (max-width: 48rem) {
  .pdf-document {
    width: min(42%, 12rem);
  }
}
</style>
