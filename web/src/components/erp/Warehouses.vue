<script setup lang="ts">
import {
  useMessage,
  NSpace,
  NInput,
  NButton,
  NButtonGroup,
  NInputNumber,
  useDialog,
} from "naive-ui";
import { ref } from "vue";
import {
  remove_warehouse,
  update_warehouse,
  get_warehouses,
  add_warehouse,
  clear_warehouses,
} from "../../api/erp";
import { Warehouse, GetWarehousesQuery } from "../../api/erp/model";
import AddOrUpdateModal from "../SmartForm/modal/AddOrUpdate/Modal.vue";
import { FormRow, FormRowType, ModalType } from "../SmartForm/interfaces";
import SmartTable from "../SmartForm/SmartTable.vue";
import SmartSelect from "../SmartForm/SmartSelect.vue";
import { WebSocketFlag } from "../../api/ws/models";
import { useMySelfUser } from "../../stores/me";
import { useCached } from "../../stores/cached";
import { ComponentInstance } from "../../util";
import { error_to_string, fmt_err } from "../../AppError";
import { useI18n } from "vue-i18n";
import { UserType } from "../../api/user_system/models";

const { t } = useI18n();
const cached = useCached();
const myself = useMySelfUser();
const message = useMessage();
const dialog = useDialog();
const modalRef = ref<ComponentInstance<typeof AddOrUpdateModal<Warehouse>>>();
const tableRef = ref<ComponentInstance<typeof SmartTable<Warehouse>>>();
const refreshRows = (page = 1) => tableRef.value?.$.exposed?.refreshRows(page);
const refreshRow = (id: number, value: any) =>
  tableRef.value?.$.exposed?.refreshRow(id, value);

const to_add_template: Warehouse = {
  id: 0,
  area_id: 0,
  name: "",
  description: "",
  address: "",
  person_in_charge_id: 0,
};
const form: FormRow[] = [
  {
    key: "id",
    type: FormRowType.ID,
    disabled: true,
  },
  {
    key: "area_id",
    type: FormRowType.Area,
  },
  {
    key: "person_in_charge_id",
    type: FormRowType.Person,
  },
  {
    key: "name",
    type: FormRowType.TextColor,
  },
  {
    key: "description",
    type: FormRowType.TextArea,
  },
  {
    key: "address",
    type: FormRowType.TextArea,
  },
  {
    key: "color",
    type: FormRowType.Text,
    onlyModal: true,
  },
  {
    key: "text_color",
    type: FormRowType.Text,
    onlyModal: true,
  },
];
if (myself.authenticated?.user.user_type === UserType.Admin) {
  form.push({
    key: "warehouse_linked_users",
    type: FormRowType.WarehouseLinkedUsers,
    onlyModal: true,
    initSelf: true,
  });
}

const query = ref<GetWarehousesQuery>({
  index: 0,
  limit: 30,
  sorters: [],
});
async function queryCallback(p: number, sorters: string[]) {
  query.value.index = p - 1;
  query.value.sorters = sorters;
  return await get_warehouses(query.value);
}

async function confirmClicked(n: Warehouse, mt: ModalType) {
  let r = undefined;
  try {
    if (mt == ModalType.Update) {
      r = await update_warehouse(n.id!, n);
      message.success(t("message.updateSuccess", { obj: n.name }));
    } else if (mt == ModalType.Add) {
      r = await add_warehouse(n);
      message.success(t("message.addSuccess", { obj: n.name }));
    }
    await refreshRows();
  } catch (error: any) {
    const msg = fmt_err(error, {
      obj: t("main.warehouse"),
    });
    message.error(msg ?? error_to_string(error));
  }
  return r;
}

async function removeCallback(row: Warehouse) {
  try {
    await remove_warehouse(row.id!);
    message.success(t("message.removeSuccess", { obj: row.name }));
    return true;
  } catch (error: any) {
    const msg = fmt_err(error, {
      obj: t("main.warehouse"),
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
      const r = await clear_warehouses(query.value);
      message.info(
        t("message.clearResult", { success: r.success, failed: r.failed })
      );
    },
  });
}

function addClicked() {
  modalRef.value?.showModal(to_add_template, ModalType.Add);
}

myself.subscribe(async (flag) => {
  if (
    flag.isFlag(WebSocketFlag.AddWarehouse) ||
    flag.isFlag(WebSocketFlag.RemoveWarehouse) ||
    flag.isFlag(WebSocketFlag.ClearWarehouses)
  ) {
    await refreshRows();
  } else if (flag.isFlag(WebSocketFlag.UpdateWarehouse)) {
    const value = await cached.getWarehouse(flag.id!);
    await refreshRow(flag.id!, value);
  }
});
</script>

<template>
  <div>
    <AddOrUpdateModal
      ref="modalRef"
      :form-rows="form"
      :confirm-callback="confirmClicked"
    ></AddOrUpdateModal>

    <NSpace align="center" class="m-3">
      <NButtonGroup>
        <NButton @click="addClicked">{{ t("action.add") }}</NButton>
        <NButton @click="refreshRows(1)">{{ t("action.filter") }}</NButton>
        <NButton @click="clearRows">{{ t("action.clear") }}</NButton>
      </NButtonGroup>
      <n-input-number
        v-model:value="query.id"
        :min="1"
        clearable
        :placeholder="t('common.id')"
      />
      <n-input
        v-model:value="query.name"
        type="text"
        clearable
        :placeholder="t('common.name')"
      />
      <SmartSelect
        v-model:value="query.area_id"
        :row="{ type: FormRowType.Area, key: 'area_id' }"
      />
      <SmartSelect
        v-model:value="query.person_in_charge_id"
        :row="{ type: FormRowType.Person, key: 'person_in_charge_id' }"
      />
    </NSpace>
    <SmartTable
      ref="tableRef"
      :form-rows="form"
      :detail-callback="
        async (row) => modalRef?.showModal(row, ModalType.Update)
      "
      :limit="query.limit"
      :identity-key="async (row) => row.name"
      :query-callback="queryCallback"
      :remove-callback="removeCallback"
    ></SmartTable>
  </div>
</template>

<style scoped></style>
../../api/erp
