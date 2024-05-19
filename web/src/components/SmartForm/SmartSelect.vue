<script setup lang="ts">
import { ref, watch, onMounted, h, VNodeChild } from "vue";
import {
  get_areas,
  get_order_categories,
  get_orders,
  get_persons,
} from "../../api/erp";
import { get_warehouses, get_sku_categories, get_skus } from "../../api/erp";
import {
  getOrderCurrencyText,
  getOrderPaymentStatusText,
  getOrderTypeText,
  getUserTypeText,
  get_persons_expect,
  get_sku_categories_expect,
} from "../..//util";
import { FormRowType, FormRow } from "./interfaces";
import { Option } from "naive-ui/es/transfer/src/interface";
import { NSelect, NPagination, NSpace, NTime } from "naive-ui";
import {
  Order,
  OrderCurrency,
  OrderPaymentStatus,
  OrderType,
  Person,
  SKU,
  SKUCategory,
} from "../../api/erp/model";
import { UserType } from "../../api/user_system/models";
import { SelectOption, NText } from "naive-ui";
import { getOrderTypeElement } from "../../composables/OrderTypeElement";
import { getTagElement } from "../../composables/TagElement";
import { useCached } from "../../stores/cached";
import { getTitleByFormRow } from "./util";
import { getOrderPaymentStatusElement } from "../../composables/OrderPaymentStatusElement";
import { get_users } from "../../api/user_system";
import { isEqual } from "lodash";
import getSymbolFromCurrency from "currency-symbol-map";

const props = defineProps<{
  limit?: number;
  row: FormRow;
  readonly?: boolean;
  value: any;
  placeholder?: string;
  multiple?: boolean;
  disabledValue?: any;
}>();

const emit = defineEmits<{
  (e: "update:value", v: any): void;
  (e: "confirm", v: any): void;
}>();
const query = ref<any>({
  ...props.row.query,
  index: 0,
  limit: 30,
});
const cached = useCached();
const labelKey = ref<string | undefined>();
const searchKey = ref<string | undefined>();
const valueKey = ref<string | undefined>();
const count = ref(0);
const isLoading = ref(false);
const sku_categories = ref<Map<number, SKUCategory>>(new Map());
const persons = ref<Map<number, Person>>(new Map());
const sources = ref<Map<any, any>>(new Map());
const options = ref<Option[]>([]);
let disabledValue = ref<Set<any>>(new Set());
const reloadDisabledValue = () => {
  if (props.disabledValue instanceof Array) {
    disabledValue.value = new Set(props.disabledValue);
  } else {
    disabledValue.value = new Set([props.disabledValue]);
  }
};
reloadDisabledValue();

