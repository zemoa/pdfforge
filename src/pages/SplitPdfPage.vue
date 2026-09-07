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
  NRadioButton,
  NRadioGroup,
  NSpace,
  NText,
  NThing,
} from "naive-ui";
import { computed, onBeforeUnmount, onMounted, ref } from "vue";
import { useI18n } from "vue-i18n";

import LineIcon from "../components/LineIcon.vue";
import ToolWorkspaceShell from "../components/ToolWorkspaceShell.vue";
import { useSplitStore } from "../stores/split/useSplitStore";

const { t } = useI18n();
const split = useSplitStore();
const showSummary = ref(false);
const groupSummary = computed(() =>
  split.groups.map((group, index) => ({
    id: group.id,
    index: index + 1,
    pages: group.pages.join(", "),
  })),
);

onMounted(() => {
  void split.initialize();
  void split.protectWindowClose(async () => window.confirm(t("split.closeWhileRunning")));
});
onBeforeUnmount(() => split.dispose());
async function openSummary() {
  if (await split.requestSummary()) showSummary.value = true;
}
function selectMode(mode: string | number) {
  if (mode === "eachPage" || mode === "extract" || mode === "groups") split.chooseMode(mode);
}
</script>

<template>
  <ToolWorkspaceShell
    active-tool="split"
    :navigation-disabled="split.phase === 'running'"
    :title="t('split.heading')"
  >
    <template #left-panel>
      <section class="documents-panel">
        <div class="panel-heading">
          <span class="panel-eyebrow">{{ t("common.documents") }}</span>
          <template v-if="split.source"
            ><div class="source-row">
              <LineIcon name="document" :size="16" />
              <div>
                <strong>{{ split.source.name }}</strong
                ><span>{{ t("common.pageCount", { count: split.source.pageCount }) }}</span>
              </div>
              <NButton
                quaternary
                class="source-row__remove"
                :aria-label="t('split.removeSource')"
                @click="split.removeSource"
                ><LineIcon name="trash" :size="14"
              /></NButton>
            </div>
            <NButton
              quaternary
              size="small"
              :disabled="split.phase === 'running'"
              @click="split.choosePdfFile"
              ><LineIcon name="add" :size="14" /> {{ t("split.replaceSource") }}</NButton
            ></template
          >
          <template v-else
            ><NText depth="3" class="panel-hint">{{ t("split.dropHint") }}</NText
            ><NButton
              class="add-pdf-button"
              :disabled="split.phase === 'running'"
              @click="split.choosePdfFile"
              ><LineIcon name="add" :size="14" /> {{ t("split.addSource") }}</NButton
            ></template
          >
        </div>
        <div v-if="split.source" class="output-panel">
          <span class="panel-eyebrow">{{ t("common.output") }}</span
          ><label
            >{{ t("split.outputName")
            }}<NInput
              :value="split.outputName"
              :disabled="split.phase === 'running'"
              :placeholder="t('split.outputPlaceholder')"
              @update:value="split.renameOutput" /></label
          ><label
            >{{ t("split.destinationPath")
            }}<NInput
              :value="split.destination"
              :disabled="split.phase === 'running'"
              :placeholder="t('split.destinationPlaceholder')"
              @update:value="split.chooseDestination" /></label
          ><NButton
            quaternary
            size="small"
            :disabled="split.phase === 'running'"
            @click="split.chooseDestinationFolder"
            ><LineIcon name="folder" :size="14" /> {{ t("split.browse") }}</NButton
          ><span class="originals-note">{{ t("common.originalsSafe") }}</span>
        </div>
      </section>
    </template>

    <template v-if="split.source && split.phase !== 'running'" #right-panel>
      <NRadioGroup
        class="mode-options"
        :value="split.mode"
        name="split-mode"
        size="small"
        @update:value="selectMode"
      >
        <NRadioButton value="eachPage">{{ t("split.eachPage") }}</NRadioButton>
        <NRadioButton value="extract">{{ t("split.extract") }}</NRadioButton>
        <NRadioButton value="groups">{{ t("split.groups") }}</NRadioButton>
      </NRadioGroup>
    </template>

    <section class="split-workspace">
      <template v-if="split.phase === 'running'"
        ><div class="process-state">
          <NThing :title="t('split.processing')"
            ><NText depth="3">{{ t("split.progress", split.progress) }}</NText></NThing
          ><NProgress
            :percentage="split.progress.percent"
            indicator-placement="inside"
            processing
          /><NButton type="error" secondary @click="split.cancelSplit">{{
            t("split.cancel")
          }}</NButton>
        </div></template
      >
      <template v-else>
        <NAlert v-if="split.errorMessage" type="error" :title="t('split.error')">{{
          split.errorMessage
        }}</NAlert>
        <NAlert v-if="split.outcome === 'succeeded'" type="success" :title="t('split.success')">{{
          t("split.successBody")
        }}</NAlert
        ><NAlert v-if="split.outcome === 'cancelled'" type="info" :title="t('split.cancelled')">{{
          t("split.cancelledBody")
        }}</NAlert>
        <NEmpty v-if="!split.source" class="workspace-empty" :description="t('split.emptySource')"
          ><template #extra
            ><NButton class="primary-document-action" @click="split.choosePdfFile"
              ><LineIcon name="add" :size="15" /> {{ t("split.addSource") }}</NButton
            ></template
          ></NEmpty
        >
        <template v-else>
          <div class="workspace-toolbar">
            <div>
              <span class="toolbar-title">{{ t("split.mode") }}</span
              ><span class="toolbar-meta">{{ t(`split.modeHelp.${split.mode}`) }}</span>
            </div>
          </div>
          <div v-if="split.mode !== 'eachPage'" class="selection-strip">
            <span>{{
              t("split.selectedPages", { pages: split.selectedPages.join(", ") || "—" })
            }}</span
            ><NSpace size="small"
              ><NButton
                v-if="split.mode === 'groups'"
                size="small"
                :disabled="!split.selectedPages.length"
                @click="split.createGroupFromSelection"
                >{{ t("split.createGroup") }}</NButton
              ><NButton
                quaternary
                size="small"
                :disabled="!split.selectedPages.length"
                @click="split.clearSelectedPages"
                >{{ t("split.clearSelection") }}</NButton
              ></NSpace
            >
          </div>
          <div v-if="split.mode === 'groups' && split.groups.length" class="groups-strip">
            <span v-for="group in groupSummary" :key="group.id" class="group-token"
              >{{ t("split.group", { number: group.index }) }} · {{ group.pages }}
              <button type="button" @click="split.removeGroup(group.id)">×</button></span
            >
          </div>
          <div class="thumbnail-workspace">
            <div class="thumbnail-grid">
              <button
                v-for="page in split.displayedPages"
                :key="page"
                type="button"
                class="page-thumbnail"
                :class="{
                  selected: split.selectedPages.includes(page),
                  assigned: split.assignedPages.includes(page),
                }"
                :disabled="split.mode === 'eachPage' || split.assignedPages.includes(page)"
                @click="split.togglePage(page)"
              >
                <span class="page-thumbnail__number">{{ page }}</span
                ><img
                  v-if="split.thumbnails[page]"
                  :src="split.thumbnails[page]"
                  :alt="t('split.pageThumbnail', { page })"
                /><span v-else class="thumbnail-placeholder" /><span
                  class="page-thumbnail__label"
                  >{{ t("split.page", { page }) }}</span
                >
              </button>
            </div>
            <NButton
              v-if="split.canLoadMoreThumbnails"
              quaternary
              size="small"
              :loading="split.thumbnailsLoading"
              @click="split.loadNextThumbnails"
              >{{ t("split.loadMore") }}</NButton
            >
          </div>
        </template>
      </template>
    </section>
    <template #footer
      ><div class="footer-copy">
        {{ split.outputName || t("split.outputPlaceholder") }}.pdf
        <span>{{ t("common.creationNote") }}</span>
      </div>
      <NSpace size="small"
        ><NButton
          quaternary
          :disabled="split.phase === 'running'"
          @click="split.resetPreparation"
          >{{ t("common.reset") }}</NButton
        ><NButton
          class="create-button"
          :disabled="split.phase === 'running' || !split.canRequestSummary"
          @click="openSummary"
          >{{ t("split.review") }} <i /></NButton></NSpace
    ></template>
  </ToolWorkspaceShell>
  <NModal
    v-model:show="showSummary"
    preset="card"
    :title="t('split.summaryTitle')"
    style="width: min(92vw, 42rem)"
    ><NList bordered
      ><NListItem v-for="path in split.outputPreview?.outputPaths" :key="path">{{
        path
      }}</NListItem></NList
    ><template #action
      ><NSpace justify="end"
        ><NButton @click="showSummary = false">{{ t("split.back") }}</NButton
        ><NButton
          type="primary"
          @click="
            showSummary = false;
            split.confirmSplit();
          "
          >{{ t("split.confirm") }}</NButton
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
  gap: 0.65rem;
}
.panel-eyebrow {
  color: var(--text-tertiary);
  font-size: 0.625rem;
  font-weight: 700;
  letter-spacing: 0.1em;
  text-transform: uppercase;
}
.panel-hint,
.originals-note {
  font-size: 0.6875rem;
  line-height: 1.45;
}
.add-pdf-button {
  justify-content: flex-start;
}
.source-row {
  align-items: center;
  background: var(--surface);
  border-radius: var(--radius-sm);
  display: grid;
  gap: 0.5rem;
  grid-template-columns: auto minmax(0, 1fr) auto;
  padding: 0.55rem;
}
.source-row div {
  display: grid;
  gap: 0.15rem;
  min-width: 0;
}
.source-row strong {
  font-size: 0.72rem;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}
