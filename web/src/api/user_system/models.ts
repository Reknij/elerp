import { OrderCurrency, OrderType } from "../erp/model";

export enum UserType {
  Admin = "Admin",
  General = "General",
}

export interface UserInfo {
  id: number;
  alias: string;
  username: string;
  password: string;
  user_type: UserType;
  permission: UserPermission;
  is_connected: boolean;
}

export interface UserConfigureDefaults {
  order_type: OrderType;
  order_category_id: number;
  warehouse_id: number;
  person_related_id: number;
  order_currency: OrderCurrency;
}

export interface UserConfigure {
  user_id: number;
  language: string;
  defaults: UserConfigureDefaults;
}

export interface AuthenticatedUser {
  user: UserInfo;
  token: string;
}

export interface GetUsersQuery {
  alias?: string;
  username?: string;
  user_type?: UserType;
  index: number;
  limit: number;
  sorters?: string[];
}

export interface GetTokenQuery {
  username: string;
  password: string;
}

export enum UserPermission {
  EMPTY = 0,
  MANAGE_AREA = 1,
  MANAGE_PERSON = 2,
  MANAGE_WAREHOUSE = 4,
  MANAGE_SKU = 8,
  MANAGE_SKU_CATEGORY = 16,
  ADD_ORDER = 32,
  UPDATE_REMOVE_ORDER = 64,
  MANAGE_ORDER_CATEGORY = 128,
  ADD_ORDER_PAYMENT = 256,
  UPDATE_REMOVE_ORDER_PAYMENT = 512,
}