let refreshFunc: (query: any) => any;
let getTargetFunc: (v: any) => any;
switch (props.row.type) {
  case FormRowType.User:
    labelKey.value = "alias";
    searchKey.value = "alias";
    valueKey.value = "id";
    refreshFunc = get_users;
    getTargetFunc = cached.getUser;
    break;
  case FormRowType.Order:
    labelKey.value = "id";
    searchKey.value = "fuzzy";
    valueKey.value = "id";
    refreshFunc = get_orders;
    getTargetFunc = cached.getOrder;
    query.value.sorters = ["date:descend"];
    break;
  case FormRowType.Area:
    labelKey.value = "name";
    searchKey.value = "name";
    valueKey.value = "id";
    refreshFunc = get_areas;
    getTargetFunc = cached.getArea;
    break;
  case FormRowType.Person:
    labelKey.value = "name";
    searchKey.value = "name";
    valueKey.value = "id";
    refreshFunc = get_persons;
    getTargetFunc = cached.getPerson;
    break;
  case FormRowType.Warehouse:
    labelKey.value = "name";
    searchKey.value = "name";
    valueKey.value = "id";
    refreshFunc = get_warehouses;
    getTargetFunc = cached.getWarehouse;
    break;
  case FormRowType.SKU:
    labelKey.value = "name";
    searchKey.value = "name";
    valueKey.value = "id";
    refreshFunc = get_skus;
    getTargetFunc = cached.getSKU;
    break;
  case FormRowType.SKUCategory:
    labelKey.value = "name";
    searchKey.value = "name";
    valueKey.value = "id";
    refreshFunc = get_sku_categories;
    getTargetFunc = cached.getSKUCategory;
    break;
  case FormRowType.OrderCategory:
    labelKey.value = "name";
    searchKey.value = "name";
    valueKey.value = "id";
    refreshFunc = get_order_categories;
    getTargetFunc = cached.getOrderCategory;
    break;
  case FormRowType.OrderType:
    labelKey.value = "name";
    searchKey.value = "name";
    valueKey.value = "id";
    refreshFunc = async (q) => {
      const values = Object.values(OrderType);
      const items = [];
      for (let i = 0; i < values.length; i++) {
        const id = values[i];
        const name = getOrderTypeText(id);
        if (
          typeof q.name === "string" &&
          !name.toLowerCase().includes(q.name.toLocaleLowerCase())
        ) {
          continue;
        }
        items.push({
          name,
          id,
        });
      }
      return {
        count: items.length,
        items,
      };
    };
    getTargetFunc = async (v) => {
      return {
        name: getOrderTypeText(v),
        id: v,
      };
    };
    break;
  case FormRowType.OrderPaymentStatus:
    labelKey.value = "name";
    searchKey.value = "name";
    valueKey.value = "id";
    refreshFunc = async (q) => {
      const values = Object.values(OrderPaymentStatus);
      const items = [];
      for (let i = 0; i < values.length; i++) {
        const id = values[i];
        const name = getOrderPaymentStatusText(id);
        if (
          typeof q.name === "string" &&
          !name.toLowerCase().includes(q.name.toLocaleLowerCase())
        ) {
          continue;
        }
        items.push({
          name,
          id,
        });
      }
      return {
        count: items.length,
        items,
      };
    };
    getTargetFunc = async (v) => {
      return {
        name: getOrderTypeText(v),
        id: v,
      };
    };
    break;
  case FormRowType.UserType:
    labelKey.value = "name";
    searchKey.value = "name";
    valueKey.value = "id";
    refreshFunc = async (q) => {
      const values = Object.values(UserType);
      const items = [];
      for (let i = 0; i < values.length; i++) {
        const id = values[i];
        const name = getUserTypeText(id);
        if (
          typeof q.name === "string" &&
          !name.toLowerCase().includes(q.name.toLocaleLowerCase())
        ) {
          continue;
        }
        items.push({
          name,
          id,
        });
      }
      return {
        count: items.length,
        items,
      };
    };
    getTargetFunc = async (v) => {
      return {
        name: getUserTypeText(v),
        id: v,
      };
    };
    break;
  case FormRowType.OrderCurrency:
    labelKey.value = "name";
    searchKey.value = "name";
    valueKey.value = "id";
    refreshFunc = async (q: any) => {
      const values = Object.values(OrderCurrency);
      const items = [];
      for (let i = 0; i < values.length; i++) {
        const id = values[i];
        const name = getOrderCurrencyText(id);
        if (
          typeof q.name === "string" &&
          !name.toLowerCase().includes(q.name.toLocaleLowerCase())
        ) {
          continue;
        }
        items.push({
          name,
          id,
        });
      }
      return {
        count: items.length,
        items,
      };
    };
    getTargetFunc = async (v) => {
      return {
        name: getOrderCurrencyText(v),
        id: v,
      };
    };
    break;
  default:
    throw new Error("Not support the row type!");
}
const refreshOptions = async () => {
  isLoading.value = true;
  let r = await refreshFunc(query.value);
  count.value = r.count;
  const arr: Option[] = [];
  const sourceArr = new Map();
  for (let i = 0; i < r.items.length; i++) {
    const item = r.items[i];
    const disabled = disabledValue.value.has(item[valueKey.value!]);
    if (item[valueKey.value!]) {
      sourceArr.set(item[valueKey.value!], item);
      arr.push({
        label: item[labelKey.value!],
        value: item[valueKey.value!],
        disabled,
      });
    }
  }
  sku_categories.value = await get_sku_categories_expect(r.items);
  persons.value = await get_persons_expect(r.items);
  sources.value = sourceArr;
  options.value = arr;
  await tryGetUnknownValue(props.multiple, props.value);
  isLoading.value = false;
};

