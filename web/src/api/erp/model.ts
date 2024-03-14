export interface ClearResult {
  success: number;
  failed: number;
}

export interface PopularSKU {
  id: number;
  currency: OrderCurrency;
  order_count: number;
  average_price: number;
  total_out: number;
}

export interface SalesAmountWithCurrency {
  any: number;
  settled: number;
  partial_settled: number;
  unsettled: number;
  currency: OrderCurrency;
}

export interface StatisticalOrderData {
  total_count: StatisticalOrderCountData;
  total_amount: SalesAmountWithCurrency[];
}

export interface StatisticalOrderCountData {
  any_count: number;
  stock_in_count: number;
  stock_out_count: number;
  exchange_count: number;
  return_count: number;
  calibration_count: number;
  calibration_strict_count: number;
  verification_count: number;
  verification_strict_count: number;
}


export interface StatisticalData {
  area_count: number;
  person_count: number;
  warehouse_count: number;
  sku_count: number;
  sku_category_count: number;
  order: StatisticalOrderData;
  order_category_count: number;
  most_popular_skus: PopularSKU[];
}

export interface GetStatisticalDataQuery {
  date_start?: number;
  date_end?: number;
  order_category_id?: number;
  warehouse_ids?: Set<number>;
  items?: Set<number>;
  item_categories?: Set<number>;
  person_related_id?: number;
  person_in_charge_id?: number;
  currency?: OrderCurrency;
  reverse: Set<string>;
}

export interface Area {
  id?: number;
  name: string;
  description: string;
}

export interface Person {
  id?: number;
  name: string;
  description: string;
  person_in_charge_id: number;
  area_id: number;
  address: string;
  contact: string;
  email: string;
  color?: string;
  text_color?: string;
}

export interface GetAreasQuery {
  index: number;
  limit: number;
  id?: number;
  name?: string;
  sorters?: string[];
}

export interface GetPersonsQuery {
  index: number;
  limit: number;
  id?: number;
  name?: string;
  address?: string;
  area_id?: number;
  person_in_charge_id?: number;
  contact?: string;
  email?: string;
  sorters?: string[];
}

export interface Warehouse {
  id: number;
  name: string;
  description: string;
  area_id: number;
  address: string;
  person_in_charge_id: number;
  color?: string;
  text_color?: string;
}

export interface WarehouseToLinkQuery {
  user_id: number;
}

export interface GetWarehouseLinkedUsersQuery {
  index: number;
  limit: number;
}

export interface SKUCategory {
  id: number;
  name: string;
  description: string;
  color?: string;
  text_color?: string;
}

export interface SKU {
  id: number;
  sku_category_id: number;
  name: string;
  description: string;
  color?: string;
  text_color?: string;
}

export interface InventoryProduct {
  sku_id: number;
  sku_category_id: number;
  warehouse_id: number;
  quantity: number;
}

export enum OrderType {
  StockIn = "StockIn",
  StockOut = "StockOut",
  Return = "Return",
  Exchange = "Exchange",
  Calibration = "Calibration",
  CalibrationStrict = "CalibrationStrict",
  Verification = "Verification",
  VerificationStrict = "VerificationStrict",
}

export enum OrderPaymentStatus {
  Settled = "Settled",
  Unsettled = "Unsettled",
  PartialSettled = "PartialSettled",
  None = "None",
}

export enum OrderCurrency {
  CNY = "CNY",
  HKD = "HKD",
  USD = "USD",
  GBP = "GBP",
  MYR = "MYR",
  IDR = "IDR",
  INR = "INR",
  PHP = "PHP",
  Unknown = "Unknown",
}

export interface OrderItem {
  sku_id: number;
  quantity: number;
  price: number;
  exchanged: boolean;
}

