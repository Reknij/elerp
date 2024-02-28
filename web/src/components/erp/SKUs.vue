<script setup lang="ts">
import { NSpace, useMessage, NInput, NButton, NButtonGroup, NInputNumber, useDialog } from "naive-ui";
import { ref } from "vue";
import { remove_sku, update_sku, get_skus, add_sku, clear_skus } from "../../api/erp";
import { SKU, GetSKUsQuery } from "../../api/erp/model";
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

const { t } = useI18n();
const cached = useCached();
const myself = useMySelfUser();
const message = useMessage();
const dialog = useDialog();
const modalRef = ref<ComponentInstance<typeof AddOrUpdateModal<SKU>>>();
const tableRef = ref<ComponentInstance<typeof SmartTable<SKU>>>();
const refreshRows = (page: number = 1) =>
  tableRef.value?.$.exposed?.refreshRows(page);
const refreshRow = (id: number, value: any) =>
  tableRef.value?.$.exposed?.refreshRow(id, value);

const to_add_template: SKU = {
  id: 0,
  sku_category_id: 0,
  name: "",
  description: "",
};
const form: FormRow[] = [
  {
    key: "id",
    type: FormRowType.ID,
    disabled: true,
  },
  {
    key: "sku_category_id",
    type: FormRowType.SKUCategory,
  },
  {
    key: "name",
    type: FormRowType.TextAreaColor,
  },
  {
    key: "description",
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

const query = ref<GetSKUsQuery>({
  index: 0,
  limit: 30,
  sorters: [],
});
async function queryCallback(p: number, sorters: string[]) {
  query.value.index = p - 1;
  query.value.sorters = sorters;
  return await get_skus(query.value);
}

async function confirmClicked(n: SKU, mt: ModalType) {
  try {
    if (mt == ModalType.Update) {
      await update_sku(n.id!, n);
      message.success(t("message.updateSuccess", { obj: n.name }));
    } else if (mt == ModalType.Add) {
      const names = n.name.split("\n");
      const promises = [];
      for (let i = 0; i < names.length; i++) {
        const name = names[i];
        if (name.length > 0) {
          n.name = name;
          promises.push(add_sku(n));
        }
      }
      await Promise.all(promises);
      if (names.length > 1) {
        message.success(
          t("message.addMultipleSuccess", { obj: t("main.SKU") })
        );
      } else {
        message.success(t("message.addSuccess", { obj: n.name }));
      }
    }
    await refreshRows();
  } catch (error: any) {
    const msg = fmt_err(error, {
        obj: t("main.SKU")
      });
      message.error(msg ?? error_to_string(error));
  }
}

async function removeCallback(row: SKU) {
  try {
    await remove_sku(row.id!);
    message.success(t("message.removeSuccess", { obj: row.name }));
    return true;
  } catch (error: any) {
    const msg = fmt_err(error, {
        obj: t("main.SKU")
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
      const r = await clear_skus(query.value);
      message.info(t("message.clearResult", {success: r.success, failed: r.failed}));
    }
  })
}

function addClicked() {
  modalRef.value?.showModal(to_add_template, ModalType.Add);
}

myself.subscribe(async (flag) => {
  if (
    flag.isFlag(WebSocketFlag.AddSKU) ||
    flag.isFlag(WebSocketFlag.RemoveSKU) ||
    flag.isFlag(WebSocketFlag.ClearSKUs)
  ) {
    await refreshRows();
  } else if (flag.isFlag(WebSocketFlag.UpdateSKU)) {
    const value = await cached.getSKU(flag.id!);
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
      <n-input
        v-model:value="query.name"
        type="text"
        clearable
        :placeholder="t('common.name')"
      />
      <SmartSelect
        :row="{
          type: FormRowType.SKUCategory,
          key: 'sku_category_id',
        }"
        :limit="30"
        v-model:value="query.sku_category_id"
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