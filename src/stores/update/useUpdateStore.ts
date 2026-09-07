import { computed, ref } from "vue";
import { defineStore } from "pinia";

import { type UpdateCheck, updateClient } from "../../application/updateClient";
import { errorCodeFrom, type ErrorCode } from "../../application/error";

export const useUpdateStore = defineStore("update", () => {
  const installedVersion = ref("");
  const rollbackAvailable = ref(false);
  const updatedTo = ref<string | null>(null);
  const result = ref<UpdateCheck | null>(null);
  const phase = ref<"idle" | "checking" | "downloading">("idle");
  const progress = ref({ downloaded: 0, total: null as number | null });
  const errorCode = ref<ErrorCode | null>(null);
  const manualDownloadVersion = ref<string | null>(null);
  let unlisten: (() => void) | undefined;

  const progressPercent = computed(() =>
    progress.value.total && progress.value.total > 0
      ? Math.round((progress.value.downloaded / progress.value.total) * 100)
      : null,
  );

  async function initialize() {
    const status = await updateClient.status();
    installedVersion.value = status.installedVersion;
    rollbackAvailable.value = status.rollbackAvailable;
    updatedTo.value = status.updatedTo;
    if (status.installationError) errorCode.value = "updateInstallFailed";
    unlisten ??= await updateClient.onUpdateEvent((event) => {
      if (event.type === "progress") {
        progress.value = { downloaded: event.downloaded, total: event.total };
      }
      if (event.type === "cancelled") {
        phase.value = "idle";
      }
      if (event.type === "manualDownload") {
        phase.value = "idle";
        manualDownloadVersion.value = event.version;
      }
      if (event.type === "failed") {
        phase.value = "idle";
        errorCode.value = event.error;
      }
    });
  }

  async function check(locale: string) {
    if (phase.value !== "idle") return;
    phase.value = "checking";
    errorCode.value = null;
    manualDownloadVersion.value = null;
    try {
      result.value = await updateClient.check(locale);
    } catch (error) {
      errorCode.value = errorCodeFrom(error, "updateCheckFailed");
    } finally {
      phase.value = "idle";
    }
  }

  async function install() {
    if (phase.value !== "idle" || result.value?.kind !== "available") return;
    errorCode.value = null;
    manualDownloadVersion.value = null;
    progress.value = { downloaded: 0, total: null };
    try {
      phase.value = "downloading";
      await updateClient.start();
    } catch (error) {
      phase.value = "idle";
      errorCode.value = errorCodeFrom(error, "updateInstallFailed");
    }
  }

  async function cancel() {
    if (phase.value === "downloading") await updateClient.cancel();
  }

  async function restore() {
    if (phase.value !== "idle" || !rollbackAvailable.value) return;
    errorCode.value = null;
    try {
      await updateClient.restore();
    } catch (error) {
      errorCode.value = errorCodeFrom(error, "updateRestoreFailed");
    }
  }

  function dispose() {
    unlisten?.();
    unlisten = undefined;
  }

  return {
    installedVersion,
    rollbackAvailable,
    updatedTo,
    result,
    phase,
    progress,
    progressPercent,
    errorCode,
    manualDownloadVersion,
    initialize,
    check,
    install,
    cancel,
    restore,
    dispose,
  };
});
