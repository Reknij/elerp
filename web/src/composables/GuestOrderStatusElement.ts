import { NTag } from "naive-ui";
import { h } from "vue";
import { GuestOrderStatus } from "../api/erp/model";
import { TagColor } from "naive-ui/es/tag/src/common-props";
import { getGuestOrderStatusText } from "../util";

export const getGuestOrderStatusElement = (status: GuestOrderStatus) => {
  let order_type_tag: "default" | "success" | "error" | "warning" | "info" =
    "success";
  let bordered = false;
  let color: TagColor | undefined = undefined;
  if (status == GuestOrderStatus.Expired) {
    order_type_tag = "warning";
  } else if (status == GuestOrderStatus.Pending) {
    order_type_tag = "info";
  } else if (status == GuestOrderStatus.Confirmed) {
    order_type_tag = "success";
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
    () => getGuestOrderStatusText(status)
  );
};
