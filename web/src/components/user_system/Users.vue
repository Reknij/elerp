<script setup lang="ts">
import AddOrUpdateModal from "../SmartForm/modal/AddOrUpdate/Modal.vue";
import SmartTable from "../SmartForm/SmartTable.vue";
import {
  GetUsersQuery,
  UserInfo,
  UserPermission,
  UserType,
} from "../../api/user_system/models";
import {
  get_users,
  add_user,
  update_user,
  remove_user,
} from "../../api/user_system";
import { FormRow, FormRowType, ModalType } from "../SmartForm/interfaces";
import { ref } from "vue";
import { useMessage, NSpace, NButtonGroup, NButton, NInput } from "naive-ui";
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
const tableRef = ref<ComponentInstance<typeof SmartTable<UserInfo>>>();
const modalRef = ref<ComponentInstance<typeof AddOrUpdateModal<UserInfo>>>();
const refreshRows = (page = 1) => tableRef.value?.$.exposed?.refreshRows(page);
const refreshRow = (id: number, value: any) =>
  tableRef.value?.$.exposed?.refreshRow(id, value);

const to_add_template: UserInfo = {
  id: 0,
  alias: "",
  username: "",
  password: "",
  user_type: UserType.General,
  permission: UserPermission.EMPTY,
  is_connected: false,
};
const form: FormRow[] = [
  {
    key: "id",
    type: FormRowType.ID,
    disabled: true,
  },
  {
    key: "is_connected",
    type: FormRowType.DotStatus,
    disabled: true,
  },
  {
    key: "alias",
    type: FormRowType.Text,
  },
  {
    key: "username",
    type: FormRowType.Text,
  },
  {
    key: "password",
    type: FormRowType.Text,
  },
  {
    key: "user_type",
    type: FormRowType.UserType,
    disabled: true,
  },
  {
    key: "permission",
    type: FormRowType.UserPermission,
  },
];

const query = ref<GetUsersQuery>({
  index: 0,
  limit: 30,
  sorters: [],
});
async function queryCallback(p: number, sorters: string[]) {
  query.value.index = p - 1;
  query.value.sorters = sorters;
  return await get_users(query.value);
}

async function confirmClicked(n: UserInfo, mt: ModalType) {
  let r = undefined;
  try {
    if (mt == ModalType.Update) {
      r = await update_user(n.id!, n);
      message.success(t("message.updateSuccess", { obj: n.username }));
    } else if (mt == ModalType.Add) {
      r = await add_user(n);
      message.success(t("message.addSuccess", { obj: n.username }));
    }

    await refreshRows();
  } catch (error: any) {
    const msg = fmt_err(error, {
        obj: t("main.user")
      });
      message.error(msg ?? error_to_string(error));
  }
  return r;
}

async function removeCallback(row: UserInfo) {
  try {
    await remove_user(row.id!);
    message.success(t("message.removeSuccess", { obj: row.username }));
    return true;
  } catch (error: any) {
    const msg = fmt_err(error, {
        obj: t("main.user")
      });
      message.error(msg ?? error_to_string(error));
    return false;
  }
}

function addClicked() {
  modalRef.value?.showModal(to_add_template, ModalType.Add);
}

myself.subscribe(async (flag) => {
  if (
    flag.isFlag(WebSocketFlag.AddUser) ||
    flag.isFlag(WebSocketFlag.RemoveUser)
  ) {
    await refreshRows();
  } else if (flag.isFlag(WebSocketFlag.UpdateUser) || flag.isFlag(WebSocketFlag.UserConnected) || flag.isFlag(WebSocketFlag.UserDisconnected)) {
    const value = await cached.getUser(flag.id!);
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
      </NButtonGroup>
      <n-input
        v-model:value="query.alias"
        type="text"
        clearable
        :placeholder="t('common.alias')"
      />
      <n-input
        v-model:value="query.username"
        type="text"
        clearable
        :placeholder="t('common.username')"
      />
    </NSpace>
    <SmartTable
      ref="tableRef"
      :form-rows="form"
      :identity-key="async (row) => `${row.alias}@${row.username}`"
      :detail-callback="
        async (row) => modalRef?.showModal(row, ModalType.Update)
      "
      :limit="query.limit"
      :query-callback="queryCallback"
      :remove-callback="removeCallback"
    ></SmartTable>
  </div>
</template>
