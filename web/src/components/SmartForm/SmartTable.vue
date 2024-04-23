<script setup lang="ts" generic="T extends RowData">
import { TableColumn, RowData } from "naive-ui/es/data-table/src/interface";
import { DropdownMixedOption } from "naive-ui/es/dropdown/src/interface";
import getSymbolFromCurrency from "currency-symbol-map";
import {
  NSpace,
  NButton,
  NDataTable,
  NText,
  NTime,
  NDropdown,
  useDialog,
} from "naive-ui";
import { FormRow, FormRowType } from "./interfaces";
import { h, reactive, ref, Ref } from "vue";
import { Area, OrderCategory, Person } from "../../api/erp/model";
import { OrderType, SKU, SKUCategory, Warehouse } from "../../api/erp/model";
import {
  getOrderCurrencyText,
  getUserTypeText,
  get_areas_expect,
  get_persons_expect,
  get_sku_categories_expect,
  get_skus_expect,
  get_warehouses_expect,
  getItemsResult,
  get_order_categories_expect,
} from "../../util";
import { getOrderTypeElement } from "../../composables/OrderTypeElement";
import { getIDElement } from "../../composables/IDElement";
import { getTagElement } from "../../composables/TagElement";
import { getTitleByFormRow } from "./util";
import { useI18n } from "vue-i18n";
import { ListSlice } from "../../api/models";
import { watch } from "vue";
import { getOrderPaymentStatusElement } from "../../composables/OrderPaymentStatusElement";
import { getGuestOrderStatusElement } from "../../composables/GuestOrderStatusElement";
import { useWindowSize } from "@vueuse/core";

const { t, locale } = useI18n();
const props = defineProps<{
  formRows: FormRow[];
  limit: number;
  identityKey?: (row: T) => Promise<string>;
  queryCallback: (page: number, sorters: string[]) => Promise<ListSlice<T>>;
  removeCallback?: (row: T) => Promise<boolean>;
  addBaseCallback?: (row: T, remove: boolean) => Promise<boolean>;
  detailCallback: (row: T) => Promise<void>;
}>();

const { width } = useWindowSize();

const rows = ref<ListSlice<T>>({
  count: 0,
  items: [],
}) as Ref<ListSlice<T>>;
const pagination = reactive({
  page: 1,
  pageSize: props.limit,
  itemCount: rows.value.count,
  onChange: (page: number) => {
    pagination.page = page;
    refreshRows(page);
  },
});
watch(rows, () => pagination.itemCount = rows.value.count)

const dialog = useDialog();
const sorters = ref<string[]>([]);
const warehouses = ref<Map<number, Warehouse>>(new Map());
const skus = ref<Map<number, SKU>>(new Map());
const sku_categories = ref<Map<number, SKUCategory>>(new Map());
const areas = ref<Map<number, Area>>(new Map());
const persons = ref<Map<number, Person>>(new Map());
const order_categories = ref<Map<number, OrderCategory>>(new Map());
const loading = ref(false);

props.formRows.map((row) => {
  if (row.sorter) {
    sorters.value.push(`${row.key}:${row.sorter}`);
  }
});

const getRowsRefs = async () => {
  const items = rows.value.items as any;
  const arr = [
    get_warehouses_expect(items).then((v) => (warehouses.value = v)),
    get_skus_expect(items).then((v) => (skus.value = v)),
    get_sku_categories_expect(items).then((v) => (sku_categories.value = v)),
    get_areas_expect(items).then((v) => (areas.value = v)),
    get_persons_expect(items).then((v) => (persons.value = v)),
    get_order_categories_expect(items).then(
      (v) => (order_categories.value = v)
    ),
  ];
  await Promise.all(arr);
};
const refreshRows = async (p: number) => {
  loading.value = true;
  rows.value = await props.queryCallback(p, sorters.value);
  await getRowsRefs();
  loading.value = false;
};
await refreshRows(pagination.page);

const getTagElementWithNameColor = (obj: any) => {
  return getTagElement(
    obj?.name ?? t("common.unknown"),
    obj?.color,
    obj?.text_color
  );
};

function tryMinColumns(row: FormRow) {
  if (width.value > 1024) return false;

  return !(
    // if row.type is not between type.
    (row.key === "id" || row.key === "name" || row.key === "total_amount")
  );
}

