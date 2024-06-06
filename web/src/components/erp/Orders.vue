<script setup lang="ts">
import {
  useMessage,
  NSpace,
  NButton,
  NButtonGroup,
  NInputNumber,
  useDialog,
  useLoadingBar,
  NCheckbox,
} from "naive-ui";
import { ref } from "vue";
import {
  remove_order,
  get_orders,
  add_order,
  check_order,
  update_order,
  recalc_orders,
  clear_orders,
} from "../../api/erp";
import {
  Order,
  GetOrdersQuery,
  OrderType,
  OrderCurrency,
  OrderPaymentStatus,
} from "../../api/erp/model";
import AddOrUpdateModal from "../SmartForm/modal/AddOrUpdate/Modal.vue";
import { FormRow, FormRowType, ModalType } from "../SmartForm/interfaces";
import SmartTable from "../SmartForm/SmartTable.vue";
import Result from "../SmartForm/modal/Result.vue";
import SmartSelect from "../SmartForm/SmartSelect.vue";
import { useMySelfUser } from "../../stores/me";
import { WebSocketFlag } from "../../api/ws/models";
import { useCached } from "../../stores/cached";
import { ComponentInstance, getOrderTypeText } from "../../util";
import { error_to_string, fmt_err } from "../../AppError";
import { useI18n } from "vue-i18n";
import MyDatePicker from "../MyDatePicker.vue";
import SmartCheckbox from "../SmartForm/SmartCheckbox.vue";
import { getCheckOrderResultToStringArray } from "../SmartForm/util";
import { watch } from "vue";

const { t } = useI18n();
const cached = useCached();
const myself = useMySelfUser();
const loadingBar = useLoadingBar();
const message = useMessage();
const dialog = useDialog();
const modalRef = ref<ComponentInstance<typeof AddOrUpdateModal<Order>>>();
const tableRef = ref<ComponentInstance<typeof SmartTable<Order>>>();
const refreshRows = (page = 1) => tableRef.value?.$.exposed?.refreshRows(page);
const refreshRow = (id: number, value: any) =>
  tableRef.value?.$.exposed?.refreshRow(id, value);

const lastResult = ref<string[][]>([]);
const showCheckOrderResult = ref(false);

function initTemplate() {
  return {
    id: 0,
    date: 0,
    order_type: myself.config?.defaults.order_type ?? OrderType.StockOut,
    order_category_id: myself.config?.defaults.order_category_id ?? 0,
    warehouse_id: myself.config?.defaults.warehouse_id ?? 0,
    description: "",
    person_related_id: myself.config?.defaults.person_related_id ?? 0,
    currency: myself.config?.defaults.order_currency ?? OrderCurrency.Unknown,
    total_amount: 0,
    total_amount_settled: 0,
    order_payment_status: OrderPaymentStatus.None,
    items: [],
  };
}

let to_add_template: Order = initTemplate();

const form: FormRow[] = [
  {
    key: "id",
    type: FormRowType.ID,
    disabled: true,
  },
  {
    key: "from_guest_order_id",
    type: FormRowType.FromGuestOrder,
    disabled: true,
    onlyModal: true,
    visibleIf(row) {
      return row.from_guest_order_id > 0;
    },
  },
  {
    key: "created_by_user_id",
    type: FormRowType.User,
    disabled: true,
    onlyModal: true,
  },
  {
    key: "updated_by_user_id",
    type: FormRowType.User,
    disabled: true,
    onlyModal: true,
  },
  {
    key: "date",
    type: FormRowType.Date,
    sorter: "descend",
    disabled: true,
  },
  {
    key: "last_updated_date",
    type: FormRowType.Date,
    disabled: true,
    onlyModal: true,
  },
  {
    key: "order_type",
    type: FormRowType.OrderType,
    noUpdate: true,
  },
  {
    key: "is_record",
    type: FormRowType.CheckBox,
    onlyModal: true,
    noUpdate: true,
  },
  {
    key: "non_payment",
    type: FormRowType.CheckBox,
    onlyModal: true,
    noUpdate: true,
  },
  {
    key: "order_category_id",
    type: FormRowType.OrderCategory,
  },
  {
    key: "order_payment_status",
    type: FormRowType.OrderPaymentStatus,
    disabled: true,
  },
  {
    key: "warehouse_id",
    type: FormRowType.Warehouse,
    noUpdate: true,
  },
  {
    key: "person_related_id",
    type: FormRowType.Person,
  },
  {
    key: "description",
    type: FormRowType.TextArea,
  },
  {
    key: "currency",
    type: FormRowType.OrderCurrency,
  },
  {
    key: "items",
    type: FormRowType.OrderItems,
    opt: {
      orderIdKey: "id",
    },
    noUpdate: true,
    onlyModal: true,
  },
  {
    key: "total_amount",
    type: FormRowType.TotalAmount,
    disabled: true,
  },
  {
    key: "total_amount_settled",
    type: FormRowType.TotalAmount,
    disabled: true,
  },
];

