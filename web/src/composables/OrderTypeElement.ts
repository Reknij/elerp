import { NTag } from "naive-ui";
import { h } from "vue";
import { OrderType } from "../api/erp/model";
import { getOrderTypeText } from "../util";
import { TagColor } from "naive-ui/es/tag/src/common-props";

export const getOrderTypeElement = (orderType: OrderType) => {
  let order_type_tag: "default" | "success" | "error" | "warning" | "info" =
    "success";
  let bordered = false;
  let color: TagColor | undefined = undefined;
  if (orderType == OrderType.StockOut) {
    order_type_tag = "error";
  } else if (orderType == OrderType.Exchange) {
    order_type_tag = "warning";
  } else if (orderType == OrderType.Return) {
    order_type_tag = "info";
  } else if (orderType == OrderType.Calibration || orderType == OrderType.CalibrationStrict) {
    color = {
      color: "#f6dbff",
      textColor: "#6a0094",
    };
    order_type_tag = "default";
  } else if (
    orderType == OrderType.Verification ||
    orderType == OrderType.VerificationStrict
  ) {
    order_type_tag = "default";
  }
  return h(
    NTag,
    {
      type: order_type_tag,
      bordered,
      style: {
        margin: "3px",
      },
      color,
    },
    () => getOrderTypeText(orderType)
  );
};
