import { MessageApi } from "naive-ui";
import { get_skus } from "../../api/erp";
import {
  CheckOrderResult,
  ItemNotAvailable,
  OrderItem,
  OrderType,
} from "../../api/erp/model";
import { isEmptyOrSpaces } from "../../util";
import { FormRow, FormRowType } from "./interfaces";
import { i18n } from "../../i18n";

export function getTitleByFormRow(row: FormRow): string {
  const { t } = i18n.global;

  switch (row.key) {
    case "id":
      return "ID";
    case "from_guest_order_id":
      return t("common.fromGuestOrderID");
    case "created_by_user_id":
      return t("common.createdByUserID");
    case "updated_by_user_id":
      return t("common.updatedByUserID");
    case "name":
      return t("common.name");
    case "alias":
      return t("common.alias");
    case "username":
      return t("common.username");
    case "password":
      return t("common.password");
    case "description":
      return t("common.description");
    case "remark":
      return t("common.remark");
    case "date":
      return t("common.date");
    case "last_updated_date":
      return t("common.lastUpdatedDate");
    case "creation_date":
      return t("common.creationDate");
    case "actual_date":
      return t("common.actualDate");
    case "quantity":
      return t("common.quantity");
    case "area_id":
      return t("main.area");
    case "person_related_id":
      return t("common.personRelated");
    case "person_in_charge_id":
      return t("common.personInCharge");
    case "address":
      return t("common.address");
    case "contact":
      return t("common.contact");
    case "email":
      return t("common.email");
    case "user_id":
      return t("main.user");
    case "warehouse_id":
      return t("main.warehouse");
    case "sku_category_id":
    case "category_id":
      return t("main.SKUCategory");
    case "sku_id":
      return t("main.SKU");
    case "order_id":
      return t("main.order");
    case "order_type":
      return t("common.orderType");
    case "is_record":
      return t("common.isRecord");
    case "order_category_id":
      return t("main.orderCategory");
    case "items":
      return t("common.orderItems");
    case "user_type":
      return t("common.userType");
    case "permission":
      return t("common.userPermission");
    case "currency":
      return t("common.currency");
    case "color":
      return t("common.color");
    case "text_color":
      return t("common.textColor");
    case "total_amount":
      return t("common.totalAmount");
    case "total_amount_settled":
      return t("common.totalAmountSettled");
    case "order_payment_status":
      return t("main.orderPaymentStatus");
    case "warehouse_linked_users":
      return t("common.warehouseLinkedUsers");
    case "guest_order_status":
      return t("main.guestOrderStatus");
    case "is_connected":
      return t("common.userStatus");
  }

  return "Unknown title";
}

export async function getCheckOrderResultToStringArray(
  cached: any,
  orderType: OrderType,
  cor: CheckOrderResult
): Promise<string[][]> {
  const { t } = i18n.global;
  const map = new Map();
  for (let i = 0; i < cor.items_not_available.length; i++) {
    const item = cor.items_not_available[i] as ItemNotAvailable;
    const sku = await cached.getSKU(item.sku_id);
    const category = await cached.getSKUCategory(sku.sku_category_id);
    let subArr = map.get(category) ?? [];
    subArr.push(item);
    map.set(category, subArr);
  }
  const msg: string[][] = [];
  const categories = Array.from(map.keys());

  if (
    (orderType == OrderType.StockOut || orderType == OrderType.Exchange) &&
    categories.length > 0
  ) {
    for (let i = 0; i < categories.length; i++) {
      const category = categories[i];
      const items = map.get(category)!;
      const arr = [`[${category.name}]`];
      for (let j = 0; j < items.length; j++) {
        const item = items[j] as ItemNotAvailable;
        const sku = await cached.getSKU(item.sku_id);
        arr.push(
          t("result.checkResult.requireActually", {
            sku: sku.name,
            require: item.require_quantity,
            actually: item.actual_quantity,
            notAvailable: item.require_quantity - item.actual_quantity,
          })
        );
      }
      msg.push(arr);
    }
    return msg;
  } else if (
    orderType == OrderType.Verification ||
    (orderType == OrderType.VerificationStrict && categories.length > 0)
  ) {
    for (let i = 0; i < categories.length; i++) {
      const category = categories[i];
      const items = map.get(category)!;
      const arr = [`[${category.name}]`];
      for (let j = 0; j < items.length; j++) {
        const item = items[j] as ItemNotAvailable;
        const sku = await cached.getSKU(item.sku_id);
        arr.push(
          item.require_quantity < item.actual_quantity
            ? t("result.checkResult.requireActuallyLess", {
              sku: sku.name,
              require: item.require_quantity,
              actually: item.actual_quantity,
              notAvailable: item.actual_quantity - item.require_quantity,
            })
            : t("result.checkResult.requireActuallyMore", {
              sku: sku.name,
              require: item.require_quantity,
              actually: item.actual_quantity,
              notAvailable: item.require_quantity - item.actual_quantity,
            })
        );
      }
      msg.push(arr);
    }
    return msg;
  }
  return [["Empty!"]];
}

