<script setup lang="ts">
import Chart from "chart.js/auto";
import { onMounted, watch } from "vue";
import { ref } from "vue";
import { StatisticalData } from "../api/erp/model";
import { useCached } from "../stores/cached";
import { useI18n } from "vue-i18n";
import getSymbolFromCurrency from "currency-symbol-map";

const props = defineProps<{
  data: StatisticalData;
}>();

const { t } = useI18n();
const cached = useCached();
const chart1 = ref(null);
let totalOutInst: Chart | undefined = undefined;
const loading = ref(false);
async function renderChart() {
  loading.value = true;
  const data = {
    labels: await Promise.all(
      props.data.most_popular_skus.map(async (sku) => {
        let name = (await cached.getSKU(sku.id)).name;
        return `${name} - ${getSymbolFromCurrency(sku.currency)} ${(sku.average_price * sku.total_out).toFixed(2)}`;
      })
    ),
    datasets: [
      {
        label: t("common.totalOut"),
        data: props.data.most_popular_skus.map((sku) => sku.total_out),
      },
    ],
  };
  if (chart1.value && !totalOutInst) {
    totalOutInst = new Chart(chart1.value, {
      type: "bar",
      data,
    });
  } else if (totalOutInst) {
    totalOutInst.data = data;
    totalOutInst.update();
  }
  loading.value = false;
}

onMounted(async () => {
  await renderChart();
});
watch(props, async () => await renderChart());
</script>

<template>
  <div>
    <h1 v-if="loading" class="text-6xl">
      {{ t("common.loading") }}
    </h1>
    <div class="chartContainer">
      <canvas ref="chart1"></canvas>
    </div>
  </div>
</template>

<style scoped>
.chartContainer {
  @apply flex justify-center relative w-full max-h-[800px];
}
</style>
