<script setup lang="ts">
import { NCheckbox, NSpace } from "naive-ui";
import { UserPermission } from "../../../api/user_system/models";
import { useI18n } from "vue-i18n";

const { t } = useI18n();

defineProps<{
  value: UserPermission;
}>();
const emit = defineEmits<{
  (e: "update:value", v: UserPermission): void;
}>();

const permissions = [
  {
    label: t("userPermission.manageArea"),
    value: UserPermission.MANAGE_AREA,
  },
  {
    label: t("userPermission.managePerson"),
    value: UserPermission.MANAGE_PERSON,
  },
  {
    label: t("userPermission.manageWarehouse"),
    value: UserPermission.MANAGE_WAREHOUSE,
  },
  {
    label: t("userPermission.manageSKU"),
    value: UserPermission.MANAGE_SKU,
  },
  {
    label: t("userPermission.manageSKUCategory"),
    value: UserPermission.MANAGE_SKU_CATEGORY,
  },
  {
    label: t("userPermission.manageOrderStatus"),
    value: UserPermission.MANAGE_ORDER_STATUS,
  },
  {
    label: t("userPermission.addOrder"),
    value: UserPermission.ADD_ORDER,
  },
  {
    label: t("userPermission.updateRemoveOrder"),
    value: UserPermission.UPDATE_REMOVE_ORDER,
  },
  {
    label: t("userPermission.addOrderPayment"),
    value: UserPermission.ADD_ORDER_PAYMENT,
  },
  {
    label: t("userPermission.updateRemoveOrderPayment"),
    value: UserPermission.UPDATE_REMOVE_ORDER_PAYMENT,
  },
];
</script>

<template>
  <n-space item-style="display: flex;">
    <n-checkbox
      :default-checked="(value & p.value) === p.value"
      v-for="p in permissions"
      :on-update-checked="
        (checked) =>
          emit('update:value', checked ? value | p.value : value & ~p.value)
      "
      :label="p.label"
    />
  </n-space>
</template>
