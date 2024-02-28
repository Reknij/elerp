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
  remove_order_payment,
  add_order_payment,
  get_order_payments,
clear_order_payments,
} from "../../api/erp";
import { OrderPayment, GetOrderPaymentsQuery, OrderPaymentStatus } from "../../api/erp/model";
import AddOrUpdateModal from "../SmartForm/modal/AddOrUpdate/Modal.vue";
import { FormRow, FormRowType, ModalType } from "../SmartForm/interfaces";
import SmartTable from "../SmartForm/SmartTable.vue";
import Result from "../SmartForm/modal/Result.vue";
import SmartSelect from "../SmartForm/SmartSelect.vue";
import { useMySelfUser } from "../../stores/me";
import { WebSocketFlag } from "../../api/ws/models";
import { useCached } from "../../stores/cached";
import { ComponentInstance } from "../../util";
import { error_to_string, fmt_err } from "../../AppError";
import { useI18n } from "vue-i18n";
import MyDatePicker from "../MyDatePicker.vue";

const { t } = useI18n();
const cached = useCached();
const myself = useMySelfUser();
const message = useMessage();
const dialog = useDialog();
const modalRef =
  ref<ComponentInstance<typeof AddOrUpdateModal<OrderPayment>>>();
const tableRef = ref<ComponentInstance<typeof SmartTable<OrderPayment>>>();
const refreshRows = (page = 1) => tableRef.value?.$.exposed?.refreshRows(page);

const lastResult = ref<string[][]>([]);
const showCheckOrderResult = ref(false);
let to_add_template: OrderPayment = {
  id: 0,
  creation_date: 0,
  actual_date: 0,
  order_id: 0,
  person_in_charge_id: 0,
  total_amount: 0,
  remark: "",
};
const form: FormRow[] = [
  {
    key: "id",
    type: FormRowType.ID,
    disabled: true,
  },
  {
    key: "creation_date",
    type: FormRowType.Date,
    initSelf: true,
    sorter: 'descend',
  },
  {
    key: "actual_date",
    type: FormRowType.DatePicker,
  },
  {
    key: "order_id",
    type: FormRowType.Order,
    noUpdate: true,
    query: {
      order_payment_status: [OrderPaymentStatus.Unsettled, OrderPaymentStatus.PartialSettled]
    }
  },
  {
    key: "person_in_charge_id",
    type: FormRowType.Person,
  },
  {
    key: "total_amount",
    type: FormRowType.Number,
  },
  {
    key: "remark",
    type: FormRowType.TextArea,
  },
];

const query = ref<GetOrderPaymentsQuery>({
  index: 0,
  limit: 30,
  sorters: [],
});
async function queryCallback(p: number, sorters: string[]) {
  query.value.index = p - 1;
  query.value.sorters = sorters;
  return await get_order_payments(query.value);
}

async function confirmClicked(n: OrderPayment, mt: ModalType) {
  try {
    if (mt == ModalType.Add) {
      let op = await add_order_payment(n);
      message.success(t("message.addSuccess", { obj: op.id }));
    }
    await refreshRows();
  } catch (error: any) {
    const msg = fmt_err(error, {
      obj: t("main.payment"),
    });
    message.error(msg ?? error_to_string(error));
  }

  return;
}

async function removeCallback(row: OrderPayment) {
  try {
    await remove_order_payment(row.id!);
    message.success(t("message.removeSuccess", { obj: row.id }));
    return true;
  } catch (error: any) {
    const msg = fmt_err(error, {
      obj: t("main.payment"),
    });
    message.error(msg ?? error_to_string(error));
    return false;
  }
}

async function clearRows() {
  dialog.warning({
    positiveText: t('action.yes'),
    negativeText: t('action.no'),
    title: t("common.confirmTitle"),
    content: t("message.clearAll"),
    async onPositiveClick() {
      const r = await clear_order_payments(query.value);
      message.info(t("message.clearResult", {success: r.success, failed: r.failed}));
    }
  })
}

function addClicked(template: OrderPayment) {
  modalRef.value?.showModal(template, ModalType.Add);
}

async function addBaseCallback(base: OrderPayment, remove: boolean) {
  to_add_template = base;
  if (!remove || (remove && (await removeCallback(base)))) {
    addClicked(to_add_template);
    return true;
  }
  return false;
}

myself.subscribe(async (flag) => {
  if (
    flag.isFlag(WebSocketFlag.AddOrderPayment) ||
    flag.isFlag(WebSocketFlag.RemoveOrderPayment)
  ) {
    await refreshRows();
  }
});
</script>

<template>
  <div>
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
    <AddOrUpdateModal
      ref="modalRef"
      :form-rows="form"
      :confirm-callback="confirmClicked"
    >
    </AddOrUpdateModal>

    <NSpace align="center" class="m-3">
      <NButtonGroup>
        <NButton @click="addClicked(to_add_template)">{{
          t("action.add")
        }}</NButton>
        <NButton @click="refreshRows(1)">{{ t("action.filter") }}</NButton>
        <NButton @click="clearRows">{{
          t("action.clear")
        }}</NButton>
      </NButtonGroup>
      <n-input-number
        v-model:value="query.id"
        :min="1"
        clearable
        :placeholder="t('common.id')"
      />
      <SmartSelect
        :row="{ type: FormRowType.Order, key: 'order_id' }"
        v-model:value="query.order_id"
      ></SmartSelect>
      <SmartSelect
        :row="{
          type: FormRowType.Person,
          key: 'person_in_charge_id',
        }"
        v-model:value="query.person_in_charge_id"
      ></SmartSelect>
      <MyDatePicker
        v-model:date_start="query.creation_date_start"
        v-model:date_end="query.creation_date_end"
      />
      <MyDatePicker
        v-model:date_start="query.actual_date_start"
        v-model:date_end="query.actual_date_end"
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
      :identity-key="async (row: OrderPayment) => t('message.personOrderPayment', {
        person: (await cached.getPerson(row.person_in_charge_id)).name,
        order_payment_id: row.id,
      })"
      :query-callback="queryCallback"
      :add-base-callback="addBaseCallback"
      :remove-callback="removeCallback"
    ></SmartTable>
  </div>
</template>

<style scoped></style>
