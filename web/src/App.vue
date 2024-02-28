<script setup lang="ts">
import {
  NDialogProvider,
  NMessageProvider,
  NConfigProvider,
  NLoadingBarProvider,
  GlobalThemeOverrides,
  dateZhCN,
  zhCN,
} from "naive-ui";
import { ref, watch } from "vue";
import { useI18n } from "vue-i18n";

const { t, locale } = useI18n();
const nLocale = ref();
const nDateLocale = ref();
watch(locale, () => {
  if (locale.value == "cn") {
    nLocale.value = zhCN;
    nDateLocale.value = dateZhCN;
  } else {
    nLocale.value = null;
    nDateLocale.value = null;
  }
});
const themeOverrides: GlobalThemeOverrides = {
  common: {
    textColorDisabled: "#808080",
  },
  Button: {
    color: "#e5e5e5",
  },
  Tabs: {
    tabTextColorActiveCard: "black",
  },
  Pagination: {
    buttonColor: "#e5e5e5",
  },
};
</script>

<template>
  <n-config-provider
    :theme-overrides="themeOverrides"
    :locale="nLocale"
    :date-locale="nDateLocale"
  >
    <n-loading-bar-provider>
      <n-dialog-provider>
        <n-message-provider>
          <Suspense>
            <RouterView></RouterView>
            <template #fallback>
              <h1 class="text-6xl">{{ t("common.loading") }}</h1>
            </template>
          </Suspense>
        </n-message-provider>
      </n-dialog-provider>
    </n-loading-bar-provider>
  </n-config-provider>
</template>
