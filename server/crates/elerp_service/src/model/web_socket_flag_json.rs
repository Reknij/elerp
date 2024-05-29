use elerp_common::model::WebSocketFlags;
use serde::Serialize;

#[derive(Debug, Serialize)]
pub struct WebSocketFlagsJson {
    flag: String,
    id: Option<i64>,
}

impl From<WebSocketFlags> for WebSocketFlagsJson {
    fn from(value: WebSocketFlags) -> Self {
        let id = match value {
            WebSocketFlags::AddArea(id)
            | WebSocketFlags::UpdateArea(id)
            | WebSocketFlags::RemoveArea(id)
            | WebSocketFlags::AddPerson(id)
            | WebSocketFlags::UpdatePerson(id)
            | WebSocketFlags::RemovePerson(id)
            | WebSocketFlags::AddWarehouse(id)
            | WebSocketFlags::UpdateWarehouse(id)
            | WebSocketFlags::RemoveWarehouse(id)
            | WebSocketFlags::LinkedWarehouse(id)
            | WebSocketFlags::UnlinkedWarehouse(id)
            | WebSocketFlags::AddSKUCategory(id)
            | WebSocketFlags::UpdateSKUCategory(id)
            | WebSocketFlags::RemoveSKUCategory(id)
            | WebSocketFlags::AddSKU(id)
            | WebSocketFlags::UpdateSKU(id)
            | WebSocketFlags::RemoveSKU(id)
            | WebSocketFlags::AddOrder(id)
            | WebSocketFlags::UpdateOrder(id)
            | WebSocketFlags::RemoveOrder(id)
            | WebSocketFlags::AddGuestOrder(id)
            | WebSocketFlags::ConfirmGuestOrder(id)
            | WebSocketFlags::RemoveGuestOrder(id)
            | WebSocketFlags::AddUser(id)
            | WebSocketFlags::UpdateUser(id)
            | WebSocketFlags::RemoveUser(id)
            | WebSocketFlags::UserRepeatLogin(id)
            | WebSocketFlags::LinkedUser(id)
            | WebSocketFlags::UnlinkedUser(id)
            | WebSocketFlags::AddOrderCategory(id)
            | WebSocketFlags::RemoveOrderCategory(id)
            | WebSocketFlags::UpdateOrderCategory(id)
            | WebSocketFlags::AddOrderPayment(id)
            | WebSocketFlags::RemoveOrderPayment(id)
            | WebSocketFlags::UserConnected(id)
            | WebSocketFlags::UserDisconnected(id) => Some(id),

            WebSocketFlags::RecalcOrders
            | WebSocketFlags::ReadyAccess
            | WebSocketFlags::Ping
            | WebSocketFlags::ClearAreas
            | WebSocketFlags::ClearOrderPayments
            | WebSocketFlags::ClearOrders
            | WebSocketFlags::ClearPersons
            | WebSocketFlags::ClearSKUCategories
            | WebSocketFlags::ClearSKUs
            | WebSocketFlags::ClearWarehouses
            | WebSocketFlags::ClearOrderCategories => None,
        };
        Self { flag: value.to_string(), id }
    }
}
