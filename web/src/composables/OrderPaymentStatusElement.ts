import { NTag } from "naive-ui";
import { h } from "vue";
import { OrderPaymentStatus } from "../api/erp/model";
import { getOrderPaymentStatusText } from "../util";
import { TagColor } from "naive-ui/es/tag/src/common-props";

export const getOrderPaymentStatusElement = (ops: OrderPaymentStatus) => {
  let order_type_tag: "default" | "success" | "error" | "warning" | "info" =
    "success";
  let bordered = false;
  let color: TagColor | undefined = undefined;
  if (ops == OrderPaymentStatus.Settled) {
    order_type_tag = "success";
  } else if (ops == OrderPaymentStatus.Unsettled) {
    order_type_tag = "error";
  } else if (ops == OrderPaymentStatus.PartialSettled) {
    order_type_tag = "warning";
  } else if (ops == OrderPaymentStatus.None) {
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
    () => getOrderPaymentStatusText(ops)
  );
};
