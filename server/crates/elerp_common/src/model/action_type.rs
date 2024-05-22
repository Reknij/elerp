
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ActionType {
    General(i64),
    GeneralAllowed(i64),
    Admin,
    System,
}