<script setup lang="ts">
import { NavPath, getNavLabel } from "../util";
import { useRouter } from "vue-router";
import { useDialog, useMessage, NSpace, NTooltip, NTag } from "naive-ui";
import { useMySelfUser } from "../stores/me";
import { useI18n } from "vue-i18n";
import { UserType } from "../api/user_system/models";

const { t } = useI18n();

let emit = defineEmits<{
  (e: "navChange", p: NavPath): void;
}>();
const router = useRouter();
const dialog = useDialog();
const message = useMessage();
const myself = useMySelfUser();

async function logout() {
  if (myself.logined) {
    dialog.warning({
      title: t("action.logout"),
      content: t("message.logoutNow", {
        obj: myself.authenticated!.user.alias,
      }),
      positiveText: t("action.yes"),
      negativeText: t("action.no"),
      onPositiveClick: async () => {
        myself.clearAuthorization();
        message.success(t("message.logoutSuccess"))
        router.replace("/login");
      },
    });
  } else {
    message.error(t("message.pleaseLogin"));
  }
}
</script>

<template>
  <div>
    <div class="drawer">
      <input id="my-drawer" type="checkbox" class="drawer-toggle" />
      <div class="drawer-content bg-neutral-300">
        <div class="navbar bg-neutral-200 shadow-md shadow-gray">
          <div class="flex-none">
            <label class="btn btn-square btn-ghost" for="my-drawer">
              <svg
                xmlns="http://www.w3.org/2000/svg"
                fill="none"
                viewBox="0 0 24 24"
                class="inline-block w-5 h-5 stroke-current"
              >
                <path
                  stroke-linecap="round"
                  stroke-linejoin="round"
                  stroke-width="2"
                  d="M4 6h16M4 12h16M4 18h16"
                ></path>
              </svg>
            </label>
          </div>
          <div class="flex-1">
            <NSpace align="center" justify="center">
              <span class="text-3xl">ELERP WEB</span>
              <NTag size="small" :type="'info'" :bordered="false">{{ myself.authenticated?.user.alias }}</NTag>
              <n-tooltip trigger="hover">
                <template #trigger>
                  <span
                    :class="myself.readyAccess ? 'greenDot' : 'redDot'"
                  ></span>
                </template>
                <p>{{ t(myself.readyAccess ?"message.connectedToServer": "message.notConnectedToServer") }}</p>
              </n-tooltip>
            </NSpace>
          </div>
        </div>
        <slot></slot>
      </div>
      <div class="drawer-side z-10">
        <label
          for="my-drawer"
          aria-label="close sidebar"
          class="drawer-overlay"
        ></label>
        <ul class="menu p-4 w-80 min-h-full bg-neutral-200 text-base-content">
          <!-- Sidebar content here -->
          <li>
            <details open>
              <summary>{{ t("main.crm") }}</summary>
              <ul>
                <li @click="emit('navChange', NavPath.Areas)">
                  <a>{{ getNavLabel(NavPath.Areas) }}</a>
                </li>
                <li @click="emit('navChange', NavPath.Persons)">
                  <a>{{ getNavLabel(NavPath.Persons) }}</a>
                </li>
              </ul>
            </details>
          </li>
          <li>
            <details open>
              <summary>{{ t("main.wms") }}</summary>
              <ul>
                <li @click="emit('navChange', NavPath.Warehouses)">
                  <a>{{ getNavLabel(NavPath.Warehouses) }}</a>
                </li>
                <li @click="emit('navChange', NavPath.SKUCategories)">
                  <a>{{ getNavLabel(NavPath.SKUCategories) }}</a>
                </li>
                <li @click="emit('navChange', NavPath.SKUs)">
                  <a>{{ getNavLabel(NavPath.SKUs) }}</a>
                </li>
                <li @click="emit('navChange', NavPath.Inventory)">
                  <a>{{ getNavLabel(NavPath.Inventory) }}</a>
                </li>
                <li @click="emit('navChange', NavPath.OrderCategories)">
                  <a>{{ getNavLabel(NavPath.OrderCategories) }}</a>
                </li>
                <li @click="emit('navChange', NavPath.Orders)">
                  <a>{{ getNavLabel(NavPath.Orders) }}</a>
                </li>
                <li @click="emit('navChange', NavPath.GuestOrders)">
                  <a>{{ getNavLabel(NavPath.GuestOrders) }}</a>
                </li>
                <li @click="emit('navChange', NavPath.Payments)">
                  <a>{{ getNavLabel(NavPath.Payments) }}</a>
                </li>
              </ul>
            </details>
          </li>
          <li>
            <details open>
              <summary>{{ t("main.userSystem") }}</summary>
              <ul>
                <li @click="emit('navChange', NavPath.Users)" v-if="myself.authenticated?.user.user_type === UserType.Admin">
                  <a>{{ getNavLabel(NavPath.Users) }}</a>
                </li>
                <li @click="emit('navChange', NavPath.PersonalConfiguration)">
                  <a>{{ getNavLabel(NavPath.PersonalConfiguration) }}</a>
                </li>
                <li @click="logout">
                  <a>{{ t("action.logout") }}</a>
                </li>
              </ul>
            </details>
          </li>
        </ul>
      </div>
    </div>
  </div>
</template>

<style scoped>
.redDot {
  height: 8px;
  width: 8px;
  background-color: red;
  border-color: black;
  border-radius: 100%;
  display: inline-block;
}
.greenDot {
  height: 8px;
  width: 8px;
  background-color: rgb(0, 255, 0);
  border-color: black;
  border-radius: 100%;
  display: inline-block;
}
</style>
