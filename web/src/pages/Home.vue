<script setup lang="ts">
import { ref } from "vue";
import Nav from "../components/Nav.vue";
import { NavPath, getNavLabel } from "../util";
import Workplace from "../components/Workplace.vue";

import { useI18n } from "vue-i18n";
import { ElTabs, ElTabPane, TabPaneName } from "element-plus";
import { useMySelfUser } from "../stores/me";
import { WebSocketFlag } from "../api/ws/models";
import { useMessage } from "naive-ui";
import router from "../router";
import LoadingCount from "../components/LoadingCount.vue";
import { defineAsyncComponent } from "vue";

const { t } = useI18n();
const myself = useMySelfUser();
const message = useMessage();
const tabs = ref<NavPath[]>([]);
const selectedTab = ref<string>(NavPath.Workplace);
function addTab(nav: NavPath) {
  const index = tabs.value.findIndex((v) => v == nav);
  if (index < 0) {
    tabs.value.push(nav);
  }
  selectedTab.value = nav;
}
function handleClose(key: TabPaneName) {
  const index = tabs.value.findIndex((v) => v == key);
  tabs.value.splice(index, 1);
  if (selectedTab.value == key) selectedTab.value = NavPath.Workplace;
}


const Areas = defineAsyncComponent(() => import("../components/erp/Areas.vue"));
const Persons = defineAsyncComponent(() => import("../components/erp/Persons.vue"));
const Inventory = defineAsyncComponent(() => import("../components/erp/Inventory.vue"));
const Orders = defineAsyncComponent(() => import("../components/erp/Orders.vue"));
const GuestOrders = defineAsyncComponent(() => import("../components/erp/GuestOrders.vue"));
const OrderCategories = defineAsyncComponent(() => import("../components/erp/OrderCategories.vue"));
const OrderPayments = defineAsyncComponent(() => import("../components/erp/OrderPayments.vue"));
const Warehouses = defineAsyncComponent(() => import("../components/erp/Warehouses.vue"));
const SKUCategories = defineAsyncComponent(() => import("../components/erp/SKUCategories.vue"));
const SKUs = defineAsyncComponent(() => import("../components/erp/SKUs.vue"));
const Users = defineAsyncComponent(() => import("../components/user_system/Users.vue"));
const Configure = defineAsyncComponent(() => import("../components/user_system/Configure.vue"));
function getComponent(path: NavPath) {
  switch (path) {
    case NavPath.Areas:
      return Areas;
    case NavPath.Persons:
      return Persons;
    case NavPath.Warehouses:
      return Warehouses;
    case NavPath.SKUCategories:
      return SKUCategories;
    case NavPath.SKUs:
      return SKUs;
    case NavPath.Inventory:
      return Inventory;
    case NavPath.Orders:
      return Orders;
    case NavPath.GuestOrders:
      return GuestOrders;
    case NavPath.OrderCategories:
      return OrderCategories;
    case NavPath.Payments:
      return OrderPayments;
    case NavPath.Users:
      return Users;
    case NavPath.PersonalConfiguration:
      return Configure;
    default:
      break;
  }
}

myself.subscribe((flag) => {
  if (
    flag.isFlag(WebSocketFlag.UserRepeatLogin) &&
    flag.id === myself.authenticated?.user.id
  ) {
    message.warning(t("message.userRepeatLogin"), {
      duration: 3000,
    });
    myself.clearAuthorization();
    router.replace("/login");
  }
});
</script>

<template>
  <Nav @nav-change="addTab">
    <span
      class="text-6xl flex justify-center items-center h-[90vh]"
      v-if="!myself.readyAccess"
      >{{ t("common.loading") }}</span
    >
    <ElTabs
      v-else
      v-model="selectedTab"
      class="m-3 p-4 !bg-neutral-300"
      @tab-remove="handleClose"
    >
      <ElTabPane
        :label="getNavLabel(NavPath.Workplace)"
        :name="NavPath.Workplace"
        :closable="false"
      >
        <div>
          <KeepAlive>
            <Suspense>
              <Workplace></Workplace>
              <template #fallback>
                <LoadingCount />
              </template>
            </Suspense>
          </KeepAlive>
        </div>
      </ElTabPane>
      <ElTabPane
        v-for="tab in tabs"
        :key="tab"
        :label="getNavLabel(tab)"
        :name="tab"
        closable
      >
        <div>
          <KeepAlive>
            <Suspense>
              <component :is="getComponent(tab)" :key="tab"></component>
              <template #fallback>
                <span class="text-xl">{{ t("common.loading") }}</span>
              </template>
            </Suspense>
          </KeepAlive>
        </div>
      </ElTabPane>
    </ElTabs>
  </Nav>
</template>

<style scoped></style>