export interface Order {
  id?: number;
  from_guest_order_id?: number;
  created_by_user_id?: number;
  updated_by_user_id?: number;
  date?: number;
  last_updated_date?: number;
  description: string;
  currency: OrderCurrency;
  items: OrderItem[];
  total_amount: number;
  total_amount_settled: number;
  order_payment_status: OrderPaymentStatus;
  order_type: OrderType;
  order_category_id: number;
  person_related_id: number;
  warehouse_id: number;
}

export enum GuestOrderStatus {
  Confirmed = "Confirmed",
  Pending = "Pending",
  Expired = "Expired",
}

export interface GuestOrder {
  id?: number;
  date: number;
  confirmed_date: number;
  order_id: number;
  order_category_id: number;
  sub_token: string;
  guest_order_status: GuestOrderStatus;
  created_by_user_id?: number;
  description: string;
  currency: OrderCurrency;
  items: OrderItem[];
  order_type: OrderType;
  person_related_id: number;
  warehouse_id: number;
}

export interface GuestOrderConfirm {
  check_result: CheckOrderResult;
  order?: GuestOrder;
}

export interface OrderCategory {
  id: number;
  name: string;
  description: string;
  color?: string;
  text_color?: string;
}

export interface OrderPayment {
  id: number;
  order_id: number;
  person_in_charge_id: number;
  creation_date: number;
  actual_date: number;
  total_amount: number;
  remark: string;
}

export interface GetInventoryQuery {
  index: number;
  limit: number;
  id?: number;
  warehouse_id?: number;
  sku_id?: number;
  sku_category_id?: number;
  quantity_start?: number;
  quantity_end?: number;
  sorters: string[];
}

export interface GetOrdersQuery {
  index: number;
  limit: number;
  fuzzy?: string;
  id?: number;
  created_by_user_id?: number;
  updated_by_user_id?: number;
  warehouse_ids?: Set<number>;
  items?: Set<number>;
  item_categories?: Set<number>;
  person_related_id?: number;
  person_in_charge_id?: number;
  order_type?: OrderType;
  order_category_id?: number;
  order_payment_status?: Set<OrderPaymentStatus>;
  currency?: OrderCurrency;
  date_start?: number;
  date_end?: number;
  last_updated_date_start?: number;
  last_updated_date_end?: number;
  sorters?: string[];
  reverse: Set<string>;
}

export interface GetGuestOrdersQuery {
  index: number;
  limit: number;
  fuzzy?: string;
  id?: number;
  created_by_user_id?: number;
  warehouse_ids?: Set<number>;
  person_related_id?: number;
  person_in_charge_id?: number;
  order_type?: OrderType;
  currency?: OrderCurrency;
  order_category_id?: number;
  date_start?: number;
  date_end?: number;
  sorters?: string[];
  reverse: Set<string>;
}

export interface GetOrderPaymentsQuery {
  index: number;
  limit: number;
  id?: number;
  order_id?: number;
  warehouse_ids?: Set<number>;
  person_in_charge_id?: number;
  creation_date_start?: number;
  creation_date_end?: number;
  actual_date_start?: number;
  actual_date_end?: number;
  sorters?: string[];
}

export interface CalcOrdersQuery {
  warehouse_id?: number;
}

export interface ItemNotAvailable {
  sku_id: number;
  require_quantity: number;
  actual_quantity: number;
}

export interface CheckOrderResult {
  items_not_available: ItemNotAvailable[];
}

export interface GetOrderCategoryQuery {
  index: number;
  limit: number;
  id?: number;
  name?: string;
  sorters?: string[];
}

export interface GetSKUCategoriesQuery {
  index: number;
  limit: number;
  id?: number;
  name?: string;
  sorters?: string[];
}

export interface GetSKUsQuery {
  index: number;
  limit: number;
  id?: number;
  name?: string;
  sku_category_id?: number;
  sorters?: string[];
}

export interface GetWarehousesQuery {
  index: number;
  limit: number;
  id?: number;
  name?: string;
  person_in_charge_id?: number;
  area_id?: number;
  sorters?: string[];
}
