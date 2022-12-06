//! Input Abstractions

use bitflags::bitflags;

pub type ScanCode = u32;

/// Input event abstraction
#[derive(Debug, Clone, PartialEq)]
pub enum Event {
    ButtonPress { button: Button },
    ButtonRelease { button: Button },
    CharacterInput { character: char },
    CursorPosition { horizontal: f64, vertical: f64 },
    ModifiersChange { modifiers: Modifiers },
    MouseMove { horizontal: f64, vertical: f64 },
    MouseScroll { delta: MouseScroll },
    DropFileHover { path: std::path::PathBuf },
    DropFileCancel,
    DropFile { path: std::path::PathBuf },
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash)]
pub enum Button {
    Key {
        key_code: Option<KeyCode>,
        scan_code: ScanCode,
    },
    MouseLeft,
    MouseRight,
    MouseMiddle,
    MouseOther(u16),
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum MouseScroll {
    Pixels { horizontal: f64, vertical: f64 },
    Lines { horizontal: f64, vertical: f64 },
}

bitflags! {
    /// State of modifiers
    #[derive(Default)]
    pub struct Modifiers: u32 {
        /// Shift modifier
        const SHIFT = 0b100;
        /// Control modifier
        const CTRL = 0b100 << 3;
        /// Alt modifier
        const ALT = 0b100 << 6;
        /// Super key modifier
        const SUPER = 0b100 << 9;
    }
}

impl Modifiers {
    pub fn shift(&self) -> bool {
        self.intersects(Self::SHIFT)
    }

    pub fn ctrl(&self) -> bool {
        self.intersects(Self::CTRL)
    }

    pub fn alt(&self) -> bool {
        self.intersects(Self::ALT)
    }

    pub fn supr(&self) -> bool {
        self.intersects(Self::SUPER)
    }

    pub fn opt(&self) -> bool {
        self.supr()
    }

    pub fn cmd(&self) -> bool {
        self.alt()
    }
}

#[derive(Debug, Hash, Ord, PartialOrd, PartialEq, Eq, Clone, Copy)]
#[repr(u32)]
pub enum KeyCode {
    Key1,
    Key2,
    Key3,
    Key4,
    Key5,
    Key6,
    Key7,
    Key8,
    Key9,
    Key0,

    A,
    B,
    C,
    D,
    E,
    F,
    G,
    H,
    I,
    J,
    K,
    L,
    M,
    N,
    O,
    P,
    Q,
    R,
    S,
    T,
    U,
    V,
    W,
    X,
    Y,
    Z,

    Escape,

    F1,
    F2,
    F3,
    F4,
    F5,
    F6,
    F7,
    F8,
    F9,
    F10,
    F11,
    F12,
    F13,
    F14,
    F15,
    F16,
    F17,
    F18,
    F19,
    F20,
    F21,
    F22,
    F23,
    F24,

    Snapshot,

    Scroll,

    Pause,

    Insert,
    Home,
    Delete,
    End,
    PageDown,
    PageUp,

    Left,
    Up,
    Right,
    Down,

    Backspace,
    Return,
    Space,

    Compose,

    Caret,

    Numlock,
    Numpad0,
    Numpad1,
    Numpad2,
    Numpad3,
    Numpad4,
    Numpad5,
    Numpad6,
    Numpad7,
    Numpad8,
    Numpad9,
    NumpadAdd,
    NumpadDivide,
    NumpadDecimal,
    NumpadComma,
    NumpadEnter,
    NumpadEquals,
    NumpadMultiply,
    NumpadSubtract,

    AbntC1,
    AbntC2,
    Apostrophe,
    Apps,
    Asterisk,
    At,
    Ax,
    Backslash,
    Calculator,
    Capital,
    Colon,
    Comma,
    Convert,
    Equals,
    Grave,
    Kana,
    Kanji,
    LAlt,
    LBracket,
    LControl,
    LShift,
    LWin,
    Mail,
    MediaSelect,
    MediaStop,
    Minus,
    Mute,
    MyComputer,

    NavigateForward,

    NavigateBackward,
    NextTrack,
    NoConvert,
    OEM102,
    Period,
    PlayPause,
    Plus,
    Power,
    PrevTrack,
    RAlt,
    RBracket,
    RControl,
    RShift,
    RWin,
    Semicolon,
    Slash,
    Sleep,
    Stop,
    Sysrq,
    Tab,
    Underline,
    Unlabeled,
    VolumeDown,
    VolumeUp,
    Wake,
    WebBack,
    WebFavorites,
    WebForward,
    WebHome,
    WebRefresh,
    WebSearch,
    WebStop,
    Yen,
    Copy,
    Paste,
    Cut,

    Unknown,
}
