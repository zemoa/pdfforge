<script setup lang="ts">
import {
  NAlert,
  NButton,
  NCard,
  NCheckbox,
  NEmpty,
  NInput,
  NList,
  NListItem,
  NModal,
  NProgress,
  NRadioButton,
  NRadioGroup,
  NSpace,
  NTag,
  NText,
  NThing,
} from "naive-ui";
import { computed, onBeforeUnmount, onMounted, ref } from "vue";
import { useI18n } from "vue-i18n";

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
      <section class="panel-section">
        <NText strong>{{ t("split.source") }}</NText>
        <template v-if="split.source">
          <NThing :title="split.source.name" :description="split.source.path">
            <NText depth="3">{{ t("split.pageCount", { count: split.source.pageCount }) }}</NText>
          </NThing>
          <NButton block :disabled="split.phase === 'running'" @click="split.choosePdfFile">{{
            t("split.replaceSource")
          }}</NButton>
          <NButton
            block
            type="error"
            :disabled="split.phase === 'running'"
            @click="split.removeSource"
            >{{ t("split.removeSource") }}</NButton
          >
        </template>
        <template v-else>
          <NText depth="3" class="drop-hint">{{ t("split.dropHint") }}</NText>
          <NButton
            type="primary"
            block
            :disabled="split.phase === 'running'"
            @click="split.choosePdfFile"
            >{{ t("split.addSource") }}</NButton
          >
        </template>
      </section>
    </template>

    <template #right-panel>
      <section class="panel-section">
        <NText strong>{{ t("split.destination") }}</NText>
        <label
          >{{ t("split.outputName")
          }}<NInput
            :value="split.outputName"
            :disabled="split.phase === 'running'"
            :placeholder="t('split.outputPlaceholder')"
            @update:value="split.renameOutput"
        /></label>
        <label
          >{{ t("split.destinationPath")
          }}<NInput
            :value="split.destination"
            :disabled="split.phase === 'running'"
            :placeholder="t('split.destinationPlaceholder')"
            @update:value="split.chooseDestination"
        /></label>
        <NButton :disabled="split.phase === 'running'" @click="split.chooseDestinationFolder">{{
          t("split.browse")
        }}</NButton>
      </section>
    </template>

    <section class="split-workspace">
      <NCard v-if="split.phase === 'running'" embedded class="process-card">
        <NThing :title="t('split.processing')"
          ><NText depth="3">{{ t("split.progress", split.progress) }}</NText></NThing
        >
        <NProgress :percentage="split.progress.percent" indicator-placement="inside" processing />
        <NButton type="error" @click="split.cancelSplit">{{ t("split.cancel") }}</NButton>
      </NCard>

      <template v-else>
        <NAlert v-if="split.errorMessage" type="error" :title="t('split.error')">{{
          split.errorMessage
        }}</NAlert>
        <NAlert v-if="split.outcome === 'succeeded'" type="success" :title="t('split.success')">{{
          t("split.successBody")
        }}</NAlert>
        <NAlert v-if="split.outcome === 'cancelled'" type="info" :title="t('split.cancelled')">{{
          t("split.cancelledBody")
        }}</NAlert>

        <NEmpty
          v-if="!split.source"
          class="workspace-empty"
          :description="t('split.emptySource')"
        />

        <template v-else>
          <div class="split-toolbar">
            <div>
              <NText strong>{{ t("split.mode") }}</NText>
              <NText depth="3" class="mode-help">{{ t(`split.modeHelp.${split.mode}`) }}</NText>
            </div>
            <NRadioGroup :value="split.mode" name="split-mode" @update:value="selectMode">
              <NRadioButton value="eachPage">{{ t("split.eachPage") }}</NRadioButton>
              <NRadioButton value="extract">{{ t("split.extract") }}</NRadioButton>
              <NRadioButton value="groups">{{ t("split.groups") }}</NRadioButton>
            </NRadioGroup>
          </div>

          <div v-if="split.mode !== 'eachPage'" class="selection-bar">
            <NText depth="3">{{
              t("split.selectedPages", { pages: split.selectedPages.join(", ") || "—" })
            }}</NText>
            <NSpace>
              <NButton
                v-if="split.mode === 'groups'"
                type="primary"
                :disabled="!split.selectedPages.length"
                @click="split.createGroupFromSelection"
                >{{ t("split.createGroup") }}</NButton
              >
              <NButton :disabled="!split.selectedPages.length" @click="split.clearSelectedPages">{{
                t("split.clearSelection")
              }}</NButton>
            </NSpace>
          </div>

          <NList v-if="split.mode === 'groups' && split.groups.length" bordered class="group-list">
            <NListItem v-for="group in groupSummary" :key="group.index">
              <NTag type="info">{{ t("split.group", { number: group.index }) }}</NTag>
              {{ group.pages }}
              <template #suffix
                ><NButton
                  quaternary
                  type="error"
                  size="small"
                  @click="split.removeGroup(group.id)"
                  >{{ t("split.remove") }}</NButton
                ></template
              >
            </NListItem>
          </NList>

          <div class="thumbnail-workspace">
            <div class="workspace-caption">
              <NText depth="3">{{ t("split.thumbnailHint") }}</NText
              ><NText depth="3">{{ t("split.pages") }}</NText>
            </div>
            <div class="thumbnail-grid">
              <NCard v-for="page in split.displayedPages" :key="page" embedded size="small">
                <img
                  v-if="split.thumbnails[page]"
                  :src="split.thumbnails[page]"
                  :alt="t('split.pageThumbnail', { page })"
                />
                <div v-else class="thumbnail-placeholder">{{ page }}</div>
                <NCheckbox
                  :checked="split.selectedPages.includes(page)"
                  :disabled="split.mode === 'eachPage' || split.assignedPages.includes(page)"
                  @update:checked="split.togglePage(page)"
                  >{{ t("split.page", { page }) }}</NCheckbox
                >
              </NCard>
            </div>
            <NButton
              v-if="split.canLoadMoreThumbnails"
              :loading="split.thumbnailsLoading"
              @click="split.loadNextThumbnails"
              >{{ t("split.loadMore") }}</NButton
            >
          </div>
        </template>
      </template>
    </section>

    <template #footer>
      <NButton
        block
        type="primary"
        size="large"
        :disabled="split.phase === 'running' || !split.canRequestSummary"
        @click="openSummary"
        >{{ t("split.review") }}</NButton
      >
    </template>
  </ToolWorkspaceShell>

  <NModal
    v-model:show="showSummary"
    preset="card"
    :title="t('split.summaryTitle')"
    style="width: min(92vw, 42rem)"
  >
    <NList bordered>
      <NListItem v-for="path in split.outputPreview?.outputPaths" :key="path">{{ path }}</NListItem>
    </NList>
    <template #action>
      <NSpace justify="end">
        <NButton @click="showSummary = false">{{ t("split.back") }}</NButton>
        <NButton
          type="primary"
          @click="
            showSummary = false;
            split.confirmSplit();
          "
        >
          {{ t("split.confirm") }}
        </NButton>
      </NSpace>
    </template>
  </NModal>
</template>

<style scoped>
.split-workspace {
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

.drop-hint,
.mode-help {
  display: block;
  line-height: 1.5;
}

.split-toolbar,
.selection-bar,
.workspace-caption {
  align-items: center;
  display: flex;
  gap: 1rem;
  justify-content: space-between;
}

.split-toolbar :deep(.n-radio-group) {
  display: flex;
  flex-wrap: wrap;
  justify-content: end;
}

.thumbnail-workspace {
  min-height: 0;
  overflow: auto;
}

.workspace-empty {
  display: grid;
  flex: 1;
  place-items: center;
}

.group-list {
  max-height: 10rem;
  overflow: auto;
}

.thumbnail-grid {
  display: grid;
  gap: 0.75rem;
  grid-template-columns: repeat(auto-fill, minmax(9rem, 1fr));
  margin: 1rem 0;
}

.thumbnail-grid img,
.thumbnail-placeholder {
  aspect-ratio: 3 / 4;
  background: var(--n-color-embedded);
  display: block;
  object-fit: contain;
  width: 100%;
}

.thumbnail-placeholder {
  align-items: center;
  display: grid;
  justify-content: center;
}
</style>