const query = ref<GetOrdersQuery>({
  index: 0,
  limit: 30,
  sorters: [],
  reverse: new Set(),
});
async function queryCallback(p: number, sorters: string[]) {
  query.value.index = p - 1;
  query.value.sorters = sorters;
  return await get_orders(query.value);
}

async function confirmClicked(n: Order, mt: ModalType) {
  try {
    if (mt == ModalType.Add) {
      if (!(await checkAndShowOrder(n))) {
        to_add_template = n;
        return;
      }
      let order = await add_order(n);
      message.success(t("message.addSuccess", { obj: order.id }));
    } else if (mt == ModalType.Update) {
      await update_order(n.id!, n);
      message.success(t("message.updateSuccess", { obj: n.id }));
    }
    await refreshRows();
  } catch (error: any) {
    const msg = fmt_err(error, {
      obj: t("main.order"),
    });
    message.error(msg ?? error_to_string(error));
  }

  return;
}

async function checkAndShowOrder(n: Order): Promise<boolean> {
  try {
    const cor = await check_order(n);
    if (cor.items_not_available.length == 0) {
      return true;
    }
    lastResult.value = await getCheckOrderResultToStringArray(
      cached,
      n.order_type,
      cor
    );
    showCheckOrderResult.value = true;
    return false;
  } catch (error) {
    const msg = fmt_err(error, {
      obj: t("main.order"),
    });
    message.error(msg ?? error_to_string(error));
    return false;
  }
}

async function removeCallback(row: Order) {
  try {
    if (row.from_guest_order_id && row.from_guest_order_id > 0) {
      message.warning(
        t("message.fromGuestOrder", { guestId: row.from_guest_order_id })
      );
      return false;
    }
    await remove_order(row.id!);
    message.success(t("message.removeSuccess", { obj: row.id }));
    return true;
  } catch (error: any) {
    const msg = fmt_err(error, {
      obj: t("main.order"),
    });
    message.error(msg ?? error_to_string(error));
    return false;
  }
}

async function clearRows() {
  dialog.warning({
    positiveText: t("action.yes"),
    negativeText: t("action.no"),
    title: t("common.confirmTitle"),
    content: t("message.clearAll"),
    async onPositiveClick() {
      const r = await clear_orders(query.value);
      message.info(
        t("message.clearResult", { success: r.success, failed: r.failed })
      );
    },
  });
}

function addClicked(template: Order) {
  modalRef.value?.showModal(template, ModalType.Add);
}

async function addBaseCallback(base: Order, remove: boolean) {
  to_add_template = base;
  to_add_template.items = await cached.getOrderItems(base.id!);
  if (!remove || (remove && (await removeCallback(base)))) {
    addClicked(to_add_template);
    return true;
  }
  return false;
}

async function recalcOrderClicked() {
  dialog.warning({
    title: t("common.confirmTitle"),
    content: t("message.askRecalcOrders"),
    positiveText: t("action.yes"),
    negativeText: t("action.no"),
    async onPositiveClick() {
      try {
        loadingBar.start();
        await recalc_orders({});
        loadingBar.finish();
        message.success(t("message.recalcOrdersSuccess"));
      } catch (error) {
        const msg = fmt_err(error, {
          obj: t("main.order"),
        });
        message.error(msg ?? error_to_string(error));
      }
    },
  });
}

watch(
  () => myself.config,
  () => (to_add_template = initTemplate())
);

myself.subscribe(async (flag) => {
  if (
    flag.isFlag(WebSocketFlag.AddOrder) ||
    flag.isFlag(WebSocketFlag.RemoveOrder) ||
    flag.isFlag(WebSocketFlag.AddOrderPayment) ||
    flag.isFlag(WebSocketFlag.RemoveOrderPayment) ||
    flag.isFlag(WebSocketFlag.ClearOrders)
  ) {
    await refreshRows();
  } else if (flag.isFlag(WebSocketFlag.UpdateOrderCategory)) {
    const value = await cached.getGuestOrder(flag.id!);
    await refreshRow(flag.id!, value);
  }
});
</script>

