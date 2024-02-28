<script setup lang="ts">
import { NSpace, useMessage, NInput, NButton, NButtonGroup, NInputNumber, useDialog } from "naive-ui";
import { ref } from "vue";
import { add_area, clear_areas, get_areas, remove_area, update_area } from "../../api/erp";
import { Area, GetAreasQuery } from "../../api/erp/model";
import AddOrUpdateModal from "../SmartForm/modal/AddOrUpdate/Modal.vue";
import { FormRow, FormRowType, ModalType } from "../SmartForm/interfaces";
import SmartTable from "../SmartForm/SmartTable.vue";
import { error_to_string, fmt_err } from "../../AppError";
import { useMySelfUser } from "../../stores/me";
import { WebSocketFlag } from "../../api/ws/models";
import { useCached } from "../../stores/cached";
import { ComponentInstance } from "../../util";
import { useI18n } from "vue-i18n";

const { t } = useI18n();
const myself = useMySelfUser();
const cached = useCached();
const modalRef = ref<ComponentInstance<typeof AddOrUpdateModal<Area>>>();
const tableRef = ref<ComponentInstance<typeof SmartTable<Area>>>();
const refreshRows = (page = 1) => tableRef.value?.$.exposed?.refreshRows(page);
const refreshRow = (id: number, value: any) =>
  tableRef.value?.$.exposed?.refreshRow(id, value);

const to_add_template: Area = {
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
    key: "name",
    type: FormRowType.TextColor,
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

const query = ref<GetAreasQuery>({
  index: 0,
  limit: 30,
  sorters: [],
});
const message = useMessage();
const dialog = useDialog();
async function confirmClicked(n: Area, mt: ModalType) {
  try {
    if (mt == ModalType.Update) {
      await update_area(n.id!, n);
      message.success(t("message.updateSuccess", { obj: n.name }));
      await refreshRows();
    } else if (mt == ModalType.Add) {
      await add_area(n);
      message.success(t("message.addSuccess", { obj: n.name }));
      await refreshRows();
    }
  } catch (error: any) {
    const msg = fmt_err(error, {
        obj: t("main.area")
      });
      message.error(msg ?? error_to_string(error));
  }
}

function addClicked() {
  modalRef.value?.showModal(to_add_template, ModalType.Add);
}
async function queryCallback(p: number, sorters: string[]) {
  query.value.index = p - 1;
  query.value.sorters = sorters;
  const r = await get_areas(query.value);
  return r;
}
async function removeCallback(row: Area) {
  try {
    await remove_area(row.id!);
    message.success(t("message.removeSuccess", { obj: row.name }));
    return true;
  } catch (error: any) {
    const msg = fmt_err(error, {
        obj: t("main.area")
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
      const r = await clear_areas(query.value);
      message.info(t("message.clearResult", {success: r.success, failed: r.failed}));
    }
  })
}

myself.subscribe(async (flag) => {
  if (
    flag.isFlag(WebSocketFlag.AddArea) ||
    flag.isFlag(WebSocketFlag.RemoveArea) ||
    flag.isFlag(WebSocketFlag.ClearAreas)
  ) {
    await refreshRows();
  } else if (flag.isFlag(WebSocketFlag.UpdateArea)) {
    const value = await cached.getArea(flag.id!);
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
    </NSpace>

    <smart-table
      ref="tableRef"
      :form-rows="form"
      :limit="query.limit"
      :identity-key="async (row) => row.name"
      :query-callback="queryCallback"
      :detail-callback="async (row: Area) => modalRef?.showModal(row, ModalType.Update)"
      :remove-callback="removeCallback"
    ></smart-table>
  </div>
</template>

<style scoped></style>
