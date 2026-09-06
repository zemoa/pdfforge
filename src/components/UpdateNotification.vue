<script setup lang="ts">
import { useMessage } from "naive-ui";
import { onMounted } from "vue";
import { useI18n } from "vue-i18n";

import { updateClient } from "../application/updateClient";

const message = useMessage();
const { t } = useI18n();

onMounted(() => {
  void updateClient.status().then((status) => {
    if (status.updatedTo) {
      message.success(t("update.updatedBody", { version: status.updatedTo }));
    }
    if (status.installationError) {
      message.error(t("update.installationFailed"));
    }
  });
});
</script>

<template><span /></template>