export function getDefaultItem(exchanged: boolean): OrderItem {
  return {
    sku_id: 0,
    quantity: 1,
    price: 0,
    exchanged,
  };
}

export async function getSKUs(
  message: MessageApi,
  skus_category_id: number | undefined,
  itemsText: string,
  skus_price: string
): Promise<OrderItem[] | undefined> {
  const itemList = itemsText.split("\n");
  const ids = [];
  const quantities = [];
  for (let i = 0; i < itemList.length; i++) {
    const result = itemList[i].trim().match(/(.+?)[\s\-*@]+(\d+)$/su);
    if (result) {
      const name = result[1].trim();
      const quantity = Number.parseInt(result[2].trim());
      if (isNaN(quantity)) {
        continue;
      }
      const skus = await get_skus({
        index: 0,
        limit: 2,
        name,
        sku_category_id: skus_category_id,
      });
      if (skus.count == 0) {
        message.error(`The SKU name '${name}' is not found!`);
        return;
      } else if (skus.count > 1) {
        message.warning(
          `The SKU name '${name}' have other similar names. Will select most similar one.`
        );
      }
      ids.push(skus.items[0].id);
      quantities.push(quantity);
    }
  }

  const _prices = skus_price.split("\n");
  const prices = [];
  for (let i = 0; i < _prices.length; i++) {
    if (isEmptyOrSpaces(_prices[i])) {
      continue;
    }
    const price = Number.parseFloat(_prices[i]);
    if (isNaN(price)) {
      message.error(`The no.${i + 1} price is not a number.`);
      return;
    }
    prices.push(price);
  }
  if (prices.length == 1) {
    const price = prices[0];
    for (let n = 1; n < ids.length; n++) {
      prices.push(price);
    }
  }

  if (ids.length != quantities.length || ids.length != prices.length) {
    message.error(
      `Can't add. Have ${ids.length} items, and ${prices.length} prices. They is not same!`
    );
    return;
  }
  let items = [];
  for (let i = 0; i < ids.length; i++) {
    const sku_id = ids[i];
    const quantity = quantities[i];
    const price = prices[i];
    items.push({
      sku_id,
      quantity,
      price,
      exchanged: false,
    });
  }
  return items;
}

export function parseRowTextType(row: FormRow) {
  switch (row.type) {
    default:
    case FormRowType.Text:
    case FormRowType.TextColor:
      return "text";
    case FormRowType.TextArea:
    case FormRowType.TextAreaColor:
      return "textarea";
  }
}

export function catchTexts(texts: string) {
  return texts.replace(/\d+/g, "").replace(/^\s+|\s+$/gm, "");
}

export function catchNumbers(numbers: string) {
  return numbers.replace(/[^\d\n]+/g, "").replace(/^\s+|\s+$/gm, "");
}
