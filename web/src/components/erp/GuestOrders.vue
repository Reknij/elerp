<script setup lang="ts">
import {
  useMessage,
  NSpace,
  NButton,
  NButtonGroup,
  NInputNumber,
  useDialog,
} from "naive-ui";
import { ref } from "vue";
import {
  remove_guest_order,
  add_guest_order,
  get_guest_orders,
  clear_guest_orders,
} from "../../api/erp";
import {
  GuestOrder,
  GetGuestOrdersQuery,
  OrderType,
  OrderCurrency,
  GuestOrderStatus,
} from "../../api/erp/model";
import AddOrUpdateModal from "../SmartForm/modal/AddOrUpdate/Modal.vue";
import { FormRow, FormRowType, ModalType } from "../SmartForm/interfaces";
import SmartTable from "../SmartForm/SmartTable.vue";
import SmartSelect from "../SmartForm/SmartSelect.vue";
import { useMySelfUser } from "../../stores/me";
import { WebSocketFlag } from "../../api/ws/models";
import { useCached } from "../../stores/cached";
import { ComponentInstance, getOrderTypeText } from "../../util";
import { error_to_string, fmt_err } from "../../AppError";
import { useI18n } from "vue-i18n";
import MyDatePicker from "../MyDatePicker.vue";
import SmartCheckbox from "../SmartForm/SmartCheckbox.vue";

const { t } = useI18n();
const cached = useCached();
const myself = useMySelfUser();
const message = useMessage();
const dialog = useDialog();
const modalRef = ref<ComponentInstance<typeof AddOrUpdateModal<GuestOrder>>>();
const tableRef = ref<ComponentInstance<typeof SmartTable<GuestOrder>>>();
const refreshRows = (page = 1) => tableRef.value?.$.exposed?.refreshRows(page);
const refreshRow = (id: number, value: any) =>
  tableRef.value?.$.exposed?.refreshRow(id, value);

let to_add_template: GuestOrder = {
  id: 0,
  date: 0,
  confirmed_date: 0,
  sub_token: "",
  order_type: OrderType.StockOut,
  warehouse_id: 0,
  description: "",
  person_related_id: 0,
  currency: OrderCurrency.MYR,
  items: [],
  guest_order_status: GuestOrderStatus.Pending,
  order_id: 0,
  order_category_id: 10001,
};
const form: FormRow[] = [
  {
    key: "id",
    type: FormRowType.ID,
    disabled: true,
  },
  {
    key: "created_by_user_id",
    type: FormRowType.User,
    disabled: true,
    onlyModal: true,
  },
  {
    key: "date",
    type: FormRowType.Date,
    disabled: true,
  },
  {
    key: "guest_order_status",
    type: FormRowType.GuestOrderStatus,
    initSelf: true,
    noUpdate: true,
  },
  {
    key: "order_type",
    type: FormRowType.OrderType,
    noUpdate: true,
  },
  {
    key: "order_category_id",
    type: FormRowType.OrderCategory,
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
      orderIdKey: 'order_id'
    },
    visibleIf(row) {
      return row.guest_order_status === GuestOrderStatus.Confirmed;
    },
    noUpdate: true,
    onlyModal: true,
  },
];

const query = ref<GetGuestOrdersQuery>({
  index: 0,
  limit: 30,
  sorters: [],
  reverse: new Set(),
});
async function queryCallback(p: number, sorters: string[]) {
  query.value.index = p - 1;
  query.value.sorters = sorters;
  return await get_guest_orders(query.value);
}

