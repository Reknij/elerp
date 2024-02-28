<script setup lang="ts">
import { useI18n } from "vue-i18n";
import {
  NButton,
  NCard,
  NInputNumber,
  NSpace,
  NTag,
  NVirtualList,
  useDialog,
  useMessage,
  InputNumberInst,
  NSwitch,
} from "naive-ui";
import { ref } from "vue";
import { FormRowType } from "../../interfaces";
import { OrderCurrency, OrderItem, OrderType } from "../../../../api/erp/model";
import getSymbolFromCurrency from "currency-symbol-map";
import { useCached } from "../../../../stores/cached";
import { getItemsResult } from "../../../../util";
import Result from "../Result.vue";
import TotalResult from "./TotalResult.vue";
import AddMoreModal from "./AddMoreModal.vue";
import SmartSelect from "../../SmartSelect.vue";
import { getDefaultItem } from "../../util";
import { toRaw } from "vue";
import { onMounted } from "vue";
import CloseButton from "../../../CloseButton.vue";
import { getOrderTypeElement } from "../../../../composables/OrderTypeElement";
import { error_to_string, fmt_err } from "../../../../AppError";
import { nextTick } from "vue";
import SmartRow from "../../SmartRow.vue";

const props = defineProps<{
  items: OrderItem[];
  disable?: boolean;
  enable_exchange?: boolean;
  currency: OrderCurrency;
}>();
const emit = defineEmits<{
  (e: "update:items", value: OrderItem[]): void;
}>();

interface OrderItemUnique {
  raw: OrderItem;
  key: number;
  hide: boolean;
  focus: number;
  beforeTimeOut?: number;
}

const { t } = useI18n();
const message = useMessage();
const cached = useCached();
const dialog = useDialog();
const showAddMore = ref(false);
const showItemsDetail = ref(false);
const limit = ref(30);
const lastItemsDetail = ref<string[][]>([]);
const currencySymbol = ref(getSymbolFromCurrency(props.currency) ?? "Unk!");

const normalDirectInput = ref<InputNumberInst>();
const exchangedDirectInput = ref<InputNumberInst>();
const normalDirectInputSwitch = ref(false);
const exchangedDirectInputSwitch = ref(false);

const normalDirectInputValue = ref<number | null>(null);
const exchangedDirectInputValue = ref<number | null>(null);

const normalItems = ref<OrderItemUnique[]>([]);
const exchangedItems = ref<OrderItemUnique[]>([]);

const addMoreIsExchange = ref(false);
let uniqueKeyStart = 0;
function fetchItems() {
  for (let i = 0; i < props.items.length; i++) {
    const item = {
      raw: props.items[i],
      key: uniqueKeyStart++,
      hide: true,
      focus: 0,
    };
    if (item.raw.exchanged) {
      exchangedItems.value.push(item);
    } else {
      normalItems.value.push(item);
    }
  }
}
onMounted(() => {
  fetchItems();
});

function sureToClearSKUs(is_exchange: boolean) {
  dialog.warning({
    title: t("common.confirmTitle"),
    content: t("message.clearAll"),
    positiveText: t("action.yes"),
    negativeText: t("action.no"),
    onPositiveClick() {
      if (is_exchange) {
        exchangedItems.value.splice(0, exchangedItems.value.length);
      } else {
        normalItems.value.splice(0, normalItems.value.length);
      }
    },
  });
}

function addEmptyItem(exchanged: boolean) {
  const item = {
    raw: getDefaultItem(exchanged),
    key: uniqueKeyStart++,
    hide: true,
    focus: 0,
  };
  if (exchanged) {
    exchangedItems.value.push(item);
  } else {
    normalItems.value.push(item);
  }
  updateItemSource();
}

function removeTargetItem(index: number, exchanged: boolean) {
  if (exchanged) {
    exchangedItems.value.splice(index, 1);
  } else {
    normalItems.value.splice(index, 1);
  }
  updateItemSource();
}

