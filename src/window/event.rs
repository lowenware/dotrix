//! Input Abstractions

use bitflags::bitflags;

/// Key Scan Code
pub type ScanCode = u32;

/// Window input event
#[derive(Debug, Clone, PartialEq)]
pub enum Event {
    /// Button press event
    ButtonPress {
        /// Button info
        button: Button,
        /// Text input
        text: Option<String>,
    },
    /// Button release event
    ButtonRelease {
        /// Button info
        button: Button,
    },
    /// Cursor position change event
    CursorPosition {
        /// Horizontal offset
        horizontal: f64,
        /// Vertical offset
        vertical: f64,
    },
    /// Modifiers change event
    ModifiersChange {
        /// Modifiers state
        modifiers: Modifiers,
    },
    /// Mouse move event
    MouseMove {
        /// Horizontal offset
        horizontal: f64,
        /// Vertical offset
        vertical: f64,
    },
    /// Mouse scroll event
    MouseScroll {
        /// Scroll delta
        delta: MouseScroll,
    },
    /// Drag and drop event
    DragAndDrop {
        /// Drag'n'drop target
        target: DragAndDrop,
    },
}

/// Drag and drop event
#[derive(Debug, Clone, PartialEq)]
pub enum DragAndDrop {
    /// File dragged over the window
    FileDragged {
        /// Dragged file path
        path: std::path::PathBuf,
    },
    /// File dropped over the window
    FileDropped {
        /// Dragged file path
        path: std::path::PathBuf,
    },
    /// Dragging canceled
    Canceled,
}

/// Button press event
#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash)]
pub enum Button {
    /// Keyboard button
    Keyboard {
        /// Button key code
        key: KeyButton,
        /// Button scan code
        code: ScanCode,
    },
    /// Left mouse button
    MouseLeft,
    /// Right mouse button
    MouseRight,
    /// Middle mouse button
    MouseMiddle,
    /// Forward mouse button
    Forward,
    /// Back mouse button
    Back,
    /// Other mouse button represented by numeric code
    MouseOther(u16),
}

/// Mouse scroll event
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum MouseScroll {
    /// Scroll delta in pixels (usually for touchpads)
    Pixels {
        /// Horizontal offset
        horizontal: f64,
        /// Vertical offset
        vertical: f64,
    },
    /// Scroll delta in lines (usually for mouse)
    Lines {
        /// Horizontal offset
        horizontal: f64,
        /// Vertical offset
        vertical: f64,
    },
}

bitflags! {
    /// State of modifiers
    #[derive(Debug, Default, Clone, Copy, Eq, PartialEq)]
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
    /// Returns true if SHIFT modifier used
    pub fn shift(&self) -> bool {
        self.intersects(Self::SHIFT)
    }

    /// Returns true if CTRL modifier used
    pub fn ctrl(&self) -> bool {
        self.intersects(Self::CTRL)
    }

    /// Returns true if ALT modifier used
    pub fn alt(&self) -> bool {
        self.intersects(Self::ALT)
    }

    /// Returns true if SUPR modifier used
    pub fn supr(&self) -> bool {
        self.intersects(Self::SUPER)
    }

    /// Returns true if OPT modifier used (MacOS)
    pub fn opt(&self) -> bool {
        self.supr()
    }

    /// Returns true if CMD modifier used (MacOS)
    pub fn cmd(&self) -> bool {
        self.alt()
    }
}

/// Keyboard key codes
#[allow(missing_docs)]
#[derive(Debug, Hash, Ord, PartialOrd, PartialEq, Eq, Clone, Copy)]
#[repr(u32)]
pub enum KeyButton {
    Num1,
    Num2,
    Num3,
    Num4,
    Num5,
    Num6,
    Num7,
    Num8,
    Num9,
    Num0,

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

    PrintScreen,

    ScrollLock,

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