async function confirmClicked(n: GuestOrder, mt: ModalType) {
  try {
    if (mt == ModalType.Add) {
      let order = await add_guest_order(n);
      message.success(t("message.addSuccess", { obj: order.id }));
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

async function removeCallback(row: GuestOrder) {
  try {
    await remove_guest_order(row.id!);
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
      const r = await clear_guest_orders(query.value);
      message.info(
        t("message.clearResult", { success: r.success, failed: r.failed })
      );
    },
  });
}

function addClicked(template: GuestOrder) {
  modalRef.value?.showModal(template, ModalType.Add);
}

async function addBaseCallback(base: GuestOrder, remove: boolean) {
  to_add_template = base;
  if (!remove || (remove && (await removeCallback(base)))) {
    addClicked(to_add_template);
    return true;
  }
  return false;
}

const copyLink = (guest: GuestOrder) => {
  navigator.clipboard.writeText(
    `${location.origin}/guest/${guest.id}?sub_token=${guest.sub_token}`
  );
  message.success(t("message.copySuccess"));
};
const openLink = (guest: GuestOrder) => {
  window.open(
    `${location.origin}/guest/${guest.id}?sub_token=${guest.sub_token}`
  );
};

myself.subscribe(async (flag) => {
  if (
    flag.isFlag(WebSocketFlag.AddGuestOrder) ||
    flag.isFlag(WebSocketFlag.RemoveGuestOrder) ||
    flag.isFlag(WebSocketFlag.ClearOrders)
  ) {
    await refreshRows();
  } else if (flag.isFlag(WebSocketFlag.ConfirmGuestOrder)) {
    const value = await cached.getGuestOrder(flag.id!);
    await refreshRow(flag.id!, value);
  }
});
</script>

<template>
  <div>
    <AddOrUpdateModal
      ref="modalRef"
      :ignore-check="['items']"
      :form-rows="form"
      :confirm-callback="confirmClicked"
    >
      <template #default="props">
        <NButton
          v-if="props.modalType === ModalType.Read"
          @click="copyLink(props.value)"
          >{{ t("action.copyLink") }}</NButton
        >
        <NButton
          v-if="props.modalType === ModalType.Read"
          @click="openLink(props.value)"
          >{{ t("action.openLink") }}</NButton
        >
      </template>
    </AddOrUpdateModal>

    <NSpace align="center" class="m-3">
      <NButtonGroup>
        <NButton @click="addClicked(to_add_template)">{{
          t("action.add")
        }}</NButton>
        <NButton @click="refreshRows(1)">{{ t("action.filter") }}</NButton>
        <NButton @click="clearRows">{{ t("action.clear") }}</NButton>
      </NButtonGroup>

      <n-input-number
        v-model:value="query.id"
        :min="1"
        clearable
        :placeholder="t('common.id')"
      />
      <SmartSelect
        :row="{ type: FormRowType.Warehouse, key: 'warehouse_ids' }"
        v-model:value="query.warehouse_ids"
        multiple
      >
        <SmartCheckbox
          v-model:value-set="query.reverse"
          value-key="warehouse_ids"
        >
          {{ t("common.equalToValue") }}</SmartCheckbox
        >
      </SmartSelect>
      <SmartSelect
        :row="{
          type: FormRowType.Person,
          key: 'person_related_id',
        }"
        v-model:value="query.person_related_id"
      >
        <SmartCheckbox
          v-model:value-set="query.reverse"
          value-key="person_related_id"
        >
          {{ t("common.equalToValue") }}</SmartCheckbox
        ></SmartSelect
      >
      <SmartSelect
        :row="{
          type: FormRowType.Person,
          key: 'person_in_charge_id',
        }"
        v-model:value="query.person_in_charge_id"
      >
        <SmartCheckbox
          v-model:value-set="query.reverse"
          value-key="person_in_charge_id"
        >
          {{ t("common.equalToValue") }}</SmartCheckbox
        ></SmartSelect
      >
      <SmartSelect
        :row="{
          type: FormRowType.User,
          key: 'created_by_user_id',
        }"
        v-model:value="query.created_by_user_id"
      >
        <SmartCheckbox
          v-model:value-set="query.reverse"
          value-key="created_by_user_id"
        >
          {{ t("common.equalToValue") }}</SmartCheckbox
        ></SmartSelect
      >
      <SmartSelect
        :row="{ type: FormRowType.OrderType, key: 'order_type' }"
        v-model:value="query.order_type"
      >
        <SmartCheckbox v-model:value-set="query.reverse" value-key="order_type">
          {{ t("common.equalToValue") }}</SmartCheckbox
        >
      </SmartSelect>
      <SmartSelect
        :row="{ type: FormRowType.OrderCurrency, key: 'currency' }"
        v-model:value="query.currency"
      >
        <SmartCheckbox v-model:value-set="query.reverse" value-key="currency">
          {{ t("common.equalToValue") }}</SmartCheckbox
        ></SmartSelect
      >
      <MyDatePicker
        v-model:date_start="query.date_start"
        v-model:date_end="query.date_end"
      />
    </NSpace>
    <SmartTable
      ref="tableRef"
      :form-rows="form"
      :detail-callback="
        async (row) => {
          modalRef?.showModal(row, ModalType.Read);
        }
      "
      :limit="query.limit"
      :identity-key="async (row: GuestOrder) => t('message.personOrder', {
        person: (await cached.getPerson(row.person_related_id)).name,
        order_id: row.id,
        order_type: getOrderTypeText(row.order_type),
      })"
      :query-callback="queryCallback"
      :add-base-callback="addBaseCallback"
      :remove-callback="removeCallback"
    ></SmartTable>
  </div>
</template>

<style scoped></style>
