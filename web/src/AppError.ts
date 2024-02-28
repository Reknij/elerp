import { i18n } from "./i18n";

export enum CustomErrorType {
  PersonNotFound = "PersonNotFound",
  AreaNotFound = "AreaNotFound",
  WarehouseNotFound = "WarehouseNotFound",
  SKUCategoryNotFound = "SKUCategoryNotFound",
  SKUNotFound = "SKUNotFound",
  OrderNotFound = "OrderNotFound",
  OrderStatusNotFound = "OrderStatusNotFound",
  OrderPaymentSettled = "OrderPaymentSettled",
  OrderPaymentNotFound = "OrderPaymentNotFound",
  OrderPaymentIsNone = "OrderPaymentIsNone",
  OrderItemsIsEmpty = "OrderItemsIsEmpty",
  UserNotFound = "UserNotFound",
  WrongPassword = "WrongPassword",
  NoPermission = "NoPermission",
  SameObject = "SameObject",
  CheckFailed = "CheckFailed",
  NotAllowed = "NotAllowed",
  TotalAmountUnexpected = "TotalAmountUnexpected",
  SomeoneDepentIt = "SomeoneDepentIt",
  GuestOrderConfirmed = "GuestOrderConfirmed",
  GuestOrderExpired = "GuestOrderExpired",
  FromGuestOrder = "FromGuestOrder",
  UserLimitExceeded = "UserLimitExceeded",
  WarehouseLimitExceeded = "WarehouseLimitExceeded",
  AreaLimitExceeded = "AreaLimitExceeded",
  PersonLimitExceeded = "PersonLimitExceeded",
  SKUCategoryLimitExceeded = "SKUCategoryLimitExceeded",
  SKULimitExceeded = "SKULimitExceeded",
  OrderLimitExceeded = "OrderLimitExceeded",
  OrderStatusLimitExceeded = "OrderStatusLimitExceeded",
  OrderPaymentLimitExceeded = "OrderPaymentLimitExceeded",
  GuestOrderLimitExceeded = "GuestOrderLimitExceeded",
}

export interface AppError {
  code: string;
  error_type: CustomErrorType;
  msg: string;
}

export function error_parse(error: any): AppError | undefined {
  if (error) {
    if (error.response?.data) {
      if (error.response.data.error_code && error.response.data.error_type) {
        return error.response.data as AppError;
      } else {
        return undefined;
      }
    } else {
      return undefined;
    }
  } else {
    return undefined;
  }
}

export function error_to_string(error: any): string {
  if (error) {
    if (error.response?.data) {
      if (error.response.data.error_code && error.response.data.error_type) {
        return error.response.data.msg;
      } else {
        return error.response.data;
      }
    } else {
      return error.response;
    }
  } else {
    return "Unknown error.";
  }
}

export function fmt_err(
  error: any,
  options: { obj: any } | undefined = undefined
): string | undefined {
  const { t } = i18n.global;
  const appError = error_parse(error);
  if (appError) {
    switch (appError.error_type) {
      case CustomErrorType.NoPermission:
        return t("error.noPermission");
      case CustomErrorType.PersonNotFound:
        return t("error.objectNotFound", {
          obj: t("main.person"),
        });
      case CustomErrorType.AreaNotFound:
        return t("error.objectNotFound", {
          obj: t("main.area"),
        });
      case CustomErrorType.WarehouseNotFound:
        return t("error.objectNotFound", {
          obj: t("main.warehouse"),
        });
      case CustomErrorType.SKUCategoryNotFound:
        return t("error.objectNotFound", {
          obj: t("main.SKUCategory"),
        });
      case CustomErrorType.SKUNotFound:
        return t("error.objectNotFound", {
          obj: t("main.SKU"),
        });
      case CustomErrorType.OrderNotFound:
        return t("error.objectNotFound", {
          obj: t("main.order"),
        });
      case CustomErrorType.OrderStatusNotFound:
        return t("error.objectNotFound", {
          obj: t("main.orderStatus"),
        });
      case CustomErrorType.OrderPaymentNotFound:
        return t("error.objectNotFound", {
          obj: t("main.orderPayment"),
        });
      case CustomErrorType.OrderPaymentSettled:
        return t("error.orderPaymentSettled");
      case CustomErrorType.OrderPaymentIsNone:
        return t("error.orderPaymentIsNone");
      case CustomErrorType.OrderItemsIsEmpty:
        return t("error.orderItemsIsEmpty");
      case CustomErrorType.NotAllowed:
        return t("error.notAllowed", {});
      case CustomErrorType.TotalAmountUnexpected:
        return t("error.totalAmountUnexpected");
      case CustomErrorType.SomeoneDepentIt:
        return t("error.someoneDepentIt");
      case CustomErrorType.UserNotFound:
        return t("error.objectNotFound", {
          obj: t("main.user"),
        });
      case CustomErrorType.SameObject:
        return t("error.sameObject", {
          obj: options?.obj,
        });
      case CustomErrorType.WrongPassword:
        return t("error.wrongPassword");
      case CustomErrorType.CheckFailed:
        return t("error.checkFailed");
      case CustomErrorType.UserLimitExceeded:
        return t("error.limitExceeded", { obj: t("main.user") });
      case CustomErrorType.AreaLimitExceeded:
        return t("error.limitExceeded", { obj: t("main.area") });
      case CustomErrorType.PersonLimitExceeded:
        return t("error.limitExceeded", { obj: t("main.person") });
      case CustomErrorType.WarehouseLimitExceeded:
        return t("error.limitExceeded", { obj: t("main.warehouse") });
      case CustomErrorType.SKULimitExceeded:
        return t("error.limitExceeded", { obj: t("main.SKU") });
      case CustomErrorType.SKUCategoryLimitExceeded:
        return t("error.limitExceeded", { obj: t("main.SKUCategory") });
      case CustomErrorType.OrderLimitExceeded:
        return t("error.limitExceeded", { obj: t("main.order") });
      case CustomErrorType.GuestOrderLimitExceeded:
        return t("error.limitExceeded", { obj: t("main.guestOrder") });
      case CustomErrorType.OrderPaymentLimitExceeded:
        return t("error.limitExceeded", { obj: t("main.payment") });
      case CustomErrorType.OrderStatusLimitExceeded:
        return t("error.limitExceeded", { obj: t("main.orderStatus") });
      case CustomErrorType.FromGuestOrder:
        return t("error.fromGuestOrder");
      case CustomErrorType.GuestOrderConfirmed:
        return t("error.guestOrderConfirmed");
      case CustomErrorType.GuestOrderExpired:
        return t("error.guestOrderExpired");
      default:
        break;
    }
  }
  return;
}
