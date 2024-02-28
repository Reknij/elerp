<script setup lang="ts">
import { useMessage, NInput, NSpace, NButton, NButtonGroup, NInputNumber, useDialog } from "naive-ui";
import { ref } from "vue";
import {
  add_person,
  remove_person,
  update_person,
  get_persons,
clear_persons,
} from "../../api/erp";
import { Person, GetPersonsQuery } from "../../api/erp/model";
import AddOrUpdateModal from "../SmartForm/modal/AddOrUpdate/Modal.vue";
import { FormRow, FormRowType, ModalType } from "../SmartForm/interfaces";
import SmartTable from "../SmartForm/SmartTable.vue";
import SmartSelect from "../SmartForm/SmartSelect.vue";
import { useMySelfUser } from "../../stores/me";
import { WebSocketFlag } from "../../api/ws/models";
import { useCached } from "../../stores/cached";
import { ComponentInstance } from "../../util";
import {
  error_to_string,
  fmt_err,
} from "../../AppError";
import { useI18n } from "vue-i18n";

const { t } = useI18n();
const cached = useCached();
const myself = useMySelfUser();
const message = useMessage();
const dialog = useDialog();
const modalRef = ref<ComponentInstance<typeof AddOrUpdateModal<Person>>>();
const tableRef = ref<ComponentInstance<typeof SmartTable<Person>>>();
const refreshRows = (page = 1) => tableRef.value?.$.exposed?.refreshRows(page);
const refreshRow = (id: number, value: any) =>
  tableRef.value?.$.exposed?.refreshRow(id, value);

const to_add_template: Person = {
  name: "",
  description: "",
  person_in_charge_id: 0,
  address: "",
  area_id: 0,
  contact: "",
  email: "",
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
    key: "person_in_charge_id",
    type: FormRowType.Person,
  },
  {
    key: "area_id",
    type: FormRowType.Area,
  },
  {
    key: "address",
    type: FormRowType.TextArea,
  },
  {
    key: "contact",
    type: FormRowType.Text,
  },
  {
    key: "email",
    type: FormRowType.Text,
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

const query = ref<GetPersonsQuery>({
  index: 0,
  limit: 30,
  sorters: [],
});
async function queryCallback(p: number, sorters: string[]) {
  query.value.index = p - 1;
  query.value.sorters = sorters;
  return await get_persons(query.value);
}

async function confirmClicked(n: Person, mt: ModalType) {
  let r = undefined;
  try {
    if (mt == ModalType.Update) {
      r = await update_person(n.id!, n);
      message.success(t("message.updateSuccess", { obj: n.name }));
    } else if (mt == ModalType.Add) {
      r = await add_person(n);
      message.success(t("message.addSuccess", { obj: n.name }));
    }
    await refreshRows();
  } catch (error: any) {
    const msg = fmt_err(error, {
        obj: t("main.person")
      });
      message.error(msg ?? error_to_string(error));
  }
  return r;
}

async function removeCallback(row: Person) {
  try {
    await remove_person(row.id!);
    message.success(t("message.removeSuccess", { obj: row.name }));
    return true;
  } catch (error: any) {
    const msg = fmt_err(error, {
        obj: t("main.person")
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
      const r = await clear_persons(query.value);
      message.info(t("message.clearResult", {success: r.success, failed: r.failed}));
    }
  })
}

function addClicked() {
  modalRef.value?.showModal(to_add_template, ModalType.Add);
}

myself.subscribe(async (flag) => {
  if (
    flag.isFlag(WebSocketFlag.AddPerson) ||
    flag.isFlag(WebSocketFlag.RemovePerson) ||
    flag.isFlag(WebSocketFlag.ClearPersons)
  ) {
    await refreshRows();
  } else if (flag.isFlag(WebSocketFlag.UpdatePerson)) {
    const value = await cached.getPerson(flag.id!);
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
        <NButton @click="addClicked">Add</NButton>
        <NButton @click="refreshRows(1)">Filter</NButton>
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
        :row="{ type: FormRowType.Person, key: 'person_in_charge_id' }"
        v-model:value="query.person_in_charge_id"
      />
      <n-input
        v-model:value="query.address"
        type="text"
        clearable
        :placeholder="t('common.address')"
      />
      <SmartSelect
        placeholder="Filter area"
        :row="{ type: FormRowType.Area, key: 'area_id' }"
        v-model:value="query.area_id"
      />
    </NSpace>
    <SmartTable
      ref="tableRef"
      :form-rows="form"
      :detail-callback="async (row: Person) => modalRef?.showModal(row, ModalType.Update)"
      :limit="query.limit"
      :identity-key="async (row) => row.name"
      :query-callback="queryCallback"
      :remove-callback="removeCallback"
    ></SmartTable>
  </div>
</template>

<style scoped></style>
