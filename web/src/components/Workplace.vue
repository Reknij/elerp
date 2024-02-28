<script setup lang="ts">
import { ref } from "vue";
import { clear_cache, get_statistical_data } from "../api/erp";
import { NImage, NSpace, NButton, useDialog, useMessage } from "naive-ui";
import { useI18n } from "vue-i18n";
import { useMySelfUser } from "../stores/me";
import { error_to_string, fmt_err } from "../AppError";
import Statistics from "./Statistics.vue";
import getSymbolFromCurrency from "currency-symbol-map";
import MyDatePicker from "./MyDatePicker.vue";
import { GetStatisticalDataQuery } from "../api/erp/model";
import {
  dateRangeConvertBackend,
  getStartAndEndTimestampCurrentMonth,
  getStartAndEndTimestampToday,
} from "../util";
import SmartSelect from "./SmartForm/SmartSelect.vue";
import { FormRowType } from "./SmartForm/interfaces";
import { computed } from "vue";
import LanguageSelect from "./LanguageSelect.vue";
import LoadingCount from "./LoadingCount.vue";

const { t } = useI18n();
const myself = useMySelfUser();
const message = useMessage();
const dialog = useDialog();
const currentMonthTimestamp = getStartAndEndTimestampCurrentMonth();
const query = ref<GetStatisticalDataQuery>({
  date_start: Math.round(currentMonthTimestamp[0] / 1000),
  date_end: Math.round(currentMonthTimestamp[1] / 1000),
});
const statistics = ref();
const loading = ref(true);

const statisticsDate = computed(() => {
  if (query.value.date_start && query.value.date_end) {
    const start = new Date(query.value.date_start).getTime();
    const end = new Date(query.value.date_end).getTime();
    const todayRange = dateRangeConvertBackend(getStartAndEndTimestampToday());
    const currentMonthRange = dateRangeConvertBackend(
      getStartAndEndTimestampCurrentMonth()
    );
    if (todayRange[0] == start && todayRange[1] == end) {
      return t("date.today");
    } else if (currentMonthRange[0] == start && currentMonthRange[1] == end) {
      return t("date.currentMonth");
    }
  }
  return t("date.specifiedDate");
});

const clearCacheClicked = () => {
  dialog.error({
    title: t("common.confirmTitle"),
    content: t("message.askClearCache"),
    positiveText: t("action.yes"),
    negativeText: t("action.no"),
    async onPositiveClick() {
      try {
        if (await clear_cache()) {
          message.success(t("message.clearCacheSuccess"));
        } else {
          message.error(t("message.clearCacheFail"));
        }
      } catch (error) {
        const msg = fmt_err(error, {
          obj: t("main.order"),
        });
        message.error(msg ?? error_to_string(error));
      }
    },
  });
};

const refresh = async () => {
  loading.value = true;
  statistics.value = await get_statistical_data(query.value);
  loading.value = false;
};

refresh();
myself.subscribe(async (flag) => {
  // refresh if any update
  if (flag.id) {
    statistics.value = await get_statistical_data(query.value);
  }
});
</script>

