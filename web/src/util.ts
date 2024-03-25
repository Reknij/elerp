import { DefineComponent } from "vue";
import { UserType } from "./api/user_system/models";
import {
  GuestOrderStatus,
  OrderCurrency,
  OrderItem,
  OrderPaymentStatus,
  OrderType,
} from "./api/erp/model";
import { useCached } from "./stores/cached";
import { i18n } from "./i18n";

export const ONE_DAY_SECONDS = 24 * 60 * 60;
export const ONE_DAY_MAX_SECONDS = ONE_DAY_SECONDS - 1;
export const ONE_MONTH_SECONDS = ONE_DAY_SECONDS * 31;

export function dateRangeConvertBackend(
  range: [number, number]
): [number, number] {
  let startTime = new Date(range[0]);
  // Set the hours, minutes, seconds and milliseconds to zero
  startTime.setHours(0, 0, 0);
  // Create a new date object for the end time of the target day
  let endTime = new Date(range[1]);
  // Set the hours, minutes, seconds and milliseconds to the last moment of the day
  endTime.setHours(23, 59, 59);
  const start = Math.floor(startTime.getTime() / 1000);
  const end = Math.floor(endTime.getTime() / 1000);
  return [start, end];
}

export function getStartAndEndTimestampToday(): [number, number] {
  // Get the current date
  var first = new Date();
  first.setHours(0, 0, 0);

  // Get the current date
  var last = new Date();
  last.setHours(23, 59, 59);

  return [first.getTime(), last.getTime()];
}

export function getStartAndEndTimestampCurrentMonth(): [number, number] {
  // Get the current date
  var date = new Date();

  // Get the first day of the current month
  var firstDay = new Date(date.getFullYear(), date.getMonth(), 1);
  firstDay.setHours(0, 0, 0);

  // Get the last day of the current month
  var lastDay = new Date(date.getFullYear(), date.getMonth() + 1, 0);
  lastDay.setHours(23, 59, 59);

  // Convert the dates to timestamps
  return [firstDay.getTime(), lastDay.getTime()];
}

export enum NavPath {
  Workplace = "main.workplace",
  Areas = "main.areas",
  Persons = "main.persons",
  Warehouses = "main.warehouses",
  SKUCategories = "main.SKUCategories",
  SKUs = "main.SKUs",
  Inventory = "main.inventory",
  Orders = "main.orders",
  GuestOrders = "main.guestOrders",
  OrderCategories = "main.orderCategories",
  Payments = "main.payments",
  Users = "main.users",
  UserConfigure = "main.userConfigure",
}

export function getNavLabel(nav: NavPath): string {
  const { t } = i18n.global;
  return t(nav);
}

export function isEmptyOrSpaces(str: string | null) {
  return (
    str === null || str.match(/^ *$/) !== null || str == "\n" || str == "\r\n"
  );
}

export interface ItemsResult {
  totalQuantity: number;
  totalAmount: number;
  averagePrice: number;
}

export function getItemsResult(items: OrderItem[]): ItemsResult {
  let totalQuantity = 0;
  let totalAmount = 0;
  let totalPrice = 0;
  for (let i = 0; i < items.length; i++) {
    const item = items[i] as OrderItem;
    totalQuantity += item.quantity;
    totalAmount += item.quantity * item.price;
    totalPrice += item.price;
  }
  return {
    totalAmount,
    totalQuantity,
    averagePrice: items.length > 0 ? totalPrice / items.length : 0,
  };
}

export function getGuestOrderStatusText(o: GuestOrderStatus) {
  const { t } = i18n.global;
  switch (o) {
    case GuestOrderStatus.Pending:
      return t("guestOrderStatus.pending");
    case GuestOrderStatus.Expired:
      return t("guestOrderStatus.expired");
    case GuestOrderStatus.Confirmed:
      return t("guestOrderStatus.confirmed");
    default:
      return t("common.unknown");
  }
}

export function getOrderTypeText(o: OrderType) {
  const { t } = i18n.global;
  switch (o) {
    case OrderType.StockIn:
      return t("orderType.stockIn");
    case OrderType.StockOut:
      return t("orderType.stockOut");
    case OrderType.Exchange:
      return t("orderType.exchange");
    case OrderType.Return:
      return t("orderType.return");
    case OrderType.Calibration:
      return t("orderType.calibration");
    case OrderType.CalibrationStrict:
      return t("orderType.calibrationStrict");
    case OrderType.Verification:
      return t("orderType.verification");
    case OrderType.VerificationStrict:
      return t("orderType.verificationStrict");
    default:
      return t("common.unknown");
  }
}

export function getOrderPaymentStatusText(o: OrderPaymentStatus) {
  const { t } = i18n.global;
  switch (o) {
    case OrderPaymentStatus.Settled:
      return t("orderPaymentStatus.settled");
    case OrderPaymentStatus.Unsettled:
      return t("orderPaymentStatus.unsettled");
    case OrderPaymentStatus.PartialSettled:
      return t("orderPaymentStatus.partialSettled");
    case OrderPaymentStatus.None:
      return t("orderPaymentStatus.none");
    default:
      return t("common.unknown");
  }
}