const renderLabel = (option: SelectOption): VNodeChild => {
  let arr = [];
  const rt = props.row.type;
  const source = sources.value.get(option.value);
  if (!source) {
    return;
  }
  if (rt == FormRowType.SKU) {
    const sku = source as SKU;
    const category = sku_categories.value.get(sku?.sku_category_id ?? -1);
    arr.push(
      getTagElement(
        category?.name ?? "Unknown category",
        category?.color,
        category?.text_color
      )
    );
  } else if (rt == FormRowType.Order) {
    const order = source as Order;
    const timestamp = order.date ?? 0;
    const person = persons.value.get(order?.person_related_id);
    arr.push(
      h(NTime, {
        time: timestamp * 1000,
      }),
      getTagElement(
        person?.name ?? "Unknown person",
        person?.color,
        person?.text_color
      ),
      getTagElement(
        `${getSymbolFromCurrency(order.currency) ?? "Unk"} ${order.total_amount
        }`
      )
    );
  }

  arr.push(
    h(
      NText,
      {
        style: {
          marginLeft: "6px",
        },
      },
      () => option.label
    )
  );

  switch (rt) {
    case FormRowType.OrderType:
      const ot = option.value as OrderType;
      arr[arr.length - 1] = getOrderTypeElement(ot);
      break;

    case FormRowType.OrderPaymentStatus:
      const ops = option.value as OrderPaymentStatus;
      arr[arr.length - 1] = getOrderPaymentStatusElement(ops);
      break;
    case FormRowType.Area:
    case FormRowType.Person:
    case FormRowType.Warehouse:
    case FormRowType.SKUCategory:
    case FormRowType.SKU:
    case FormRowType.OrderCategory:
    case FormRowType.Order:
      arr[arr.length - 1] = getTagElement(
        source[labelKey.value!],
        source?.color,
        source?.text_color
      );
      break;
    default:
      break;
  }
  return arr;
};
const tryGetUnknownValue = async (multiple: boolean, valueOrArr: any) => {
  if (multiple) {
    if (valueOrArr?.length) {
      for (let i = 0; i < valueOrArr.length; i++) {
        const value = valueOrArr[i];
        if (options.value.findIndex((item) => item.value == value) == -1) {
          let item = await getTargetFunc(value);
          if (item && searchKey.value && valueKey.value) {
            sources.value.set(item[valueKey.value], item);
            if (
              value &&
              options.value.findIndex((item) => item.value == value) == -1
            ) {
              // check again before push.
              options.value.splice(0, 0, {
                label: item[searchKey.value],
                value: item[valueKey.value],
              });
              count.value += 1;
            }
          }
        }
      }
      sku_categories.value = await get_sku_categories_expect(
        Array.from(sources.value.values())
      );
    }
    return;
  } else {
    const value = valueOrArr;
    if (value && options.value.findIndex((item) => item.value == value) == -1) {
      let item = await getTargetFunc(value);

      if (item && searchKey.value && valueKey.value) {
        sources.value.set(item[valueKey.value], item);
        sku_categories.value = await get_sku_categories_expect(
          Array.from(sources.value.values())
        );
        if (
          value &&
          options.value.findIndex((item) => item.value == value) == -1
        ) {
          // check again before push.
          options.value.splice(0, 0, ({
            label: item[searchKey.value],
            value: item[valueKey.value],
          }));
          count.value += 1;
        }
      }
    }
  }
};

function valueIsNotNull(v: any): boolean {
  if (props.multiple) {
    return v && v.length > 0;
  } else {
    return v !== undefined && v !== null && v !== 0;
  }
}
async function initValue() {
  if (
    props.row.type === FormRowType.OrderType ||
    props.row.type === FormRowType.OrderPaymentStatus ||
    props.row.type === FormRowType.UserType ||
    props.row.type === FormRowType.OrderCurrency ||
    props.row.type === FormRowType.OrderCategory
  ) {
    await refreshOptions();
  } else if (valueIsNotNull(props.value) && searchKey.value && valueKey.value) {
    await tryGetUnknownValue(props.multiple, props.value);
  } else {
    await refreshOptions();
  }
}
onMounted(initValue);
watch(props, (newValue, oldValue) => {
  if (!isEqual(newValue.disabledValue, oldValue.disabledValue)) {
    reloadDisabledValue();
  }
  if (
    newValue.row != oldValue.row ||
    !isEqual(newValue.value, oldValue.value)
  ) {
    initValue();
  }
});
</script>

<template>
  <n-select v-bind="$attrs" class="min-w-[240px] w-full" :consistent-menu-width="false" :loading="isLoading"
    :disabled="readonly || row.disabled" clearable @clear="async () => {
      if (searchKey && query[searchKey] != undefined) {
        query[searchKey] = undefined;
      }
      await refreshOptions();
    }
      " :placeholder="getTitleByFormRow(row)" :options="options" :value="value" :filterable="searchKey != undefined"
    remote :multiple="multiple" @blur="async () => {
      if (searchKey && query[searchKey] != undefined && !value) {
        query[searchKey] = undefined;
        await refreshOptions();
      }
    }
      " @search="async (v) => {
        query[searchKey!] = v;
        await refreshOptions();
      }
        " @update:value="async (v) => {
          if (!valueIsNotNull(v)) {
            await refreshOptions();
          }
          emit('update:value', v);
          emit('confirm', v);
        }
          " :fallback-option="false" :render-label="renderLabel">
    <template #action>
      <NSpace justify="center" vertical>
        <slot></slot>
        <n-pagination :page="query.index + 1" :page-size="query.limit" :item-count="count" simple @update-page="async (p) => {
          query.index = p - 1;
          await refreshOptions();
        }
          " />
      </NSpace>
    </template>
  </n-select>
</template>
