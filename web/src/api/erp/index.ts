import { erp } from "..";
import { ListSlice } from "../models";
import { UserInfo } from "../user_system/models";
import {
  CheckOrderResult,
  GetInventoryQuery,
  GetOrdersQuery,
  GetSKUCategoriesQuery,
  GetSKUsQuery,
  GetWarehousesQuery,
  InventoryProduct,
  Order,
  SKU,
  SKUCategory,
  Warehouse,
  Area,
  GetAreasQuery,
  GetPersonsQuery,
  Person,
  CalcOrdersQuery,
  GetOrderCategoryQuery,
  OrderCategory,
  StatisticalData,
  GetStatisticalDataQuery,
  OrderPayment,
  GetOrderPaymentsQuery,
  WarehouseToLinkQuery,
  GetWarehouseLinkedUsersQuery,
  GuestOrder,
  GetGuestOrdersQuery,
  GuestOrderConfirm,
  ClearResult,
} from "./model";

export async function get_statistical_data(
  q: GetStatisticalDataQuery
): Promise<StatisticalData> {
  const resp = await erp.get("/statistical_data", {
    params: q,
  });
  return resp.data;
}

export async function clear_cache(): Promise<boolean> {
  const resp = await erp.post("/clear_cache");
  return resp.status == 200;
}

export async function add_area(v: Area): Promise<Area> {
  const resp = await erp.post("/areas", v);
  return resp.data;
}

export async function remove_area(id: number): Promise<boolean> {
  const resp = await erp.delete(`/areas/${id}`);
  return resp.status == 200;
}

export async function clear_areas(q: GetAreasQuery): Promise<ClearResult> {
  const resp = await erp.delete(`/areas`, {
    params: q,
  });
  return resp.data;
}

export async function get_area(id: number): Promise<Area> {
  const resp = await erp.get(`/areas/${id}`);
  return resp.data;
}

export async function get_areas(q: GetAreasQuery): Promise<ListSlice<Area>> {
  const resp = await erp.get(`/areas`, {
    params: q,
  });
  return resp.data;
}

export async function update_area(id: number, v: Area): Promise<Area> {
  const resp = await erp.put(`/areas/${id}`, v);
  return resp.data;
}

export async function add_person(v: Person): Promise<Person> {
  const resp = await erp.post("/persons", v);
  return resp.data;
}

export async function remove_person(id: number): Promise<boolean> {
  const resp = await erp.delete(`/persons/${id}`);
  return resp.status == 200;
}

export async function clear_persons(q: GetPersonsQuery): Promise<ClearResult> {
  const resp = await erp.delete(`/persons`, {
    params: q,
  });
  return resp.data;
}

export async function get_person(id: number): Promise<Person> {
  const resp = await erp.get(`/persons/${id}`);
  return resp.data;
}

export async function get_persons(
  q: GetPersonsQuery
): Promise<ListSlice<Person>> {
  const resp = await erp.get(`/persons`, {
    params: q,
  });
  return resp.data;
}

export async function update_person(id: number, v: Person): Promise<Person> {
  const resp = await erp.put(`/persons/${id}`, v);
  return resp.data;
}

export async function add_warehouse(v: Warehouse): Promise<Warehouse> {
  const resp = await erp.post("/warehouses", v);
  return resp.data;
}

export async function remove_warehouse(id: number): Promise<boolean> {
  const resp = await erp.delete(`/warehouses/${id}`);
  return resp.status == 200;
}

export async function clear_warehouses(
  q: GetWarehousesQuery
): Promise<ClearResult> {
  const resp = await erp.delete(`/warehouses`, {
    params: q,
  });
  return resp.data;
}

export async function get_warehouse(id: number): Promise<Warehouse> {
  const resp = await erp.get(`/warehouses/${id}`);
  return resp.data;
}

export async function get_warehouses(
  q: GetWarehousesQuery
): Promise<ListSlice<Warehouse>> {
  const resp = await erp.get(`/warehouses`, {
    params: q,
  });
  return resp.data;
}

export async function update_warehouse(
  id: number,
  v: Warehouse
): Promise<Warehouse> {
  const resp = await erp.put(`/warehouses/${id}`, v);
  return resp.data;
}

export async function link_warehouse(
  warehouse_id: number,
  q: WarehouseToLinkQuery
): Promise<Warehouse> {
  const resp = await erp.post(`/warehouse_link/${warehouse_id}`, undefined, {
    params: q,
  });
  return resp.data;
}