function updateItemSource() {
  emit("update:items", [
    ...toRaw(normalItems.value.map((item) => toRaw(item.raw))),
    ...toRaw(exchangedItems.value.map((item) => toRaw(item.raw))),
  ]);
}

async function showItemsResult(items: OrderItem[], simple: boolean) {
  const map = new Map<number, OrderItem[]>();
  for (let i = 0; i < items.length; i++) {
    const item = items[i];
    const sku = await cached.getSKU(item.sku_id);
    const category = await cached.getSKUCategory(sku.sku_category_id);
    if (map.has(category.id)) {
      map.get(category.id)!.push(item);
    } else {
      map.set(category.id, [item]);
    }
  }
  let msg = [];
  const categories = Array.from(map.keys());
  for (let i = 0; i < categories.length; i++) {
    const id = categories[i];
    const category = await cached.getSKUCategory(id);
    const subItems = map.get(id)!;
    const result = getItemsResult(subItems);
    let smsg = [
      `${t("result.itemsResult.haveSKUs", {
        category: category.name,
        count: subItems.length,
      })}\n`,
    ];
    for (let j = 0; j < subItems.length; j++) {
      const item = subItems[j];
      const sku = await cached.getSKU(item.sku_id);
      if (!simple) {
        smsg.push(
          `${sku.name}, ${t("common.quantity")}: ${item.quantity}, ${t(
            "common.price"
          )}: ${currencySymbol.value}${item.price}, ${t("common.total")}: ${
            currencySymbol.value
          }${item.quantity * item.price}`
        );
      } else {
        smsg.push(`${sku.name} : ${item.quantity}`);
      }
    }
    if (!simple) {
      smsg.push(
        `- ${t("result.itemsResult.totalQuantity", {
          obj: result.totalQuantity,
        })}`,
        `- ${t("result.itemsResult.averagePrice", {
          obj: `${currencySymbol.value}${result.averagePrice.toFixed(2)}`,
        })}`,
        `- ${t("result.itemsResult.totalAmount", {
          obj: `${currencySymbol.value}${result.totalAmount.toFixed(2)}`,
        })}`
      );
    } else {
      smsg.push(
        `- ${t("result.itemsResult.totalQuantity", {
          obj: result.totalQuantity,
        })}`
      );
    }
    msg.push(smsg);
  }
  lastItemsDetail.value = msg;
  showItemsDetail.value = true;
}

async function directInputHandle(e: any, exchange: boolean) {
  if (e.keyCode === 13) {
    // enter key
    const id = exchange
      ? exchangedDirectInputValue.value
      : normalDirectInputValue.value;

    exchangedDirectInputValue.value = null;
    normalDirectInputValue.value = null;
    if (!id) {
      message.warning(t("message.isNotANumber"));
      return;
    } else if (id > 9_223_372_036_854_775_807) {
      message.warning(t("message.numberTooBig"));
      return;
    }
    try {
      const sku = await cached.getSKU(id);
      const refItems = (exchange ? exchangedItems : normalItems).value;
      let needPush = true;
      for (let i = refItems.length - 1; i >= 0; i--) {
        const item = refItems[i];
        if (item.raw.sku_id === sku.id) {
          item.raw.quantity += 1;
          needPush = false;
          break;
        }
      }
      if (needPush) {
        refItems.push({
          raw: {
            sku_id: sku.id,
            quantity: 1,
            price: 0.0,
            exchanged: false,
          },
          key: uniqueKeyStart++,
          hide: true,
          focus: 0,
        });
      }

      if (exchange) {
        nextTick(() => exchangedDirectInput.value?.focus());
      } else {
        nextTick(() => normalDirectInput.value?.focus());
      }
    } catch (error) {
      const msg = fmt_err(error, {
        obj: t("main.SKU"),
      });
      message.error(msg ?? error_to_string(error));
    }
  }
}

