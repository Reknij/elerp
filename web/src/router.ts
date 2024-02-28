import * as VueRouter from "vue-router";
import { useMySelfUser } from "./stores/me";

const routes = [
  { path: "/", component: () => import("./pages/Home.vue") },
  { path: "/login", component: () => import("./pages/Login.vue") },
  { name: 'guest', path: "/guest/:id", component: () => import("./pages/Guest.vue") },
];

const router = VueRouter.createRouter({
  // 4. Provide the history implementation to use. We are using the hash history for simplicity here.
  history: VueRouter.createWebHistory(),
  routes, // short for `routes: routes`
});
router.beforeEach(async (to, from) => {
  if (to.name === 'guest') {
    return;
  }
  const myself = useMySelfUser();
  if (!myself.logined) {
    await myself.refresh();
    if (!myself.logined) {
      if (to.path != "/login" && from.path != "/login") {
        router.replace("/login");
      }
    }
  }
});

export default router;