const columns = ref<TableColumn[]>([]);
const setColumns = () => {
  columns.value.splice(0, columns.value.length);
  for (let i = 0; i < props.formRows.length; i++) {
    const formRow = props.formRows[i];
    if (formRow.onlyModal || tryMinColumns(formRow)) {
      continue;
    }
    columns.value.push({
      title: getTitleByFormRow(formRow),
      key: formRow.key,
      sorter: true,
      sortOrder: formRow.sorter,
      width: width.value >= 1024?100: undefined,
      render(row: any) {
        let currency =
          getSymbolFromCurrency(row.currency) ?? t("common.unknown");
        switch (formRow.type) {
          case FormRowType.ID:
            return getIDElement(row[formRow.key]);
          case FormRowType.GuestOrderStatus:
            return getGuestOrderStatusElement(row[formRow.key]);
          case FormRowType.Area:
            return getTagElementWithNameColor(
              areas.value.get(row[formRow.key])
            );
          case FormRowType.Person:
            return getTagElementWithNameColor(
              persons.value.get(row[formRow.key]) ?? { name: t("common.empty") }
            );
          case FormRowType.Warehouse:
            return getTagElementWithNameColor(
              warehouses.value.get(row[formRow.key])
            );
          case FormRowType.SKUCategory:
            return getTagElementWithNameColor(
              sku_categories.value.get(row[formRow.key])
            );
          case FormRowType.SKU:
            return getTagElementWithNameColor(skus.value.get(row[formRow.key]));
          case FormRowType.OrderPaymentStatus:
            return getOrderPaymentStatusElement(row[formRow.key]);
          case FormRowType.TextColor:
          case FormRowType.TextAreaColor:
            return getTagElement(
              row[formRow.key] ?? t("common.unknown"),
              row?.color,
              row?.text_color
            );
          case FormRowType.Text:
          case FormRowType.TextArea:
            const text = row[formRow.key] ?? t("common.unknown");
            return h(NText, () => text);
          case FormRowType.Number:
            const num = row[formRow.key] ?? t("common.unknown");
            return h(NText, () => num);
          case FormRowType.Date:
          case FormRowType.DatePicker:
            const timestamp = row[formRow.key] ?? 0;
            return h(NTime, { time: timestamp * 1000 });
          case FormRowType.Order:
            const orderId = row[formRow.key] ?? t("common.unknown");
            return h(NText, () => `${t("main.order")} (ID ${orderId})`);
          case FormRowType.OrderType:
            const ot = row[formRow.key] as OrderType;
            return getOrderTypeElement(ot);
          case FormRowType.OrderCategory:
            return getTagElementWithNameColor(
              order_categories.value.get(row[formRow.key])
            );
          case FormRowType.UserType:
            return getTagElement(getUserTypeText(row[formRow.key]));
          case FormRowType.OrderCurrency:
            return getTagElement(getOrderCurrencyText(row[formRow.key]));
          case FormRowType.UserPermission:
            return getTagElement(row[formRow.key] ?? 0);
          case FormRowType.OrderItems:
            let result = getItemsResult(row[formRow.key]);
            return h(
              NText,
              () => `${currency}${result.totalAmount.toFixed(2)}`
            );
          case FormRowType.TotalAmount:
          case FormRowType.TotalAmountInput:
            let totalAmount = row[formRow.key] as number;
            return h(NText, () => `${currency}${totalAmount.toFixed(2)}`);
          default:
            return h(NText, () => t("common.unknown"));
        }
      },
    });
  }

  columns.value.push({
    title: t("common.action"),
    key: "action",
    fixed: "right",
    width: 100,
    render(row: any, rowIndex: number) {
      const actionOptions: DropdownMixedOption[] = [];

      if (props.addBaseCallback) {
        actionOptions.push({
          label: t("action.based"),
          key: "base",
          children: [
            {
              label: t("action.addBasedOnly"),
              key: "addbaseonly",
            },
            {
              label: t("action.addAndRemove"),
              key: "addandremove",
            },
          ],
        });
      }
      if (props.removeCallback) {
        actionOptions.push({
          label: t("action.remove"),
          key: "remove",
        });
      }

      async function handleSelect(key: string) {
        const identity = props.identityKey
          ? await props.identityKey(row)
          : "Selected Row";
        switch (key) {
          case "addbaseonly":
            if (props.addBaseCallback) {
              await props.addBaseCallback!(row, false);
            }
            break;
          case "addandremove":
            if (props.addBaseCallback) {
              dialog.warning({
                title: t("common.confirmTitle"),
                content: t("message.askAddAndRemove", { obj: identity }),
                positiveText: t("action.yes"),
                negativeText: t("action.no"),
                async onPositiveClick() {
                  if (await props.addBaseCallback!(row, true)) {
                    rows.value.items.splice(rowIndex, 1);
                  }
                },
              });
            }
            break;
          case "remove":
            if (props.removeCallback) {
              dialog.error({
                title: t("common.confirmTitle"),
                content: t("message.askRemove", { obj: identity }),
                positiveText: t("action.yes"),
                negativeText: t("action.no"),
                async onPositiveClick() {
                  if (await props.removeCallback!(row)) {
                    rows.value.items.splice(rowIndex, 1);
                  }
                },
              });
            }
            break;
          default:
            break;
        }
      }

      return h(
        NDropdown,
        {
          options: actionOptions,
          onSelect: handleSelect,
          renderLabel(info) {
            let text_type:
              | "default"
              | "success"
              | "info"
              | "warning"
              | "error" = "default";
            if (info.key == "detail") {
              text_type = "info";
            } else if (info.key == "remove" || info.key == "addandremove") {
              text_type = "error";
            }
            return h(
              NText,
              {
                type: text_type,
              },
              () => info.label
            );
          },
        },
        () =>
          h(
            NButton,
            {
              onClick(e) {
                e.stopPropagation();
                props.detailCallback(row);
              },
            },
            () => t("action.detail")
          )
      );
    },
  });
};
setColumns();
watch(locale, setColumns);
watch(width, setColumns);

async function handleSorterChange(sorter: any) {
  const column: any = columns.value.find(
    (v: any) => v.key === sorter.columnKey
  );
  if (column) {
    const values = sorters.value.filter((v) => {
      const key = v.replace(/\:.+$/, "");
      return key !== column.key;
    });
    column.sortOrder = sorter.order;

    if (sorter.order) {
      values.push(`${column.key}:${sorter.order}`);
    }
    sorters.value = values;

    await refreshRows(pagination.page);
  }
}

defineExpose({
  refreshRows,
  async refreshRow(id: number, v: any) {
    const index = rows.value.items.findIndex((v) => v.id == id);
    if (index != -1) {
      rows.value.items[index] = v;
      await getRowsRefs();
    }
  },
});
</script>

<template>
  <div>
    <NSpace vertical justify="center">
      <n-data-table
        :columns="columns"
        :data="rows.items"
        :bordered="false"
        :row-key="(row: any) => row.id"
        :pagination="pagination"
        @update:sorter="handleSorterChange"
      />
    </NSpace>
  </div>
</template>

<style scoped></style>
