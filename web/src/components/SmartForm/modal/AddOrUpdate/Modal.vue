<script setup lang="ts" generic="T">
import { computed, ref, toRaw, watch } from "vue";
import { FormRow, FormRowType, ModalType } from "../../interfaces";
import {
  NInput,
  NInputNumber,
  NSpace,
  NCard,
  NModal,
  NButton,
  NTag,
  NDatePicker,
  NCheckbox,
} from "naive-ui";
import SmartSelect from "../../SmartSelect.vue";
import { isEqual } from "lodash";
import UserPermission from "../UserPermission.vue";
import { parseRowTextType } from "../../util";
import getSymbolFromCurrency from "currency-symbol-map";
import { useI18n } from "vue-i18n";
import { getTitleByFormRow } from "../../util";
import { getIDElement } from "../../../../composables/IDElement";
import { useMySelfUser } from "../../../../stores/me";
import { UserType } from "../../../../api/user_system/models";
import OrderItemList from "./OrderItemList.vue";
import { GuestOrderStatus, OrderType } from "../../../../api/erp/model";
import CloseButton from "../../../CloseButton.vue";
import { getUserElement } from "../../../../composables/UserElement";
import LinkedUsers from "./LinkedUsers.vue";
import { getGuestOrderStatusElement } from "../../../../composables/GuestOrderStatusElement";
import { useWindowSize } from "@vueuse/core";

const props = defineProps<{
  formRows: FormRow[];
  confirmCallback?: (result: T, modalType: ModalType) => void;
}>();
const { t } = useI18n();
const myself = useMySelfUser();
const template = ref();
const mutTemplate = ref();
const modalType = ref<ModalType>(ModalType.Read);
const show = ref(false);
const showAddMore = ref(false);
const limit = ref(30);
const { width } = useWindowSize();

const title = computed(() => {
  switch (modalType.value) {
    case ModalType.Add:
      return t("action.add");
    case ModalType.Read:
    case ModalType.Update:
      return t("action.detail");
    default:
      return "Modal..";
  }
});
const confirmBtnText = computed(() => {
  switch (modalType.value) {
    case ModalType.Add:
      return t("action.add");
    case ModalType.Update:
      return t("action.update");
    default:
      return "Unknown";
  }
});

async function confirmBtnClicked(result: T, mt: ModalType) {
  if (props.confirmCallback) {
    props.confirmCallback(toRaw(result), mt);
    show.value = false;
  }
}

const isDisable = (row: FormRow) => {
  return (
    modalType.value == ModalType.Read ||
    row.disabled ||
    (modalType.value == ModalType.Update && row.noUpdate) ||
    (modalType.value == ModalType.Add && row.initSelf)
  );
};

const rowIsVisible = (row: FormRow) => {
  if (row.visibleIf) {
    return row.visibleIf(mutTemplate.value);
  }
  return true;
};

defineExpose({
  showModal(_template: T, mt: ModalType) {
    const raw = toRaw(_template);
    template.value = structuredClone(raw);
    mutTemplate.value = structuredClone(raw);
    modalType.value = mt;
    show.value = true;
  },
});

const updatedSet = ref(new Set());
watch(template, () => updatedSet.value.clear());
function checkUpdate(key: string) {
  console.log('hello')
  if (!isEqual(template.value[key], mutTemplate.value[key])) {
    updatedSet.value.add(key);
  } else {
    updatedSet.value.delete(key);
  }
}
</script>

