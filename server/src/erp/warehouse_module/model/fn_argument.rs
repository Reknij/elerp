use crate::user_system::models::user_info::UserInfo;

pub enum UserInfoID<'a> {
    InfoRef(&'a UserInfo),
    ID(i64),
}

pub enum WarehouseIsFrom {
    ID(i64),
    Order(i64),
    OrderPayment(i64),
    GuestOrder(i64),
}

impl From<i64> for UserInfoID<'static> {
    fn from(value: i64) -> Self {
        UserInfoID::ID(value)
    }
}

impl<'a> From<&'a UserInfo> for UserInfoID<'a> {
    fn from(value: &'a UserInfo) -> Self {
        UserInfoID::InfoRef(value)
    }
}