export async function unlink_warehouse(
  warehouse_id: number,
  body: WarehouseToLinkQuery
): Promise<boolean> {
  const resp = await erp.delete(`/warehouse_link/${warehouse_id}`, {
    params: body,
  });
  return resp.status == 200;
}

export async function get_linked_users(
  warehouse_id: number,
  q: GetWarehouseLinkedUsersQuery
): Promise<ListSlice<UserInfo>> {
  const resp = await erp.get(`/warehouse_link/${warehouse_id}`, {
    params: q,
  });
  return resp.data;
}

export async function add_sku(v: SKU): Promise<SKU> {
  const resp = await erp.post("/skus", v);
  return resp.data;
}

export async function remove_sku(id: number): Promise<boolean> {
  const resp = await erp.delete(`/skus/${id}`);
  return resp.status == 200;
}

export async function clear_skus(q: GetSKUsQuery): Promise<ClearResult> {
  const resp = await erp.delete(`/skus`, {
    params: q,
  });
  return resp.data;
}

export async function get_sku(id: number): Promise<SKU> {
  const urlParams = new URLSearchParams(window.location.search);
  const resp = await erp.get(`/skus/${id}`, {
    headers: {
      "X-Sub-Authorization": urlParams.get("sub_token"),
    },
  });
  return resp.data;
}

export async function get_skus(q: GetSKUsQuery): Promise<ListSlice<SKU>> {
  const urlParams = new URLSearchParams(window.location.search);
  const resp = await erp.get(`/skus`, {
    params: q,
    headers: {
      "X-Sub-Authorization": urlParams.get("sub_token"),
    },
  });
  return resp.data;
}

export async function update_sku(id: number, v: SKU): Promise<SKU> {
  const resp = await erp.put(`/skus/${id}`, v);
  return resp.data;
}

export async function add_sku_category(v: SKUCategory): Promise<SKUCategory> {
  const resp = await erp.post("/sku_categories", v);
  return resp.data;
}

export async function remove_sku_category(id: number): Promise<boolean> {
  const resp = await erp.delete(`/sku_categories/${id}`);
  return resp.status == 200;
}

export async function clear_sku_categories(
  q: GetSKUCategoriesQuery
): Promise<ClearResult> {
  const resp = await erp.delete(`/sku_categories`, {
    params: q,
  });
  return resp.data;
}

export async function get_sku_category(id: number): Promise<SKUCategory> {
  const urlParams = new URLSearchParams(window.location.search);
  const resp = await erp.get(`/sku_categories/${id}`, {
    headers: {
      "X-Sub-Authorization": urlParams.get("sub_token"),
    },
  });
  return resp.data;
}

export async function get_sku_categories(
  q: GetSKUCategoriesQuery
): Promise<ListSlice<SKUCategory>> {
  const urlParams = new URLSearchParams(window.location.search);
  const resp = await erp.get(`/sku_categories`, {
    params: q,
    headers: {
      "X-Sub-Authorization": urlParams.get("sub_token"),
    },
  });
  return resp.data;
}

export async function update_sku_category(
  id: number,
  v: SKUCategory
): Promise<SKUCategory> {
  const resp = await erp.put(`/sku_categories/${id}`, v);
  return resp.data;
}

export async function add_order(v: Order): Promise<Order> {
  const resp = await erp.post("/orders", v);
  return resp.data;
}

export async function remove_order(id: number): Promise<boolean> {
  const resp = await erp.delete(`/orders/${id}`);
  return resp.status == 200;
}

export async function clear_orders(q: GetOrdersQuery): Promise<ClearResult> {
  const resp = await erp.delete(`/orders`, {
    params: q,
  });
  return resp.data;
}

export async function get_order(id: number): Promise<Order> {
  const resp = await erp.get(`/orders/${id}`);
  return resp.data;
}

export async function get_orders(q: GetOrdersQuery): Promise<ListSlice<Order>> {
  const resp = await erp.get(`/orders`, {
    params: q,
  });
  return resp.data;
}

export async function update_order(id: number, v: Order): Promise<Order> {
  const resp = await erp.put(`/orders/${id}`, v);
  return resp.data;
}

export async function add_guest_order(v: GuestOrder): Promise<GuestOrder> {
  const resp = await erp.post("/guest_orders", v);
  return resp.data;
}