<template>
  <LoadingCount v-if="loading" />
  <NSpace vertical v-else>
    <NSpace justify="center" align="center">
      <n-image width="64" src="/elerp_logo.svg" />
      <h1 class="text-4xl m-3 text-center">{{ t("main.workplace") }}</h1>
    </NSpace>
    <div class="stats shadow bg-gray-200 w-full">
      <div class="stat">
        <div class="stat-title">
          {{
            t("common.orderRemark", {
              remark: statisticsDate,
            })
          }}
        </div>
        <div class="stat-value">
          {{ statistics.order.total_count.stock_in_count }}
        </div>
        <div class="stat-desc">
          {{ `${t("orderType.stockIn")} - ${t("common.quantity")}` }}
        </div>
      </div>
      <div class="stat">
        <div class="stat-title">
          {{
            t("common.orderRemark", {
              remark: statisticsDate,
            })
          }}
        </div>
        <div class="stat-value">
          {{ statistics.order.total_count.stock_out_count }}
        </div>
        <div class="stat-desc">
          {{ `${t("orderType.stockOut")} - ${t("common.quantity")}` }}
        </div>
      </div>
      <div class="stat">
        <div class="stat-title">
          {{
            t("common.orderRemark", {
              remark: statisticsDate,
            })
          }}
        </div>
        <div class="stat-value">
          {{ statistics.order.total_count.exchange_count }}
        </div>
        <div class="stat-desc">
          {{ `${t("orderType.exchange")} - ${t("common.quantity")}` }}
        </div>
      </div>
      <div class="stat">
        <div class="stat-title">
          {{
            t("common.orderRemark", {
              remark: statisticsDate,
            })
          }}
        </div>
        <div class="stat-value">
          {{ statistics.order.total_count.return_count }}
        </div>
        <div class="stat-desc">
          {{ `${t("orderType.return")} - ${t("common.quantity")}` }}
        </div>
      </div>
      <div class="stat">
        <div class="stat-title">
          {{
            t("common.orderRemark", {
              remark: statisticsDate,
            })
          }}
        </div>
        <div class="stat-value">
          {{ statistics.order.total_count.calibration_count }}
        </div>
        <div class="stat-desc">
          {{ `${t("orderType.calibration")} - ${t("common.quantity")}` }}
        </div>
      </div>
    </div>

    <div
      class="stats shadow bg-gray-200 w-full"
      v-if="statistics.order.total_amount.length == 0"
    >
      <div class="stat">
        <div class="stat-title">
          {{
            t("common.salesAmount", {
              remark: statisticsDate,
            })
          }}
        </div>
        <div class="stat-value">
          {{ t("common.empty") }}
        </div>
        <div class="stat-desc">{{ t("common.totalAmount") }}</div>
      </div>
    </div>
    <div
      class="stats shadow bg-gray-200 w-full"
      v-else
      v-for="s in statistics.order.total_amount"
    >
      <div class="stat">
        <div class="stat-title">
          {{
            t("common.salesAmount", {
              remark: statisticsDate,
            })
          }}
        </div>
        <div class="stat-value">
          {{ getSymbolFromCurrency(s.currency) }}
          {{ s.any.toFixed(2) }}
        </div>
        <div class="stat-desc">
          {{ `${s.currency} - ${t("common.totalAmount")}` }}
        </div>
      </div>
      <div class="stat">
        <div class="stat-title">
          {{
            t("common.salesAmount", {
              remark: statisticsDate,
            })
          }}
        </div>
        <div class="stat-value">
          {{ getSymbolFromCurrency(s.currency) }}
          {{ s.settled.toFixed(2) }}
        </div>
        <div class="stat-desc">
          {{ `${s.currency} - ${t("common.totalAmountSettled")}` }}
        </div>
      </div>
      <div class="stat">
        <div class="stat-title">
          {{
            t("common.salesAmount", {
              remark: statisticsDate,
            })
          }}
        </div>
        <div class="stat-value">
          {{ getSymbolFromCurrency(s.currency) }}
          {{ s.partial_settled.toFixed(2) }}
        </div>
        <div class="stat-desc">
          {{ `${s.currency} - ${t("common.totalAmountPartialSettled")}` }}
        </div>
      </div>
      <div class="stat">
        <div class="stat-title">
          {{
            t("common.salesAmount", {
              remark: statisticsDate,
            })
          }}
        </div>
        <div class="stat-value">
          {{ getSymbolFromCurrency(s.currency) }}
          {{ s.unsettled.toFixed(2) }}
        </div>
        <div class="stat-desc">
          {{ `${s.currency} - ${t("common.totalAmountUnsettled")}` }}
        </div>
      </div>
    </div>
    <div class="stats shadow bg-gray-200 w-full">
      <div class="stat">
        <div class="stat-title">{{ t("main.warehouses") }}</div>
        <div class="stat-value">{{ statistics.warehouse_count }}</div>
        <div class="stat-desc">{{ t("common.quantity") }}</div>
      </div>
      <div class="stat">
        <div class="stat-title">{{ t("main.areas") }}</div>
        <div class="stat-value">{{ statistics.area_count }}</div>
        <div class="stat-desc">{{ t("common.quantity") }}</div>
      </div>
      <div class="stat">
        <div class="stat-title">{{ t("main.persons") }}</div>
        <div class="stat-value">{{ statistics.person_count }}</div>
        <div class="stat-desc">{{ t("common.quantity") }}</div>
      </div>
      <div class="stat">
        <div class="stat-title">{{ t("main.SKUCategories") }}</div>
        <div class="stat-value">{{ statistics.sku_category_count }}</div>
        <div class="stat-desc">{{ t("common.quantity") }}</div>
      </div>
      <div class="stat">
        <div class="stat-title">{{ t("main.SKUs") }}</div>
        <div class="stat-value">{{ statistics.sku_count }}</div>
        <div class="stat-desc">{{ t("common.quantity") }}</div>
      </div>
    </div>
    <NSpace>
      <SmartSelect
        :row="{ type: FormRowType.Warehouse, key: 'warehouse_ids' }"
        v-model:value="query.warehouse_ids"
        multiple
        @confirm="refresh"
      ></SmartSelect>
      <SmartSelect
        :row="{
          type: FormRowType.Person,
          key: 'person_related_id',
        }"
        v-model:value="query.person_related_id"
        @confirm="refresh"
      ></SmartSelect>
      <SmartSelect
        :row="{
          type: FormRowType.Person,
          key: 'person_in_charge_id',
        }"
        v-model:value="query.person_in_charge_id"
        @confirm="refresh"
      ></SmartSelect>
      <SmartSelect
        :row="{ type: FormRowType.OrderStatus, key: 'order_status_id' }"
        v-model:value="query.order_status_id"
        @confirm="refresh"
      ></SmartSelect>
      <SmartSelect
        :row="{ type: FormRowType.OrderCurrency, key: 'currency' }"
        v-model:value="query.currency"
        @confirm="refresh"
      ></SmartSelect>
      <MyDatePicker
        v-model:date_start="query.date_start"
        v-model:date_end="query.date_end"
        @confirm="
          async (start, end) => {
            query.date_start = start;
            query.date_end = end;
            await refresh();
          }
        "
      />
    </NSpace>
    <Statistics :data="statistics"> </Statistics>
    <NSpace align="center">
      <LanguageSelect></LanguageSelect>
      <NButton :type="'error'" @click="clearCacheClicked">{{
        t("action.clearCache")
      }}</NButton>
    </NSpace>
  </NSpace>
</template>
