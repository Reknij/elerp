use serde::Serialize;
use strum::{AsRefStr, Display, IntoStaticStr};

#[derive(Debug, Clone, AsRefStr, IntoStaticStr, Serialize, Display)]
pub enum WebSocketFlags {
    AddArea(i64),
    UpdateArea(i64),
    RemoveArea(i64),

    AddPerson(i64),
    UpdatePerson(i64),
    RemovePerson(i64),

    AddWarehouse(i64),
    UpdateWarehouse(i64),
    RemoveWarehouse(i64),
    LinkedWarehouse(i64),
    UnlinkedWarehouse(i64),
    
    AddSKUCategory(i64),
    UpdateSKUCategory(i64),
    RemoveSKUCategory(i64),

    AddSKU(i64),
    UpdateSKU(i64),
    RemoveSKU(i64),

    AddOrder(i64),
    UpdateOrder(i64),
    RemoveOrder(i64),
    AddGuestOrder(i64),
    RemoveGuestOrder(i64),
    ConfirmGuestOrder(i64),
    RecalcOrders,

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

    AddOrderCategory(i64),
    UpdateOrderCategory(i64),
    RemoveOrderCategory(i64),

    AddOrderPayment(i64),
    RemoveOrderPayment(i64),

    AddUser(i64),
    UpdateUser(i64),
    RemoveUser(i64),
    UserRepeatLogin(i64),
    LinkedUser(i64),
    UnlinkedUser(i64),

    UserConnected(i64),
    UserDisconnected(i64),
}
