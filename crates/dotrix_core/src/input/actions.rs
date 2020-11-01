use serde::{Deserialize, Serialize};
use strum_macros::*;

#[derive(Debug, PartialEq, Eq, Hash, Copy, Clone, EnumIter, Serialize, Deserialize, Ord, PartialOrd,)]
/// All bindable actions
pub enum Action {
    ActionButton1,
    ActionButton2,
    ActionButton3,
    ActionButton4,
    ActionButton5,
    ActionButton6,
    ActionButton7,
    ActionButton8,
    ActionButton9,
    AutoRun,
    Chat,
    Jump,
    MoveBackward,
    MoveForward,
    MoveLeft,
    MoveRight,
    Sit,
    TargetNearestEnemy,
    TargetSelf,
    ToggleBackpack,
    ToggleCharacterPanel,
    ToggleGameMenu,
    ToggleInventory,
    ToggleMap,
    ToggleProfessionBook,
    ToggleQuestLog,
    ToggleSpellbook,
    ToggleTalentPane,
    ZoomIn,
    ZoomOut,
}