function setDirectInputMode(val: boolean, exchanged: boolean) {
  if (exchanged) {
    exchangedDirectInputSwitch.value = val;
    normalDirectInputSwitch.value = false;
    if (val) {
      nextTick(() => exchangedDirectInput.value?.focus());
    }
  } else {
    normalDirectInputSwitch.value = val;
    exchangedDirectInputSwitch.value = false;
    if (val) {
      nextTick(() => normalDirectInput.value?.focus());
    }
  }
}
let lastEnterItem: OrderItemUnique | null = null;
function isHideOrNot(item: OrderItemUnique) {
  if (item.beforeTimeOut) {
    clearTimeout(item.beforeTimeOut);
  }
  item.beforeTimeOut = setTimeout(() => {
    if (!item.hide && item.focus === 0 && lastEnterItem != item) {
      item.hide = true;
    } else {
      isHideOrNot(item);
    }
  }, 150);
}

function setItemHide(item: any, hide: boolean) {
  item.hide = hide;
  if (!hide) {
    lastEnterItem = item;
    isHideOrNot(item);
  } else if (lastEnterItem == item) {
    lastEnterItem = null;
  }
}
function isFocus(item: OrderItemUnique): boolean {
  return item.focus > 0;
}
function focusIt(item: OrderItemUnique) {
  item.focus += 1;
}
function blurIt(item: OrderItemUnique) {
  item.focus -= 1;
  if (!item.hide && item.focus == 0) {
    item.hide = true;
  }
}
</script>

