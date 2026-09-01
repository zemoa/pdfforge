import { createRouter, createWebHashHistory } from "vue-router";

import WelcomePage from "../pages/WelcomePage.vue";

export const router = createRouter({
  history: createWebHashHistory(),
  routes: [{ path: "/", component: WelcomePage }],
});
