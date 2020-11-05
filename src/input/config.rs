use super::{Action, Button};
use serde::*;
use serde::ser::{Serialize, Serializer};
use std::collections::{BTreeMap, HashMap};
use strum::IntoEnumIterator;
use winit::event::{MouseButton, VirtualKeyCode};

#[derive(Deserialize, Serialize, Copy, Clone)]
pub struct Binding {
    pub primary: Button,
    pub secondary: Button,
}

#[derive(Deserialize, Serialize)]
pub struct InputConfig {
    #[serde(serialize_with = "ordered_map")]
    pub bindings: HashMap<Action, Binding>,
}

/// Sort keys and serialize
fn ordered_map<S>(map: &HashMap<Action, Binding>, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    let ordered: BTreeMap<_, _> = map.iter().collect();
    ordered.serialize(serializer)
}


impl InputConfig {
    /// Default config
    pub fn default() -> Self {
        let mut bindings: HashMap<Action, Binding> = HashMap::new();

        for action in Action::iter() {
            bindings.insert(action, Binding {primary: Button::None, secondary: Button::None});
        }

        // Set default bindings here
        bindings.insert(
            Action::ActionButton1,
            Binding {
                primary: Button::Key(VirtualKeyCode::Key1),
                secondary: Button::None,
            },
        );
        bindings.insert(
            Action::ActionButton2,
            Binding {
                primary: Button::Key(VirtualKeyCode::Key2),
                secondary: Button::None,
            },
        );
        bindings.insert(
            Action::ActionButton3,
            Binding {
                primary: Button::Key(VirtualKeyCode::Key3),
                secondary: Button::None,
            },
        );
        bindings.insert(
            Action::ActionButton4,
            Binding {
                primary: Button::Key(VirtualKeyCode::Key4),
                secondary: Button::None,
            },
        );
        bindings.insert(
            Action::ActionButton5,
            Binding {
                primary: Button::Mouse(MouseButton::Other(1)),
                secondary: Button::None,
            },
        );
        bindings.insert(
            Action::ActionButton6,
            Binding {
                primary: Button::Mouse(MouseButton::Other(2)),
                secondary: Button::None,
            },
        );
        bindings.insert(
            Action::AutoRun,
            Binding {
                primary: Button::Key(VirtualKeyCode::Numlock),
                secondary: Button::None,
            },
        );
        bindings.insert(
            Action::Chat,
            Binding {
                primary: Button::Key(VirtualKeyCode::Return),
                secondary: Button::None,
            },
        );
        bindings.insert(
            Action::Jump,
            Binding {
                primary: Button::Key(VirtualKeyCode::Space),
                secondary: Button::None,
            },
        );
        bindings.insert(
            Action::TargetSelf,
            Binding {
                primary: Button::Key(VirtualKeyCode::F1),
                secondary: Button::None,
            },
        );
        bindings.insert(
            Action::TargetNearestEnemy,
            Binding {
                primary: Button::Key(VirtualKeyCode::Tab),
                secondary: Button::None,
            },
        );
        bindings.insert(
            Action::MoveBackward,
            Binding {
                primary: Button::Key(VirtualKeyCode::S),
                secondary: Button::Key(VirtualKeyCode::Down),
            },
        );
        bindings.insert(
            Action::MoveForward,
            Binding {
                primary: Button::Key(VirtualKeyCode::W),
                secondary: Button::Key(VirtualKeyCode::Up),
            },
        );
        bindings.insert(
            Action::MoveLeft,
            Binding {
                primary: Button::Key(VirtualKeyCode::A),
                secondary: Button::Key(VirtualKeyCode::Left),
            },
        );
        bindings.insert(
            Action::MoveRight,
            Binding {
                primary: Button::Key(VirtualKeyCode::D),
                secondary: Button::Key(VirtualKeyCode::Right),
            },
        );
        bindings.insert(
            Action::ToggleBackpack,
            Binding {
                primary: Button::Key(VirtualKeyCode::B),
                secondary: Button::None,
            },
        );
        bindings.insert(
            Action::ToggleCharacterPanel,
            Binding {
                primary: Button::Key(VirtualKeyCode::C),
                secondary: Button::None,
            },
        );
        bindings.insert(
            Action::ToggleInventory,
            Binding {
                primary: Button::Key(VirtualKeyCode::I),
                secondary: Button::None,
            },
        );
        bindings.insert(
            Action::ToggleMap,
            Binding {
                primary: Button::Key(VirtualKeyCode::M),
                secondary: Button::None,
            },
        );

        Self { bindings }
    }
}
