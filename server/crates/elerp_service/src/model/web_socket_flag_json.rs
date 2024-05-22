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
            WebSocketFlags::AddArea(id) => Some(id),
            WebSocketFlags::UpdateArea(id) => Some(id),
            WebSocketFlags::RemoveArea(id) => Some(id),
            WebSocketFlags::AddPerson(id) => Some(id),
            WebSocketFlags::UpdatePerson(id) => Some(id),
            WebSocketFlags::RemovePerson(id) => Some(id),
            WebSocketFlags::AddWarehouse(id) => Some(id),
            WebSocketFlags::UpdateWarehouse(id) => Some(id),
            WebSocketFlags::RemoveWarehouse(id) => Some(id),
            WebSocketFlags::LinkedWarehouse(id) => Some(id),
            WebSocketFlags::UnlinkedWarehouse(id) => Some(id),
            WebSocketFlags::AddSKUCategory(id) => Some(id),
            WebSocketFlags::UpdateSKUCategory(id) => Some(id),
            WebSocketFlags::RemoveSKUCategory(id) => Some(id),
            WebSocketFlags::AddSKU(id) => Some(id),
            WebSocketFlags::UpdateSKU(id) => Some(id),
            WebSocketFlags::RemoveSKU(id) => Some(id),
            WebSocketFlags::AddOrder(id) => Some(id),
            WebSocketFlags::UpdateOrder(id) => Some(id),
            WebSocketFlags::RemoveOrder(id) => Some(id),
            WebSocketFlags::AddGuestOrder(id) => Some(id),
            WebSocketFlags::ConfirmGuestOrder(id) => Some(id),
            WebSocketFlags::RemoveGuestOrder(id) => Some(id),
            WebSocketFlags::AddUser(id) => Some(id),
            WebSocketFlags::UpdateUser(id) => Some(id),
            WebSocketFlags::RemoveUser(id) => Some(id),
            WebSocketFlags::UserRepeatLogin(id) => Some(id),
            WebSocketFlags::LinkedUser(id) => Some(id),
            WebSocketFlags::UnlinkedUser(id) => Some(id),
            WebSocketFlags::AddOrderCategory(id) => Some(id),
            WebSocketFlags::RemoveOrderCategory(id) => Some(id),
            WebSocketFlags::UpdateOrderCategory(id) => Some(id),
            WebSocketFlags::AddOrderPayment(id) => Some(id),
            WebSocketFlags::RemoveOrderPayment(id) => Some(id),

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
        Self {
            flag: value.to_string(),
            id,
        }
    }
}
