<script setup lang="ts">
import { ref } from "vue";
import SmartSelect from "../../SmartSelect.vue";
import { FormRowType } from "../../interfaces";
import {
  get_linked_users,
  link_warehouse,
  unlink_warehouse,
} from "../../../../api/erp";
import { ListSlice } from "../../../../api/models";
import { UserInfo } from "../../../../api/user_system/models";
import { useMySelfUser } from "../../../../stores/me";
import { useMessage } from "naive-ui";
import { useI18n } from "vue-i18n";
import { useCached } from "../../../../stores/cached";

const props = defineProps<{
  id: number;
}>();
const myself = useMySelfUser();
const cached = useCached();
const message = useMessage();
const { t } = useI18n();

const users = ref<ListSlice<UserInfo>>(
  await get_linked_users(props.id, {
    index: 0,
    limit: 999999999,
  })
);
const linked = ref<number[]>([]);
for (let i = 0; i < users.value.items.length; i++) {
  const user = users.value.items[i];
  linked.value.push(user.id);
}
let linked_users: Set<number> = new Set(linked.value);
async function updateEvent(v?: number[]) {
  if (!v) {
    v = [];
  }
  for (let i = 0; i < v.length; i++) {
    const user_id = v[i];
    if (!linked_users.delete(user_id)) {
      const user = await cached.getUser(user_id);
      await link_warehouse(props.id, {
        user_id,
      });
      message.success(
        t("message.linkedWarehouseSuccess", {
          user: user.alias,
        })
      );
    }
  }
  const arr = Array.from(linked_users.values());
  for (let j = 0; j < arr.length; j++) {
    const user_id = arr[j];
    const user = await cached.getUser(user_id);
    await unlink_warehouse(props.id, {
      user_id,
    });
    message.success(
      t("message.unlinkedWarehouseSuccess", {
        user: user.alias,
      })
    );
  }
  linked_users = new Set(v);
  if (myself.authenticated?.user.id) {
    linked_users.delete(myself.authenticated.user.id);
  }
}
</script>

<template>
  <SmartSelect
    :row="{
      type: FormRowType.User,
      key: 'user_id',
    }"
    :limit="30"
    v-model:value="linked"
    multiple
    @update:value="updateEvent"
    :disabled-value="myself.authenticated?.user.id"
  />
</template>
