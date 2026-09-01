import { createRouter, createWebHashHistory } from "vue-router";

import MergePdfPage from "../pages/MergePdfPage.vue";

export const router = createRouter({
  history: createWebHashHistory(),
  routes: [{ path: "/", component: MergePdfPage }],
});
