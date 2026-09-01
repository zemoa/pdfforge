import { createRouter, createWebHashHistory } from "vue-router";

import MergePdfPage from "../pages/MergePdfPage.vue";
import SplitPdfPage from "../pages/SplitPdfPage.vue";
import WelcomePage from "../pages/WelcomePage.vue";

export const router = createRouter({
  history: createWebHashHistory(),
  routes: [
    { path: "/", component: WelcomePage },
    { path: "/merge", component: MergePdfPage },
    { path: "/split", component: SplitPdfPage },
  ],
});
