<script setup lang="ts">
import { ref, toRaw } from "vue";
import { get_inventory, download_inventory_excel } from "../../api/erp";
import { GetInventoryQuery, InventoryProduct } from "../../api/erp/model";
import { FormRow, FormRowType, ModalType } from "../SmartForm/interfaces";
import AddOrUpdateModal from "../SmartForm/modal/AddOrUpdate/Modal.vue";
import SmartTable from "../SmartForm/SmartTable.vue";
import SmartSelect from "../SmartForm/SmartSelect.vue";
import {
  NSpace,
  NInputNumber,
  NButton,
  NButtonGroup,
  useDialog,
  useMessage,
} from "naive-ui";
import { useMySelfUser } from "../../stores/me";
import { WebSocketFlag } from "../../api/ws/models";
import { ComponentInstance } from "../../util";
import { useI18n } from "vue-i18n";

const { t } = useI18n();
const dialog = useDialog();
const message = useMessage();
const myself = useMySelfUser();
const modalRef =
  ref<ComponentInstance<typeof AddOrUpdateModal<InventoryProduct>>>();
const tableRef = ref<ComponentInstance<typeof SmartTable<InventoryProduct>>>();
const refreshRows = (page = 1) => tableRef.value?.$.exposed?.refreshRows(page);

const form: FormRow[] = [
  {
    key: "warehouse_id",
    type: FormRowType.Warehouse,
  },
  {
    key: "sku_id",
    type: FormRowType.SKU,
  },
  {
    key: "sku_category_id",
    type: FormRowType.SKUCategory,
  },
  {
    key: "quantity",
    type: FormRowType.Number,
  },
];
const query = ref<GetInventoryQuery>({
  index: 0,
  limit: 30,
  sorters: [],
});
async function queryCallback(p: number, sorters: string[]) {
  query.value.index = p - 1;
  query.value.sorters = sorters;
  const r = await get_inventory(query.value);
  return r;
}
async function detailCallback(row: InventoryProduct) {
  modalRef.value?.showModal(row, ModalType.Read);
}

function exportExcel() {
  dialog.info({
    title: t("action.export"),
    content: t("message.exportToExcel", { obj: t("main.inventory") }),
    positiveText: t("action.yes"),
    negativeText: t("action.no"),
    async onPositiveClick() {
      let q = structuredClone(toRaw(query.value)) as any;
      delete q.index;
      delete q.limit;
      await Promise.all([refreshRows(1), download_inventory_excel(q)]);
      message.success("Export inventory list successfully!");
    },
  });
}

myself.subscribe(async (flag) => {
  if (
    flag.isFlag(WebSocketFlag.AddOrder) ||
    flag.isFlag(WebSocketFlag.RemoveOrder) ||
    flag.isFlag(WebSocketFlag.RecalcOrders) ||
    flag.isFlag(WebSocketFlag.ClearOrders) ||
    flag.isFlag(WebSocketFlag.ClearAreas)
  ) {
    await refreshRows();
  }
});
</script>

<template>
  <div>
    <add-or-update-modal ref="modalRef" :form-rows="form"></add-or-update-modal>
    <NSpace align="center" class="m-3">
      <NButtonGroup>
        <NButton @click="refreshRows(1)">{{ t("action.filter") }}</NButton>
        <NButton @click="exportExcel">{{ t("action.export") }}</NButton>
      </NButtonGroup>
      <SmartSelect :row="{ type: FormRowType.Warehouse, key: 'warehouse_id' }" v-model:value="query.warehouse_ids"
        multiple></SmartSelect>
      <SmartSelect :row="{ type: FormRowType.SKU, key: 'sku_id' }" v-model:value="query.sku_ids" multiple></SmartSelect>
      <SmartSelect :row="{ type: FormRowType.SKUCategory, key: 'sku_category_id' }"
        v-model:value="query.sku_category_ids" multiple></SmartSelect>
      <n-input-number v-model:value="query.quantity_start" :placeholder="t('common.minQuantity')" clearable />
      <n-input-number v-model:value="query.quantity_end" :placeholder="t('common.maxQuantity')" clearable />
    </NSpace>

    <smart-table ref="tableRef" :form-rows="form" :limit="query.limit" :query-callback="queryCallback"
      :detail-callback="detailCallback"></smart-table>
  </div>
</template>

<style scoped></style>
../../api/erp