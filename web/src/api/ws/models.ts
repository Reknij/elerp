export enum WebSocketFlag {
  AddArea,
  UpdateArea,
  RemoveArea,

  AddPerson,
  UpdatePerson,
  RemovePerson,

  AddWarehouse,
  UpdateWarehouse,
  RemoveWarehouse,
  LinkedWarehouse,
  UnlinkedWarehouse,

  AddSKUCategory,
  UpdateSKUCategory,
  RemoveSKUCategory,

  AddSKU,
  UpdateSKU,
  RemoveSKU,

  AddOrder,
  UpdateOrder,
  RemoveOrder,
  AddGuestOrder,
  ConfirmGuestOrder,
  RemoveGuestOrder,
  RecalcOrders,

  AddOrderCategory,
  UpdateOrderCategory,
  RemoveOrderCategory,

  AddOrderPayment,
  RemoveOrderPayment,

  AddUser,
  UpdateUser,
  RemoveUser,
  UserRepeatLogin,
  LinkedUser,
  UnlinkedUser,

  ReadyAccess,
  Ping,
  ClearAreas,
  ClearPersons,
  ClearWarehouses,
  ClearSKUs,
  ClearSKUCategories,
  ClearOrders,
  ClearOrderCategories,
  ClearOrderPayments,
}

export class WebSocketFlagJson {
  flag?: String;
  id?: number;
  constructor(obj: any) {
    this.flag = obj.flag;
    this.id = obj.id;
  }
  isFlag(f: WebSocketFlag) {
    return this.flag && this.flag == WebSocketFlag[f];
  }
}
