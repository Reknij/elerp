import { defineStore } from "pinia";
import { ref } from "vue";
import {
  Area,
  Person,
  Order,
  GuestOrder,
  SKU,
  SKUCategory,
  Warehouse,
  OrderPayment,
  OrderCategory,
} from "../api/erp/model";
import {
  get_area,
  get_order_category,
  get_person,
} from "../api/erp";
import { UserInfo } from "../api/user_system/models";
import {
  get_order,
  get_guest_order,
  get_sku,
  get_sku_category,
  get_warehouse,
} from "../api/erp";
import { get_user } from "../api/user_system";
import { useMySelfUser } from "./me";
import { WebSocketFlag } from "../api/ws/models";
import { useRouter } from "vue-router";
import { useMessage } from "naive-ui";
import { useI18n } from "vue-i18n";

export const useCached = defineStore("cached", () => {
  const router = useRouter();
  const areas = ref<Map<number, Area>>(new Map());
  const persons = ref<Map<number, Person>>(new Map());
  const warehouses = ref<Map<number, Warehouse>>(new Map());
  const skus = ref<Map<number, SKU>>(new Map());
  const sku_categories = ref<Map<number, SKUCategory>>(new Map());
  const orders = ref<Map<number, Order>>(new Map());
  const guest_orders = ref<Map<number, GuestOrder>>(new Map());
  const order_categories = ref<Map<number, OrderCategory>>(new Map());
  const order_payment_list = ref<Map<number, OrderPayment>>(new Map());
  const users = ref<Map<number, UserInfo>>(new Map());
  const myself = useMySelfUser();
  const message = useMessage();
  const { t } = useI18n();
  myself.subscribe((flag) => {
    if (
      flag.isFlag(WebSocketFlag.RemoveArea) ||
      flag.isFlag(WebSocketFlag.UpdateArea)
    ) {
      areas.value.delete(flag.id!);
    } else if (
      flag.isFlag(WebSocketFlag.RemovePerson) ||
      flag.isFlag(WebSocketFlag.UpdatePerson)
    ) {
      persons.value.delete(flag.id!);
    } else if (
      flag.isFlag(WebSocketFlag.RemoveWarehouse) ||
      flag.isFlag(WebSocketFlag.UpdateWarehouse)
    ) {
      warehouses.value.delete(flag.id!);
    } else if (
      flag.isFlag(WebSocketFlag.RemoveSKU) ||
      flag.isFlag(WebSocketFlag.UpdateSKU)
    ) {
      skus.value.delete(flag.id!);
    } else if (
      flag.isFlag(WebSocketFlag.RemoveSKUCategory) ||
      flag.isFlag(WebSocketFlag.UpdateSKUCategory)
    ) {
      sku_categories.value.delete(flag.id!);
    } else if (
      flag.isFlag(WebSocketFlag.UpdateOrder) ||
      flag.isFlag(WebSocketFlag.RemoveOrder)
    ) {
      orders.value.delete(flag.id!);
    } else if (
      flag.isFlag(WebSocketFlag.ConfirmGuestOrder) ||
      flag.isFlag(WebSocketFlag.RemoveGuestOrder)
    ) {
      guest_orders.value.delete(flag.id!);
    } else if (flag.isFlag(WebSocketFlag.RecalcOrders)) {
      orders.value.clear();
    } else if (
      flag.isFlag(WebSocketFlag.UpdateOrderCategory) ||
      flag.isFlag(WebSocketFlag.RemoveOrderCategory)
    ) {
      order_categories.value.delete(flag.id!);
    } else if (
      flag.isFlag(WebSocketFlag.AddOrderPayment) ||
      flag.isFlag(WebSocketFlag.RemoveOrderPayment)
    ) {
      order_payment_list.value.delete(flag.id!);
    } else if (
      flag.isFlag(WebSocketFlag.RemoveUser) ||
      flag.isFlag(WebSocketFlag.UpdateUser)
    ) {
      users.value.delete(flag.id!);
      if (myself.authenticated?.user.id === flag.id) {
        myself.clearAuthorization();
        router.replace("/login");
        if (flag.isFlag(WebSocketFlag.RemoveUser)) {
          message.error(t("message.myselfRemoved"));
        } else if (flag.isFlag(WebSocketFlag.UpdateUser)) {
          message.warning(t("message.myselfUpdated"));
        }
      }
    }
  });
  async function getArea(id: number) {
    const v = areas.value.get(id);
    if (v) {
      return v;
    } else {
      let r = await get_area(id);
      areas.value.set(id, r);
      return r;
    }
  }
  async function getPerson(id: number) {
    const v = persons.value.get(id);
    if (v) {
      return v;
    } else {
      let r = await get_person(id);
      persons.value.set(id, r);
      return r;
    }
  }
  async function getWarehouse(id: number) {
    const v = warehouses.value.get(id);
    if (v) {
      return v;
    } else {
      let r = await get_warehouse(id);
      warehouses.value.set(id, r);
      return r;
    }
  }
  async function getSKU(id: number) {
    const v = skus.value.get(id);
    if (v) {
      return v;
    } else {
      let r = await get_sku(id);
      skus.value.set(id, r);
      return r;
    }
  }
  async function getSKUCategory(id: number) {
    const v = sku_categories.value.get(id);
    if (v) {
      return v;
    } else {
      let r = await get_sku_category(id);
      sku_categories.value.set(id, r);
      return r;
    }
  }
  async function getOrder(id: number) {
    const v = orders.value.get(id);
    if (v) {
      return v;
    } else {
      let r = await get_order(id);
      orders.value.set(id, r);
      return r;
    }
  }
  async function getGuestOrder(id: number, token?: string) {
    const v = guest_orders.value.get(id);
    if (v) {
      return v;
    } else {
      let r = await get_guest_order(id, token);
      guest_orders.value.set(id, r);
      return r;
    }
  }
  async function getOrderCategory(id: number) {
    const v = order_categories.value.get(id);
    if (v) {
      return v;
    } else {
      let r = await get_order_category(id);
      order_categories.value.set(id, r);
      return r;
    }
  }

  // async function getOrderPayment(id: number) {
  //   const v = order_payment_list.value.get(id);
  //   if (v) {
  //     return v;
  //   } else {
  //     let r = await get_order_payment(id);
  //     order_payment_list.value.set(id, r);
  //     return r;
  //   }
  // }
  async function getUser(id: number) {
    const v = users.value.get(id);
    if (v) {
      return v;
    } else {
      let r = await get_user(id);
      users.value.set(id, r);
      return r;
    }
  }
  return {
    getArea,
    getPerson,
    getWarehouse,
    getSKU,
    getSKUCategory,
    getOrder,
    getGuestOrder,
    getOrderCategory,
    getUser,
  };
});
