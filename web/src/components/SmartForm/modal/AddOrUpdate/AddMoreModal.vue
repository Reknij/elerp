<script setup lang="ts">
import SmartSelect from "../../SmartSelect.vue";
import { getSKUs, catchNumbers } from "../../util";
import { NButton, NInput, useMessage, NModal, NSpace, NCard } from "naive-ui";
import { ref } from "vue";
import { useI18n } from "vue-i18n";
import { OrderItem } from "../../../../api/erp/model";
import { FormRowType } from "../../interfaces";
import CloseButton from "../../../CloseButton.vue";
import { useWindowSize } from "@vueuse/core";

const { t } = useI18n();
const message = useMessage();
defineProps({
  show: {
    type: Boolean,
    required: true,
  },
  title: {
    type: String,
    default: undefined,
  },
});
const defaultTitle = t("action.addMultiple", { obj: t("main.SKUs") });

defineEmits<{
  (e: "update:show", v: boolean): void;
  (e: "update:title", v: string): void;
  (e: "add-clicked", items: OrderItem[] | undefined): void;
}>();

const itemList = ref("");
const prices = ref("");
const category_id = ref<number>();
const { width } = useWindowSize();
</script>

<template>
  <n-modal :show="show">
    <n-card
      :style="width >= 1024 ? { width: '50%' } : {}"
      :title="title ?? defaultTitle"
      :bordered="false"
      size="small"
      role="dialog"
      aria-modal="true"
    >
      <template #header-extra>
        <CloseButton @click="$emit('update:show', false)" />
      </template>
      <NSpace vertical>
        <SmartSelect
          :row="{ type: FormRowType.SKUCategory, key: 'sku_category_id' }"
          :limit="30"
          v-model:value="category_id"
        />
        <n-input
          type="textarea"
          clearable
          :autosize="{
            minRows: 5,
          }"
          :placeholder="`${t('common.orderItem')} - ${t('common.example')}\n${t(
            'example.addMoreItem'
          )}`"
          v-model:value="itemList"
        ></n-input>
        <n-input
          type="textarea"
          clearable
          :placeholder="`${t('common.price')} - ${t('common.example')}\n${t(
            'example.addMoreItemPrice'
          )}`"
          v-model:value="prices"
        >
        </n-input>
        <NButton @click="prices = catchNumbers(prices)">{{
          t("action.catchNumbers")
        }}</NButton>
      </NSpace>
      <template #footer>
        <NButton
          @click="
            async () => {
              const items = await getSKUs(
                message,
                category_id,
                itemList,
                prices
              );
              $emit('add-clicked', items);
            }
          "
          class="btn m-1"
          >{{ t("action.add") }}</NButton
        >
      </template>
    </n-card>
  </n-modal>
</template>