<template>
  <div>
    <Result v-model:show="showCheckOrderResult" ref="resultRef" :title="t('common.result')">
      <h1>{{ t("message.itemsNotAvailable") }}</h1>
      <span v-for="(arr, i) in lastResult">
        <br v-if="i != 0" />
        <span v-for="text in arr">
          {{ text }}
          <br />
        </span>
      </span>
    </Result>
    <AddOrUpdateModal ref="modalRef" :form-rows="form" :confirm-callback="confirmClicked">
      <template #default="props">
        <NButton v-if="props.modalType === ModalType.Add" class="m-1" @click="async () => {
          if (await checkAndShowOrder(props.value))
            message.success(t('message.orderPass'));
        }
          ">
          {{ t("action.checkOrder") }}
        </NButton>
      </template>
    </AddOrUpdateModal>

    <NSpace align="center" class="m-3">
      <NButtonGroup>
        <NButton @click="addClicked(to_add_template)">{{
          t("action.add")
          }}</NButton>
        <NButton @click="refreshRows(1)">{{ t("action.filter") }}</NButton>
        <NButton @click="recalcOrderClicked">{{
          t("action.recalcOrders")
          }}</NButton>
        <NButton @click="clearRows">{{ t("action.clear") }}</NButton>
      </NButtonGroup>

      <n-input-number v-model:value="query.id" :min="1" clearable :placeholder="t('common.id')" />
      <SmartSelect :row="{ type: FormRowType.Warehouse, key: 'warehouse_id' }" v-model:value="query.warehouse_ids"
        multiple>
        <SmartCheckbox v-model:value-set="query.reverse" value-key="warehouse_ids">
          {{ t("common.equalToValue") }}</SmartCheckbox>
      </SmartSelect>
      <SmartSelect :row="{
        type: FormRowType.Person,
        key: 'person_related_id',
      }" v-model:value="query.person_related_id">
        <SmartCheckbox v-model:value-set="query.reverse" value-key="person_related_id">
          {{ t("common.equalToValue") }}</SmartCheckbox>
      </SmartSelect>
      <SmartSelect :row="{
        type: FormRowType.Person,
        key: 'person_in_charge_id',
      }" v-model:value="query.person_in_charge_id">
        <SmartCheckbox v-model:value-set="query.reverse" value-key="person_in_charge_id">
          {{ t("common.equalToValue") }}</SmartCheckbox>
      </SmartSelect>
      <SmartSelect :row="{
        type: FormRowType.User,
        key: 'created_by_user_id',
      }" v-model:value="query.created_by_user_id">
        <SmartCheckbox v-model:value-set="query.reverse" value-key="created_by_user_id">
          {{ t("common.equalToValue") }}</SmartCheckbox>
      </SmartSelect>
      <SmartSelect :row="{
        type: FormRowType.User,
        key: 'updated_by_user_id',
      }" v-model:value="query.updated_by_user_id">
        <SmartCheckbox v-model:value-set="query.reverse" value-key="updated_by_user_id">
          {{ t("common.equalToValue") }}</SmartCheckbox>
      </SmartSelect>
      <SmartSelect :row="{ type: FormRowType.OrderType, key: 'order_type' }" v-model:value="query.order_type">
        <SmartCheckbox v-model:value-set="query.reverse" value-key="order_type">
          {{ t("common.equalToValue") }}</SmartCheckbox>
      </SmartSelect>
      <NCheckbox v-model:checked="query.is_record">
        {{ t("common.isRecord") }}
      </NCheckbox>
      <NCheckbox v-model:checked="query.non_payment">
        {{ t("common.nonPayment") }}
      </NCheckbox>
      <SmartSelect :row="{ type: FormRowType.OrderCategory, key: 'order_category_id' }"
        v-model:value="query.order_category_id">
        <SmartCheckbox v-model:value-set="query.reverse" value-key="order_category_id">
          {{ t("common.equalToValue") }}</SmartCheckbox>
      </SmartSelect>
      <SmartSelect :row="{
        type: FormRowType.OrderPaymentStatus,
        key: 'order_payment_status',
      }" v-model:value="query.order_payment_status" multiple>
        <SmartCheckbox v-model:value-set="query.reverse" value-key="order_payment_status">
          {{ t("common.equalToValue") }}</SmartCheckbox>
      </SmartSelect>
      <SmartSelect :row="{ type: FormRowType.OrderCurrency, key: 'currency' }" v-model:value="query.currency">
        <SmartCheckbox v-model:value-set="query.reverse" value-key="currency">
          {{ t("common.equalToValue") }}</SmartCheckbox>
      </SmartSelect>
      <SmartSelect :row="{ type: FormRowType.SKUCategory, key: 'sku_category_id' }"
        v-model:value="query.item_categories" multiple>
        <SmartCheckbox v-model:value-set="query.reverse" value-key="item_categories">
          {{ t("common.equalToValue") }}</SmartCheckbox>
      </SmartSelect>
      <SmartSelect :row="{ type: FormRowType.SKU, key: 'sku_id' }" v-model:value="query.items" multiple>
        <SmartCheckbox v-model:value-set="query.reverse" value-key="items">
          {{ t("common.equalToValue") }}</SmartCheckbox>
      </SmartSelect>
      <MyDatePicker v-model:date_start="query.date_start" v-model:date_end="query.date_end" />
    </NSpace>
    <SmartTable ref="tableRef" :form-rows="form" :detail-callback="async (row) => {
      modalRef?.showModal(row, ModalType.Update);
    }
      " :limit="query.limit" :identity-key="async (row: Order) => t('message.personOrder', {
        person: (await cached.getPerson(row.person_related_id)).name,
        order_id: row.id,
        order_type: getOrderTypeText(row.order_type),
      })" :query-callback="queryCallback" :add-base-callback="addBaseCallback" :remove-callback="removeCallback">
    </SmartTable>
  </div>
</template>

<style scoped></style>
