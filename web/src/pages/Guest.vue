<script setup lang="ts">
import { ref } from "vue";
import { useI18n } from "vue-i18n";
import {
  NButton,
  NDatePicker,
  NCard,
  NSpace,
  NInput,
  useMessage,
} from "naive-ui";
import { useCached } from "../stores/cached";
import OrderItemList from "../components/SmartForm/modal/AddOrUpdate/OrderItemList.vue";
import { useRoute } from "vue-router";
import { get_guest_order, confirm_guest_order } from "../api/erp";
import { GuestOrder, GuestOrderStatus, OrderType } from "../api/erp/model";
import {
  getCheckOrderResultToStringArray,
  getTitleByFormRow,
} from "../components/SmartForm/util";
import Result from "../components/SmartForm/modal/Result.vue";
import { getGuestOrderStatusElement } from "../composables/GuestOrderStatusElement";
import { error_to_string, fmt_err } from "../AppError";
import LanguageSelect from "../components/LanguageSelect.vue";
import { FormRowType } from "../components/SmartForm/interfaces";
import { getIDElement } from "../composables/IDElement";

const { t } = useI18n();
const message = useMessage();
const route = useRoute();
const cached = useCached();
const id = Number.parseInt(route.params.id as string);
const token = route.query.sub_token as string;
const lastResult = ref<string[][]>([]);
const guestOrder = ref<GuestOrder>();
const showCheckOrderResult = ref(false);

try {
  guestOrder.value = await get_guest_order(id, token);
} catch (error) {}

async function confirmClicked() {
  try {
    if (!isNaN(id) && guestOrder.value) {
      const result = await confirm_guest_order(id, guestOrder.value);
      if (result.check_result.items_not_available.length == 0) {
        guestOrder.value = result.order!;
        console.log(result.order);
        message.success(t("message.addSuccess", { obj: guestOrder.value.id }));
      } else {
        lastResult.value = await getCheckOrderResultToStringArray(
          cached,
          guestOrder.value.order_type,
          result.check_result
        );
        showCheckOrderResult.value = true;
      }
    }
  } catch (error) {
    const msg = fmt_err(error, {
      obj: t("main.order"),
    });
    message.error(msg ?? error_to_string(error));
  }
}
</script>

<template>
  <NCard class="max-w-5xl m-auto">
    <span class="text-4xl">{{ t("main.guestOrder") }}</span>
    <NSpace vertical v-if="guestOrder">
      <Result
        v-model:show="showCheckOrderResult"
        ref="resultRef"
        :title="t('common.result')"
      >
        <h1>{{ t("message.itemsNotAvailable") }}</h1>
        <span v-for="(arr, i) in lastResult">
          <br v-if="i != 0" />
          <span v-for="text in arr">
            {{ text }}
            <br />
          </span>
        </span>
      </Result>

      <NSpace align="center">
        <component v-if="guestOrder" :is="getIDElement(guestOrder.id!)" />
        <n-date-picker
          disabled
          :value="guestOrder.confirmed_date * 1000"
          type="datetime"
        />
        <component
          :is="getGuestOrderStatusElement(guestOrder.guest_order_status)"
        />
      </NSpace>

      <span class="m-2">{{ t("common.description") }}</span>
      <n-input
        class="min-w-full"
        :type="'textarea'"
        :readonly="guestOrder.guest_order_status !== GuestOrderStatus.Pending"
        clearable
        :default-value="guestOrder.description"
        @change="(v) => (guestOrder!.description = v)"
        :placeholder="
          getTitleByFormRow({
            key: 'description',
            type: FormRowType.TextArea,
          })
        "
      ></n-input>
      <OrderItemList
        :order_id="guestOrder.order_id"
        :disable="guestOrder.guest_order_status !== GuestOrderStatus.Pending"
        v-model:items="guestOrder.items"
        :currency="guestOrder.currency"
        :enable_exchange="guestOrder.order_type === OrderType.Exchange"
      />
      <NButton
        v-if="guestOrder.guest_order_status === GuestOrderStatus.Pending"
        @click="confirmClicked"
        >{{ t("action.confirm") }}</NButton
      >
    </NSpace>
    <NSpace vertical v-else>
      <span class="text-6xl">404</span>
    </NSpace>
    <LanguageSelect></LanguageSelect>
  </NCard>
</template>

<style scoped></style>
