<script setup lang="ts">
import { ref, watch } from "vue";
import { NSpace } from "naive-ui";
import { OrderCurrency, OrderItem } from "../../../../api/erp/model";
import getSymbolFromCurrency from "currency-symbol-map";
import { getItemsResult } from "../../../../util";
import { useI18n } from "vue-i18n";

const { t } = useI18n();
const props = defineProps<{
  value: OrderItem[];
  currency: OrderCurrency;
}>();

const result = ref(getItemsResult(props.value));
watch(props, () => {
  result.value = getItemsResult(props.value);
});
</script>

<template>
  <NSpace align="center">
    <p>
      {{ t("result.itemsResult.totalQuantity") }} {{ result.totalQuantity }},
      {{ t("result.itemsResult.averagePrice") }}
      {{ getSymbolFromCurrency(currency) }}{{ result.averagePrice.toFixed(2) }},
      {{ t("result.itemsResult.totalAmount") }}
      {{ getSymbolFromCurrency(currency) }}{{ result.totalAmount.toFixed(2) }}
    </p>
  </NSpace>
</template>