<template>
  <n-modal v-model:show="show">
    <n-card :style="width >= 1024 ? { width: '50%' } : {}" :title="title" :bordered="false" size="huge" role="dialog"
      aria-modal="true">
      <template #header-extra>
        <CloseButton @click="show = showAddMore = false" />
      </template>
      <NSpace vertical>
        <div v-for="row in formRows">
          <div v-if="
            rowIsVisible(row) &&
            (modalType != ModalType.Add || (!row.disabled && !row.initSelf))
          ">
            <label class="ml-1 mr-2">{{ getTitleByFormRow(row) }}:</label>
            <component v-if="
              row.type == FormRowType.ID ||
              row.type == FormRowType.FromGuestOrder
            " :is="getIDElement(template[row.key])"></component>
            <component v-else-if="row.type == FormRowType.User" :is="getUserElement(template[row.key])"></component>
            <span v-else-if="row.type == FormRowType.DotStatus" :class="template[row.key]? `greenDot`: `redDot`"> </span>
            <NSpace align="center" v-else-if="row.type == FormRowType.GuestOrderStatus">
              <component :is="getGuestOrderStatusElement(template[row.key])"></component>
              <n-date-picker disabled v-if="
                mutTemplate.guest_order_status === GuestOrderStatus.Confirmed
              " :value="mutTemplate.confirmed_date * 1000" type="datetime" />
            </NSpace>
            <n-input class="min-w-full" v-else-if="
              row.type == FormRowType.Text ||
              row.type == FormRowType.TextArea ||
              row.type == FormRowType.TextColor ||
              row.type == FormRowType.TextAreaColor
            " :type="parseRowTextType(row)" :disabled="isDisable(row)" clearable :default-value="mutTemplate[row.key]"
              @change="(v) => {
                mutTemplate[row.key] = v;
                checkUpdate(row.key);
              }
                " :placeholder="getTitleByFormRow(row)"></n-input>
            <div v-else-if="
              row.type == FormRowType.Date ||
              row.type == FormRowType.DatePicker
            ">
              <n-date-picker :disabled="myself.authenticated?.user.user_type !== UserType.Admin ||
                row.key !== 'date'
                " :value="(mutTemplate[row.key] ?? 0) === 0
                  ? Date.now()
                  : mutTemplate[row.key] * 1000
                  " @update-value="(v) => {
                    mutTemplate[row.key] = Math.round(v / 1000);
                    checkUpdate(row.key);
                  }
                    " type="datetime" clearable />
            </div>
            <n-input-number v-else-if="row.type == FormRowType.Number" :min="0" v-model:value="mutTemplate[row.key]"
              @change="checkUpdate(row.key)" :disabled="isDisable(row)" clearable />
            <NCheckbox v-else-if="row.type == FormRowType.CheckBox" v-model:checked="mutTemplate[row.key]" />
            <NTag v-else-if="row.type == FormRowType.TotalAmount">
              {{ getSymbolFromCurrency(mutTemplate.currency)
              }}{{ mutTemplate[row.key].toFixed(2) }}
            </NTag>
            <SmartSelect v-else-if="
              row.type == FormRowType.Area ||
              row.type == FormRowType.Person ||
              row.type == FormRowType.SKU ||
              row.type == FormRowType.SKUCategory ||
              row.type == FormRowType.Warehouse ||
              row.type == FormRowType.Order ||
              row.type == FormRowType.OrderType ||
              row.type == FormRowType.OrderPaymentStatus ||
              row.type == FormRowType.OrderCategory ||
              row.type == FormRowType.UserType ||
              row.type == FormRowType.OrderCurrency
            " :row="row" :limit="limit" :readonly="isDisable(row)" v-model:value="mutTemplate[row.key]"
              @confirm="checkUpdate(row.key)" />
            <LinkedUsers :id="mutTemplate.id" v-else-if="row.type == FormRowType.WarehouseLinkedUsers" />
            <UserPermission v-else-if="row.type == FormRowType.UserPermission" v-model:value="mutTemplate[row.key]"
              @change="checkUpdate(row.key)"></UserPermission>

            <OrderItemList v-else-if="row.type == FormRowType.OrderItems" :disable="isDisable(row)"
              :order_id="mutTemplate[row.opt.orderIdKey]" v-model:items="mutTemplate[row.key]"
              :currency="mutTemplate.currency" :enable_exchange="mutTemplate.order_type === OrderType.Exchange" />
          </div>
        </div>
      </NSpace>

      <template #footer>
        <NSpace justifty="center" :size="'small'">
          <NButton @click="confirmBtnClicked(mutTemplate, modalType)" v-if="
            modalType == ModalType.Add ||
            (modalType == ModalType.Update && updatedSet.size > 0)
          " class="m-1">{{ confirmBtnText }}</NButton>
          <slot :value="mutTemplate" :modal-type="modalType"></slot>
        </NSpace>
      </template>
    </n-card>
  </n-modal>
</template>
