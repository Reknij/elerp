import { OrderItem } from "../../api/erp/model";

export enum FormRowType {
    ID = 'id',
    Text = 'text',
    TextArea = 'textarea',
    TextColor = 'textcolor',
    TextAreaColor = 'textarea_color',
    Date = 'date',
    DatePicker = "date_picker",
    Number = 'number',
    User = 'user',
    Area = 'area',
    Person = 'person',
    Warehouse = 'warehouse',
    SKUCategory = 'sku_category',
    SKU = 'sku',
    Order = 'order',
    OrderType = 'order_type',
    OrderItems = 'order_items',
    OrderCategory = "order_category",
    OrderPaymentStatus = "order_payment_status",
    GuestOrderStatus = "guest_order_status",
    UserType = "user_type",
    UserPermission = 'permission',
    OrderCurrency = 'order_currency',
    TotalAmount = "total_amount",
    TotalAmountInput = "total_amount_input",
    WarehouseLinkedUsers = "warehouse_linked_users",
    FromGuestOrder = "from_guest_order",
}

export interface FormRow {
    key: string,
    type: FormRowType,
    disabled?: boolean,
    onlyModal?: boolean,
    onlyTable?: boolean,
    noUpdate?: boolean,
    initSelf?: boolean,
    sorter?: 'ascend' | 'descend' | false,
    query?: any,
    opt?: any,
    visibleIf?: (arg0: any) => boolean,
}

export enum ModalType {
    Add,
    Update,
    Read,
}

export interface FilterItem {
    target: OrderItem;
    index: number;
  }