.source-row span {
  color: var(--text-tertiary);
  font-size: 0.625rem;
}
.source-row__remove {
  height: 1.5rem;
  padding: 0;
  width: 1.5rem;
}
.output-panel {
  border-top: 1px solid var(--border);
  margin-top: auto;
  padding-top: 1rem;
}
label {
  color: var(--text-secondary);
  display: grid;
  font-size: 0.655rem;
  gap: 0.3rem;
}
.originals-note {
  color: var(--text-tertiary);
}
.split-workspace {
  box-sizing: border-box;
  display: flex;
  flex-direction: column;
  gap: 1rem;
  height: 100%;
  min-height: 0;
  padding: 1rem 1.25rem 1.25rem;
}
.workspace-empty {
  display: grid;
  flex: 1;
  place-items: center;
}
.primary-document-action {
  background: var(--text);
  color: var(--surface);
}
.workspace-toolbar {
  align-items: center;
  display: flex;
  gap: 1rem;
  min-width: 0;
}
.workspace-toolbar > div {
  display: grid;
  gap: 0.2rem;
}
.toolbar-title {
  font-size: 0.75rem;
  font-weight: 650;
}
.toolbar-meta {
  color: var(--text-tertiary);
  display: block;
  font-size: 0.65rem;
  max-width: 26rem;
}
.mode-options {
  align-content: start;
  align-self: start;
  display: grid !important;
  gap: 0.4rem;
  grid-auto-rows: var(--n-height);
  height: auto !important;
  width: 100%;
}
.mode-options :deep(.n-radio-group__splitor) {
  display: none;
}
.mode-options :deep(.n-radio-button) {
  border: 1px solid var(--border);
  border-radius: var(--radius-sm);
  display: block;
  height: var(--n-height) !important;
  line-height: var(--n-height) !important;
  overflow: hidden;
  padding: 0 0.65rem;
  text-overflow: ellipsis;
  white-space: nowrap;
  width: 100%;
}
.selection-strip {
  align-items: center;
  background: var(--surface);
  border: 1px solid var(--border);
  border-radius: var(--radius-sm);
  color: var(--text-secondary);
  display: flex;
  font-size: 0.7rem;
  justify-content: space-between;
  padding: 0.5rem 0.65rem;
}
.groups-strip {
  display: flex;
  flex-wrap: wrap;
  gap: 0.35rem;
}
.group-token {
  background: var(--accent-soft);
  border-radius: var(--radius-sm);
  color: var(--accent);
  font-size: 0.65rem;
  padding: 0.3rem 0.45rem;
}
.group-token button {
  background: none;
  border: 0;
  color: inherit;
  cursor: pointer;
  font: inherit;
  padding: 0 0 0 0.25rem;
}
.thumbnail-workspace {
  flex: 1;
  min-height: 0;
  min-width: 0;
  overflow: auto;
}
.thumbnail-grid {
  display: grid;
  gap: 1rem;
  grid-template-columns: repeat(auto-fill, minmax(8rem, 1fr));
  min-width: 0;
  padding: 1rem 0;
}
.page-thumbnail {
  background: transparent;
  border: 0;
  color: var(--text);
  cursor: pointer;
  padding: 0;
  position: relative;
  text-align: left;
}
.page-thumbnail:disabled {
  cursor: default;
  opacity: 0.8;
}
.page-thumbnail img,
.thumbnail-placeholder {
  aspect-ratio: 3/4;
  background: #fff;
  border: 1px solid #d8d8d8;
  border-radius: 2px;
  box-shadow: 0 8px 18px rgb(0 0 0 / 5.5%);
  display: block;
  object-fit: contain;
  width: 100%;
}
.page-thumbnail.selected img,
.page-thumbnail.selected .thumbnail-placeholder {
  box-shadow:
    0 0 0 2px var(--accent),
    0 10px 22px rgb(0 0 0 / 6.5%);
}
.page-thumbnail__number {
  background: var(--surface);
  border: 1px solid var(--border);
  border-radius: 50%;
  font-size: 0.6rem;
  height: 1.25rem;
  left: 0.4rem;
  line-height: 1.25rem;
  position: absolute;
  text-align: center;
  top: 0.4rem;
  width: 1.25rem;
  z-index: 1;
}
.page-thumbnail.selected .page-thumbnail__number {
  background: var(--accent);
  border-color: var(--accent);
  color: #fff;
}
.page-thumbnail:not(:disabled):hover .page-thumbnail__label {
  color: var(--text);
}
.page-thumbnail__label {
  color: var(--text-secondary);
  display: block;
  font-size: 0.65rem;
  padding-top: 0.45rem;
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
@media (max-width: 52rem) {
  .workspace-toolbar {
    align-items: start;
    flex-direction: column;
  }
  .thumbnail-grid {
    grid-template-columns: repeat(auto-fill, minmax(6.5rem, 1fr));
  }
}
</style>
