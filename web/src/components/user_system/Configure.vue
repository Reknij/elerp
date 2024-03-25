<script setup lang="ts">
import { NSpace, useMessage, NButton } from "naive-ui";
import { ref } from "vue";
import { FormRowType } from "../SmartForm/interfaces";
import { useMySelfUser } from "../../stores/me";
import { useI18n } from "vue-i18n";
import SmartSelect from "../SmartForm/SmartSelect.vue";
import { toRaw } from "vue";

const { t } = useI18n();
const myself = useMySelfUser();
const message = useMessage();

const mutConfig = ref(structuredClone(toRaw(myself.config)));

async function saveIt() {
  await myself.changeConfig(mutConfig.value!);
  message.success(t("message.saveSuccess"));
}
</script>

<template>
  <div>
    <NSpace align="center" class="m-3" vertical>
      <p class="text-2xl">{{ t("common.defaultOption") }}</p>
      <SmartSelect
        :row="{
          type: FormRowType.OrderType,
          key: 'order_type',
        }"
        v-model:value="mutConfig!.defaults.order_type"
      >
      </SmartSelect>
      <SmartSelect
        :row="{
          type: FormRowType.OrderCategory,
          key: 'order_category_id',
        }"
        v-model:value="mutConfig!.defaults.order_category_id"
      >
      </SmartSelect>
      <SmartSelect
        :row="{
          type: FormRowType.Warehouse,
          key: 'warehouse_id',
        }"
        v-model:value="mutConfig!.defaults.warehouse_id"
      >
      </SmartSelect>
      <SmartSelect
        :row="{
          type: FormRowType.Person,
          key: 'person_related_id',
        }"
        v-model:value="mutConfig!.defaults.person_related_id"
      >
      </SmartSelect>
      <SmartSelect
        :row="{
          type: FormRowType.OrderCurrency,
          key: 'order_currency',
        }"
        v-model:value="mutConfig!.defaults.order_currency"
      >
      </SmartSelect>
      <NButton @click="saveIt">{{ t("action.save") }}</NButton>
    </NSpace>
  </div>
</template>

<style scoped></style>