<template>
  <div>
    <Result v-model:show="showItemsDetail" :title="t('action.detail')">
      <span v-for="(arr, i) in lastItemsDetail">
        <br v-if="i != 0" />
        <span v-for="text in arr">
          {{ text }}
          <br />
        </span>
      </span>
    </Result>
    <AddMoreModal
      v-model:show="showAddMore"
      @add-clicked="
        (items) => {
          if (!items) {
            return;
          }
          if (items.length == 0) {
            showAddMore = false;
            return;
          }
          let itemsUnique =
            items.map <
            OrderItemUnique >
            ((item) => {
              return {
                raw: item,
                exchanged: addMoreIsExchange,
                key: uniqueKeyStart++,
                hide: true,
                focus: 0,
              };
            });

          if (addMoreIsExchange) {
            exchangedItems.push(...itemsUnique);
          } else {
            normalItems.push(...itemsUnique);
          }
          updateItemSource();
          message.success(
            t('message.addMultipleSuccess', {
              obj: 'item',
              count: items.length,
            })
          );
          showAddMore = false;
        }
      "
    ></AddMoreModal>

    <NSpace vertical>
      <NCard size="small" embedded class="!border-neutral-300">
        <n-space vertical>
          <component
            v-if="enable_exchange"
            :is="getOrderTypeElement(OrderType.StockOut)"
          />
          <n-virtual-list
            key-field="key"
            style="max-height: 342px"
            :item-size="38"
            :item-resizable="!disable"
            :items="normalItems"
          >
            <template #default="{ item, index }">
              <div
                class="itemRow"
                :key="item.key"
                @[!props.disable?`mouseenter`:null]="setItemHide(item, false)"
                @[!props.disable&&!isFocus(item)?`mouseleave`:null]="
                  setItemHide(item, true)
                "
              >
                <NTag class="serialCol" round>#{{ index + 1 }}</NTag>
                <SmartRow
                  class="flex-grow"
                  v-if="item.hide"
                  :value="item.raw.sku_id"
                  :row="FormRowType.SKU"
                ></SmartRow>
                <SmartSelect
                  v-else
                  @focus="focusIt(item)"
                  @blur="blurIt(item)"
                  class="flex-grow"
                  :row="{
                    type: FormRowType.SKU,
                    key: 'sku_id',
                  }"
                  :limit="limit"
                  v-model:value="item.raw.sku_id"
                />
                <NTag v-if="item.hide">{{ item.raw.quantity }}</NTag>
                <n-input-number
                  v-else
                  v-model:value="item.raw.quantity"
                  @focus="focusIt(item)"
                  @blur="blurIt(item)"
                />
                <NTag v-if="item.hide">{{
                  `${currencySymbol} ${item.raw.price.toFixed(2)}`
                }}</NTag>
                <n-input-number
                  v-else
                  v-model:value="item.raw.price"
                  @focus="focusIt(item)"
                  @blur="blurIt(item)"
                >
                  <template #prefix>
                    {{ currencySymbol }}
                  </template>
                </n-input-number>
                <CloseButton
                  v-if="!item.hide"
                  @click="removeTargetItem(index, false)"
                />
              </div>
            </template>
          </n-virtual-list>

          <NSpace align="center" justify="space-between">
            <TotalResult
              :value="normalItems.map((item) => item.raw)"
              :currency="currency"
            ></TotalResult>
            <NSpace align="center">
              <NButton
                :size="'small'"
                v-if="normalItems.length"
                @click="
                  showItemsResult(
                    normalItems.map((item) => item.raw),
                    true
                  )
                "
                >{{ t("common.simpleResult") }}</NButton
              >
              <NButton
                :size="'small'"
                v-if="normalItems.length"
                @click="
                  showItemsResult(
                    normalItems.map((item) => item.raw),
                    false
                  )
                "
                >{{ t("common.result") }}</NButton
              >
              <NButton
                :size="'small'"
                v-if="!disable"
                @click="sureToClearSKUs(false)"
                >{{ t("action.clear") }}</NButton
              >
            </NSpace>
          </NSpace>

          <n-switch
            v-if="!disable"
            :value="normalDirectInputSwitch"
            @update-value="(val) => setDirectInputMode(val, false)"
          >
            <template #checked>
              {{ t("action.useDirectInput") }}
            </template>
            <template #unchecked>
              {{ t("action.useManualInput") }}
            </template>
          </n-switch>

          <div v-if="!disable">
            <NSpace align="center" v-if="normalDirectInputSwitch">
              <n-input-number
                v-model:value="normalDirectInputValue"
                ref="normalDirectInput"
                clearable
                placeholder="SKU ID"
                @keydown="async (e) => directInputHandle(e, false)"
                class="w-full"
              />
              <p>
                {{ t("common.directInputLabel") }}
              </p>
            </NSpace>
            <NSpace align="center" v-else>
              <n-button class="min-w-full" @click="addEmptyItem(false)">{{
                t("action.addSpecified", { obj: t("common.orderItem") })
              }}</n-button>
              <n-button
                class="min-w-full"
                @click="addMoreIsExchange = !(showAddMore = true)"
                >{{
                  t("action.addMultipleSpecified", {
                    obj: t("common.orderItem"),
                  })
                }}</n-button
              >
            </NSpace>
          </div>
        </n-space>
      </NCard>

      <NCard
        v-if="enable_exchange"
        size="small"
        embedded
        class="!border-neutral-300"
      >
        <n-space vertical class="mt-2">
          <component
            v-if="enable_exchange"
            :is="getOrderTypeElement(OrderType.Return)"
          />

          <n-virtual-list
            style="max-height: 340px"
            key-field="key"
            :item-size="34"
            :item-resizable="!disable"
            :items="exchangedItems"
          >
            <template #default="{ item, index }">
              <div
                class="itemRow"
                :key="item.key"
                @[!props.disable?`mouseenter`:null]="setItemHide(item, false)"
                @[!props.disable&&!isFocus(item)?`mouseleave`:null]="
                  setItemHide(item, true)
                "
              >
                <NTag round class="flex">{{ index + 1 }}</NTag>
                <SmartRow
                  class="flex-grow"
                  v-if="item.hide"
                  :value="item.raw.sku_id"
                  :row="FormRowType.SKU"
                ></SmartRow>
                <SmartSelect
                  v-else
                  @focus="focusIt(item)"
                  @blur="blurIt(item)"
                  class="flex-grow"
                  :row="{
                    type: FormRowType.SKU,
                    key: 'sku_id',
                  }"
                  :limit="limit"
                  v-model:value="item.raw.sku_id"
                />
                <NTag v-if="item.hide">{{ item.raw.quantity }}</NTag>
                <n-input-number
                  v-else
                  v-model:value="item.raw.quantity"
                  @focus="focusIt(item)"
                  @blur="blurIt(item)"
                />
                <NTag v-if="item.hide">{{
                  `${currencySymbol} ${item.raw.price.toFixed(2)}`
                }}</NTag>
                <n-input-number
                  v-else
                  v-model:value="item.raw.price"
                  @focus="focusIt(item)"
                  @blur="blurIt(item)"
                >
                  <template #prefix>
                    {{ currencySymbol }}
                  </template>
                </n-input-number>
                <CloseButton
                  v-if="!item.hide"
                  @click="removeTargetItem(index, true)"
                />
              </div>
            </template>
          </n-virtual-list>

          <NSpace align="center" justify="space-between">
            <TotalResult
              :value="exchangedItems.map((item) => item.raw)"
              :currency="currency"
            ></TotalResult>
            <NSpace align="center">
              <NButton
                :size="'small'"
                v-if="exchangedItems.length"
                @click="
                  showItemsResult(
                    exchangedItems.map((item) => item.raw),
                    true
                  )
                "
                >{{ t("common.simpleResult") }}</NButton
              >
              <NButton
                :size="'small'"
                v-if="exchangedItems.length"
                @click="
                  showItemsResult(
                    exchangedItems.map((item) => item.raw),
                    false
                  )
                "
                >{{ t("common.result") }}</NButton
              >
              <NButton
                :size="'small'"
                v-if="!disable"
                @click="sureToClearSKUs(true)"
                >{{ t("action.clear") }}</NButton
              >
            </NSpace>
          </NSpace>

          <n-switch
            v-if="!disable"
            :value="exchangedDirectInputSwitch"
            @update-value="(val) => setDirectInputMode(val, true)"
          >
            <template #checked>
              {{ t("action.useDirectInput") }}
            </template>
            <template #unchecked>
              {{ t("action.useManualInput") }}
            </template>
          </n-switch>

          <div v-if="!disable">
            <NSpace align="center" v-if="exchangedDirectInputSwitch">
              <n-input-number
                v-model:value="exchangedDirectInputValue"
                ref="exchangedDirectInput"
                clearable
                placeholder="SKU ID"
                @keydown="async (e) => directInputHandle(e, true)"
                class="w-full"
              />
              <p>
                {{ t("common.directInputLabel") }}
              </p>
            </NSpace>
            <NSpace align="center" v-else>
              <n-button class="min-w-full" @click="addEmptyItem(true)">{{
                t("action.addSpecified", { obj: t("common.orderItem") })
              }}</n-button>
              <n-button
                class="min-w-full"
                @click="addMoreIsExchange = showAddMore = true"
                >{{
                  t("action.addMultipleSpecified", {
                    obj: t("common.orderItem"),
                  })
                }}</n-button
              >
            </NSpace>
          </div>
        </n-space>
      </NCard>
    </NSpace>
  </div>
</template>

<style>
@media only screen and (max-width: 600px) {
  .itemRow {
    @apply flex-col items-start;
  }
  .serialCol {
    @apply !hidden;
  }
}
@media only screen and (min-width: 601px) {
  .itemRow {
    @apply items-center;
  }
}

.itemRow {
  @apply flex pr-4;
}
.itemRow > * {
  @apply m-0.5;
}
</style>