export function getUserTypeText(u: UserType) {
  const { t } = i18n.global;
  switch (u) {
    case UserType.Admin:
      return t("userType.admin");
    case UserType.General:
      return t("userType.general");
    default:
      return t("common.unknown");
  }
}

export function getOrderCurrencyText(t: OrderCurrency) {
  return OrderCurrency[t] as string;
}

export interface WarehouseId {
  warehouse_id?: number;
}

export interface AreaId {
  area_id?: number;
}

export interface PersonId {
  person_related_id?: number;
  person_in_charge_id?: number;
}

export interface CategoryId {
  category_id?: number;
  sku_category_id?: number;
}

export interface SKUId {
  sku_id?: number;
}

export interface OrderCategoryId {
  order_category_id?: number;
}

export async function get_skus_expect(arr: SKUId[]) {
  const cached = useCached();
  const rows = new Map();
  for (let i = 0; i < arr.length; i++) {
    const row = arr[i];
    if (!row.sku_id) continue;
    const id = row.sku_id;
    try {
      const v =
        id > 0
          ? await cached.getSKU(id)
          : {
              id,
              name: "Unknown",
              description: "Unknown",
            };
      rows.set(id, v);
    } catch (error: any) {
      rows.set(id, {
        id,
        name: error?.response?.data?.msg,
        description: "Unknown",
      });
    }
  }
  return rows;
}

export async function get_sku_categories_expect(arr: CategoryId[]) {
  const cached = useCached();
  const rows = new Map();
  for (let i = 0; i < arr.length; i++) {
    const row = arr[i];
    const id = row.category_id ?? row.sku_category_id ?? 0;
    try {
      const v =
        id > 0
          ? await cached.getSKUCategory(id)
          : {
              id,
              name: "Unknown",
              description: "Unknown",
            };
      rows.set(id, v);
    } catch (error: any) {
      rows.set(id, {
        id,
        name: error?.response?.data?.msg,
        description: "Unknown",
      });
    }
  }
  return rows;
}

export async function get_warehouses_expect(arr: WarehouseId[]) {
  const cached = useCached();
  const rows = new Map();
  for (let i = 0; i < arr.length; i++) {
    const row = arr[i];
    if (!row.warehouse_id) continue;
    try {
      const v =
        row.warehouse_id > 0
          ? await cached.getWarehouse(row.warehouse_id)
          : {
              id: row.warehouse_id,
              name: "Unknown",
              description: "Unknown",
            };
      rows.set(row.warehouse_id, v);
    } catch (error: any) {
      rows.set(row.warehouse_id, {
        id: row.warehouse_id,
        name: error?.response?.data?.msg,
        description: "Unknown",
      });
    }
  }
  return rows;
}

export async function get_areas_expect(arr: AreaId[]) {
  const cached = useCached();
  const row_areas = new Map();
  for (let i = 0; i < arr.length; i++) {
    const row = arr[i];
    if (!row.area_id) continue;
    try {
      const area =
        row.area_id > 0
          ? await cached.getArea(row.area_id)
          : {
              id: row.area_id,
              name: "Unknown",
              description: "Unknown",
            };
      row_areas.set(row.area_id, area);
    } catch (error: any) {
      row_areas.set(row.area_id, {
        id: row.area_id,
        name: error?.response?.data?.msg,
        description: "Unknown",
      });
    }
  }
  return row_areas;
}

export async function get_persons_expect(arr: PersonId[]) {
  const cached = useCached();
  const rows = new Map();
  for (let i = 0; i < arr.length; i++) {
    const row = arr[i];
    const ids = [row.person_related_id, row.person_in_charge_id];
    for (let j = 0; j < ids.length; j++) {
      const pid = ids[j];
      if (pid) {
        try {
          const area =
            pid > 0
              ? await cached.getPerson(pid)
              : {
                  id: pid,
                  name: "Empty",
                  description: "Empty",
                };
          rows.set(pid, area);
        } catch (error: any) {
          rows.set(pid, {
            id: pid,
            name: error?.response?.data?.msg,
            description: "Unknown",
          });
        }
      }
    }
  }
  return rows;
}

export async function get_order_categories_expect(arr: OrderCategoryId[]) {
  const cached = useCached();
  const rows = new Map();
  for (let i = 0; i < arr.length; i++) {
    const row = arr[i];
    const id = row.order_category_id ?? 0;
    try {
      const v =
        id > 0
          ? await cached.getOrderCategory(id)
          : {
              id,
              name: "Unknown",
              description: "Unknown",
            };
      rows.set(id, v);
    } catch (error: any) {
      rows.set(id, {
        id,
        name: error?.response?.data?.msg,
        description: "Unknown",
      });
    }
  }
  return rows;
}

export type ComponentInstance<T> = T extends new (...args: any[]) => infer R
  ? R
  : T extends (...args: any[]) => infer R
  ? R extends { __ctx?: infer K }
    ? Exclude<K, void> extends { expose: (...args: infer K) => void }
      ? K[0] & InstanceType<DefineComponent>
      : any
    : any
  : any;
