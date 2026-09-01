<script setup lang="ts">
import {
  NAlert,
  NButton,
  NCard,
  NCheckbox,
  NEmpty,
  NInput,
  NLayout,
  NLayoutContent,
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
import { useRouter } from "vue-router";

import { useSplitStore } from "../stores/split/useSplitStore";

const { t } = useI18n();
const router = useRouter();
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
  <NLayout class="application-shell">
    <NLayoutContent content-style="padding: 2rem;">
      <main class="split-page">
        <header class="page-heading">
          <div>
            <h1>{{ t("split.heading") }}</h1>
            <NText depth="3">{{ t("split.intro") }}</NText>
          </div>
          <NButton quaternary @click="router.push('/')">{{ t("common.home") }}</NButton>
        </header>

        <NCard v-if="split.phase === 'running'" embedded>
          <NThing :title="t('split.processing')">
            <NText depth="3">{{ t("split.progress", split.progress) }}</NText>
          </NThing>
          <NProgress :percentage="split.progress.percent" indicator-placement="inside" processing />
          <NButton type="error" @click="split.cancelSplit">{{ t("split.cancel") }}</NButton>
        </NCard>

        <template v-else>
          <NAlert v-if="split.errorMessage" type="error" :title="t('split.error')">
            {{ split.errorMessage }}
          </NAlert>
          <NAlert v-if="split.outcome === 'succeeded'" type="success" :title="t('split.success')">
            {{ t("split.successBody") }}
          </NAlert>
          <NAlert v-if="split.outcome === 'cancelled'" type="info" :title="t('split.cancelled')">
            {{ t("split.cancelledBody") }}
          </NAlert>

          <NCard embedded :title="t('split.source')">
            <template v-if="split.source">
              <NThing :title="split.source.name" :description="split.source.path">
                <NText depth="3">{{
                  t("split.pageCount", { count: split.source.pageCount })
                }}</NText>
              </NThing>
              <NSpace>
                <NButton @click="split.choosePdfFile">{{ t("split.replaceSource") }}</NButton>
                <NButton type="error" @click="split.resetPreparation">{{
                  t("split.removeSource")
                }}</NButton>
              </NSpace>
            </template>
            <template v-else>
              <NEmpty :description="t('split.emptySource')">
                <template #extra>
                  <NButton type="primary" @click="split.choosePdfFile">{{
                    t("split.addSource")
                  }}</NButton>
                </template>
              </NEmpty>
              <NText depth="3" class="drop-hint">{{ t("split.dropHint") }}</NText>
            </template>
          </NCard>

          <template v-if="split.source">
            <NCard embedded :title="t('split.mode')">
              <NRadioGroup :value="split.mode" name="split-mode" @update:value="selectMode">
                <NSpace vertical>
                  <NRadioButton value="eachPage">{{ t("split.eachPage") }}</NRadioButton>
                  <NRadioButton value="extract">{{ t("split.extract") }}</NRadioButton>
                  <NRadioButton value="groups">{{ t("split.groups") }}</NRadioButton>
                </NSpace>
              </NRadioGroup>
              <NText depth="3" class="mode-help">{{ t(`split.modeHelp.${split.mode}`) }}</NText>
            </NCard>

            <NCard embedded :title="t('split.pages')">
              <NText depth="3">{{ t("split.thumbnailHint") }}</NText>
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
                  >
                    {{ t("split.page", { page }) }}
                  </NCheckbox>
                </NCard>
              </div>
              <NButton
                v-if="split.canLoadMoreThumbnails"
                :loading="split.thumbnailsLoading"
                @click="split.loadNextThumbnails"
              >
                {{ t("split.loadMore") }}
              </NButton>
            </NCard>

            <NCard v-if="split.mode === 'extract'" embedded :title="t('split.extractSelection')">
              <NText depth="3">{{
                t("split.selectedPages", { pages: split.selectedPages.join(", ") || "—" })
              }}</NText>
              <NButton :disabled="!split.selectedPages.length" @click="split.clearSelectedPages">
                {{ t("split.clearSelection") }}
              </NButton>
            </NCard>

            <NCard v-if="split.mode === 'groups'" embedded :title="t('split.pageGroups')">
              <NText depth="3">{{
                t("split.selectedPages", { pages: split.selectedPages.join(", ") || "—" })
              }}</NText>
              <NSpace>
                <NButton
                  type="primary"
                  :disabled="!split.selectedPages.length"
                  @click="split.createGroupFromSelection"
                >
                  {{ t("split.createGroup") }}
                </NButton>
                <NButton :disabled="!split.selectedPages.length" @click="split.clearSelectedPages">
                  {{ t("split.clearSelection") }}
                </NButton>
              </NSpace>
              <NEmpty v-if="!split.groups.length" :description="t('split.emptyGroups')" />
              <NList v-else bordered>
                <NListItem v-for="group in groupSummary" :key="group.index">
                  <NTag type="info">{{ t("split.group", { number: group.index }) }}</NTag>
                  {{ group.pages }}
                  <template #suffix>
                    <NButton
                      quaternary
                      type="error"
                      size="small"
                      @click="split.removeGroup(group.id)"
                    >
                      {{ t("split.remove") }}
                    </NButton>
                  </template>
                </NListItem>
              </NList>
            </NCard>

            <NCard embedded :title="t('split.destination')">
              <label>
                {{ t("split.outputName") }}
                <NInput
                  :value="split.outputName"
                  :placeholder="t('split.outputPlaceholder')"
                  @update:value="split.renameOutput"
                />
              </label>
              <label>
                {{ t("split.destinationPath") }}
                <NInput
                  :value="split.destination"
                  :placeholder="t('split.destinationPlaceholder')"
                  @update:value="split.chooseDestination"
                />
              </label>
              <NButton @click="split.chooseDestinationFolder">{{ t("split.browse") }}</NButton>
            </NCard>

            <NButton
              block
              type="primary"
              size="large"
              :disabled="!split.canRequestSummary"
              @click="openSummary"
            >
              {{ t("split.review") }}
            </NButton>
          </template>
        </template>
      </main>
    </NLayoutContent>
  </NLayout>

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
.application-shell {
  min-height: 100vh;
}

.split-page {
  display: grid;
  gap: 1rem;
  margin: 0 auto;
  max-width: 68rem;
}

.page-heading {
  align-items: start;
  display: flex;
  gap: 1rem;
  justify-content: space-between;
}

h1 {
  margin: 0;
}

label {
  display: grid;
  gap: 0.4rem;
  margin-bottom: 0.8rem;
}

.drop-hint,
.mode-help {
  display: block;
  margin-top: 0.8rem;
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