export async function remove_guest_order(id: number): Promise<boolean> {
  const resp = await erp.delete(`/guest_orders/${id}`);
  return resp.status == 200;
}

export async function clear_guest_orders(
  q: GetGuestOrdersQuery
): Promise<ClearResult> {
  const resp = await erp.delete(`/guest_orders`, {
    params: q,
  });
  return resp.data;
}

export async function get_guest_order(
  id: number,
  token?: string
): Promise<GuestOrder> {
  const resp = await erp.get(`/guest_orders/${id}`, {
    headers: {
      "X-Sub-Authorization": token,
    },
  });
  return resp.data;
}

export async function get_guest_orders(
  q: GetGuestOrdersQuery
): Promise<ListSlice<GuestOrder>> {
  const resp = await erp.get(`/guest_orders`, {
    params: q,
  });
  return resp.data;
}

export async function confirm_guest_order(
  id: number,
  v: GuestOrder
): Promise<GuestOrderConfirm> {
  const resp = await erp.put(`/guest_orders/${id}`, v, {
    headers: {
      "X-Sub-Authorization": v.sub_token,
    },
  });
  return resp.data;
}

export async function add_order_category(v: OrderCategory): Promise<OrderCategory> {
  const resp = await erp.post("/order_categories", v);
  return resp.data;
}

export async function remove_order_category(id: number): Promise<boolean> {
  const resp = await erp.delete(`/order_categories/${id}`);
  return resp.status == 200;
}

export async function clear_order_categories(
  q: GetOrderCategoryQuery
): Promise<ClearResult> {
  const resp = await erp.delete(`/order_categories`, {
    params: q,
  });
  return resp.data;
}

export async function get_order_category(id: number): Promise<OrderCategory> {
  const resp = await erp.get(`/order_categories/${id}`);
  return resp.data;
}

export async function get_order_categories(
  q: GetOrderCategoryQuery
): Promise<ListSlice<OrderCategory>> {
  const resp = await erp.get(`/order_categories`, {
    params: q,
  });
  return resp.data;
}

export async function update_order_category(
  id: number,
  v: OrderCategory
): Promise<OrderCategory> {
  const resp = await erp.put(`/order_categories/${id}`, v);
  return resp.data;
}

export async function add_order_payment(
  v: OrderPayment
): Promise<OrderPayment> {
  const resp = await erp.post("/order_payments", v);
  return resp.data;
}

export async function remove_order_payment(id: number): Promise<boolean> {
  const resp = await erp.delete(`/order_payments/${id}`);
  return resp.status == 200;
}

export async function clear_order_payments(
  q: GetOrderPaymentsQuery
): Promise<ClearResult> {
  const resp = await erp.delete(`/order_payments`, {
    params: q,
  });
  return resp.data;
}

export async function get_order_payment(id: number): Promise<OrderPayment> {
  const resp = await erp.get(`/order_payments/${id}`);
  return resp.data;
}

export async function get_order_payments(
  q: GetOrderPaymentsQuery
): Promise<ListSlice<OrderPayment>> {
  const resp = await erp.get(`/order_payments`, {
    params: q,
  });
  return resp.data;
}

export async function recalc_orders(q: CalcOrdersQuery) {
  await erp.post(`/recalc_orders`, {
    params: q,
  });
}

export async function check_order(v: Order): Promise<CheckOrderResult> {
  const resp = await erp.post(`/check_order`, v);
  return resp.data;
}

export async function get_inventory(
  q: GetInventoryQuery
): Promise<ListSlice<InventoryProduct>> {
  const resp = await erp.get("/inventory", {
    params: q,
  });
  return resp.data;
}

export async function download_inventory_excel(q: GetInventoryQuery) {
  const resp = await erp.get(`/inventory_excel`, {
    params: q,
    responseType: "blob",
  });
  const href = window.URL.createObjectURL(resp.data);

  const anchorElement = document.createElement("a");

  anchorElement.href = href;
  const now = new Date().toISOString().split(".")[0].replace(/[T:]/g, "-");
  let filename = `Inventory-${now}.xlsx`;
  anchorElement.download = filename;

  document.body.appendChild(anchorElement);
  anchorElement.click();

  document.body.removeChild(anchorElement);
  window.URL.revokeObjectURL(href);
  return resp.data;
}
