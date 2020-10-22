use strum_macros::*;

#[derive(Debug, PartialEq, Eq, Hash, Copy, Clone, EnumIter)]
/// All mapable actions
pub enum Action {
    Ability1,
    Ability2,
    Jump,
    MoveBackward,
    MoveForward,
    MoveLeft,
    MoveRight,
    Shoot,
}