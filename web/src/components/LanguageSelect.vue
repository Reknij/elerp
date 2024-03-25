<script setup lang="ts">
import { NSelect, SelectOption } from "naive-ui";
import { ref } from "vue";
import { useMySelfUser } from "../stores/me";
import { useI18n } from "vue-i18n";

const myself = useMySelfUser();
const { t } = useI18n();
const selectedLanguage = ref(myself.config?.language ?? "en");
const languages: SelectOption[] = [
  {
    label: "English",
    value: "en",
  },
  {
    label: "简体中文",
    value: "cn",
  },
  {
    label: "Malay",
    value: "malay",
  },
];
</script>

<template>
  <NSelect
    :consistent-menu-width="false"
    class="max-w-[200px] my-3"
    :placeholder="t('common.language')"
    :options="languages"
    :value="selectedLanguage"
    @update-value="
      (v) => {
        selectedLanguage = v;
        const config = myself.config;
        config!.language = v;
        myself.changeConfig(config!);
      }
    "
  ></NSelect>
</template>
