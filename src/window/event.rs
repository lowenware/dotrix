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
        horizontal: f32,
        /// Vertical offset
        vertical: f32,
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
pub enum Button {
    KeyboardBackquote,
    KeyboardBackslash,
    KeyboardBracketLeft,
    KeyboardBracketRight,
    KeyboardComma,
    Keyboard0,
    Keyboard1,
    Keyboard2,
    Keyboard3,
    Keyboard4,
    Keyboard5,
    Keyboard6,
    Keyboard7,
    Keyboard8,
    Keyboard9,
    KeyboardEqual,
    KeyboardIntlBackslash,
    KeyboardIntlRo,
    KeyboardIntlYen,
    KeyboardA,
    KeyboardB,
    KeyboardC,
    KeyboardD,
    KeyboardE,
    KeyboardF,
    KeyboardG,
    KeyboardH,
    KeyboardI,
    KeyboardJ,
    KeyboardK,
    KeyboardL,
    KeyboardM,
    KeyboardN,
    KeyboardO,
    KeyboardP,
    KeyboardQ,
    KeyboardR,
    KeyboardS,
    KeyboardT,
    KeyboardU,
    KeyboardV,
    KeyboardW,
    KeyboardX,
    KeyboardY,
    KeyboardZ,
    KeyboardMinus,
    KeyboardPeriod,
    KeyboardQuote,
    KeyboardSemicolon,
    KeyboardSlash,
    KeyboardAltLeft,
    KeyboardAltRight,
    KeyboardBackspace,
    KeyboardCapsLock,
    KeyboardContextMenu,
    KeyboardControlLeft,
    KeyboardControlRight,
    KeyboardEnter,
    KeyboardSuperLeft,
    KeyboardSuperRight,
    KeyboardShiftLeft,
    KeyboardShiftRight,
    KeyboardSpace,
    KeyboardTab,
    KeyboardConvert,
    KeyboardKanaMode,
    KeyboardLang1,
    KeyboardLang2,
    KeyboardLang3,
    KeyboardLang4,
    KeyboardLang5,
    KeyboardNonConvert,
    KeyboardDelete,
    KeyboardEnd,
    KeyboardHelp,
    KeyboardHome,
    KeyboardInsert,
    KeyboardPageDown,
    KeyboardPageUp,
    KeyboardArrowDown,
    KeyboardArrowLeft,
    KeyboardArrowRight,
    KeyboardArrowUp,
    KeyboardNumLock,
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
    NumpadBackspace,
    NumpadClear,
    NumpadClearEntry,
    NumpadComma,
    NumpadDecimal,
    NumpadDivide,
    NumpadEnter,
    NumpadEqual,
    NumpadHash,
    NumpadMemoryAdd,
    NumpadMemoryClear,
    NumpadMemoryRecall,
    NumpadMemoryStore,
    NumpadMemorySubtract,
    NumpadMultiply,
    NumpadParenLeft,
    NumpadParenRight,
    NumpadStar,
    NumpadSubtract,
    KeyboardEscape,
    KeyboardFn,
    KeyboardFnLock,
    KeyboardPrintScreen,
    KeyboardScrollLock,
    KeyboardPause,
    KeyboardBrowserBack,
    KeyboardBrowserFavorites,
    KeyboardBrowserForward,
    KeyboardBrowserHome,
    KeyboardBrowserRefresh,
    KeyboardBrowserSearch,
    KeyboardBrowserStop,
    KeyboardEject,
    KeyboardLaunchApp1,
    KeyboardLaunchApp2,
    KeyboardLaunchMail,
    KeyboardMediaPlayPause,
    KeyboardMediaSelect,
    KeyboardMediaStop,
    KeyboardMediaTrackNext,
    KeyboardMediaTrackPrevious,
    KeyboardPower,
    KeyboardSleep,
    KeyboardAudioVolumeDown,
    KeyboardAudioVolumeMute,
    KeyboardAudioVolumeUp,
    KeyboardWakeUp,
    KeyboardMeta,
    KeyboardHyper,
    KeyboardTurbo,
    KeyboardAbort,
    KeyboardResume,
    KeyboardSuspend,
    KeyboardAgain,
    KeyboardCopy,
    KeyboardCut,
    KeyboardFind,
    KeyboardOpen,
    KeyboardPaste,
    KeyboardProps,
    KeyboardSelect,
    KeyboardUndo,
    KeyboardHiragana,
    KeyboardKatakana,
    KeyboardF1,
    KeyboardF2,
    KeyboardF3,
    KeyboardF4,
    KeyboardF5,
    KeyboardF6,
    KeyboardF7,
    KeyboardF8,
    KeyboardF9,
    KeyboardF10,
    KeyboardF11,
    KeyboardF12,
    KeyboardF13,
    KeyboardF14,
    KeyboardF15,
    KeyboardF16,
    KeyboardF17,
    KeyboardF18,
    KeyboardF19,
    KeyboardF20,
    KeyboardF21,
    KeyboardF22,
    KeyboardF23,
    KeyboardF24,
    KeyboardF25,
    KeyboardF26,
    KeyboardF27,
    KeyboardF28,
    KeyboardF29,
    KeyboardF30,
    KeyboardF31,
    KeyboardF32,
    KeyboardF33,
    KeyboardF34,
    KeyboardF35,

    KeyboardMacOS(u16),
    KeyboardAndroid(u32),
    KeyboardWindows(u16),
    KeyboardXkb(u32),

    /// Left mouse button
    MouseLeft,
    /// Right mouse button
    MouseRight,
    /// Middle mouse button
    MouseMiddle,
    /// Forward mouse button
    MouseForward,
    /// Back mouse button
    MouseBack,

    MouseOther(u16),

    Unknown,
}

impl From<&winit::event::KeyEvent> for Button {
    fn from(event: &winit::event::KeyEvent) -> Self {
        match event.physical_key {
            winit::keyboard::PhysicalKey::Code(code) => match code {
                winit::keyboard::KeyCode::Backquote => Button::KeyboardBackquote,
                winit::keyboard::KeyCode::Backslash => Button::KeyboardBackslash,
                winit::keyboard::KeyCode::BracketLeft => Button::KeyboardBracketLeft,
                winit::keyboard::KeyCode::BracketRight => Button::KeyboardBracketRight,
                winit::keyboard::KeyCode::Comma => Button::KeyboardComma,
                winit::keyboard::KeyCode::Digit0 => Button::Keyboard0,
                winit::keyboard::KeyCode::Digit1 => Button::Keyboard1,
                winit::keyboard::KeyCode::Digit2 => Button::Keyboard2,
                winit::keyboard::KeyCode::Digit3 => Button::Keyboard3,
                winit::keyboard::KeyCode::Digit4 => Button::Keyboard4,
                winit::keyboard::KeyCode::Digit5 => Button::Keyboard5,
                winit::keyboard::KeyCode::Digit6 => Button::Keyboard6,
                winit::keyboard::KeyCode::Digit7 => Button::Keyboard7,
                winit::keyboard::KeyCode::Digit8 => Button::Keyboard8,
                winit::keyboard::KeyCode::Digit9 => Button::Keyboard9,
                winit::keyboard::KeyCode::Equal => Button::KeyboardEqual,
                winit::keyboard::KeyCode::IntlBackslash => Button::KeyboardIntlBackslash,
                winit::keyboard::KeyCode::IntlRo => Button::KeyboardIntlRo,
                winit::keyboard::KeyCode::IntlYen => Button::KeyboardIntlYen,
                winit::keyboard::KeyCode::KeyA => Button::KeyboardA,
                winit::keyboard::KeyCode::KeyB => Button::KeyboardB,
                winit::keyboard::KeyCode::KeyC => Button::KeyboardC,
                winit::keyboard::KeyCode::KeyD => Button::KeyboardD,
                winit::keyboard::KeyCode::KeyE => Button::KeyboardE,
                winit::keyboard::KeyCode::KeyF => Button::KeyboardF,
                winit::keyboard::KeyCode::KeyG => Button::KeyboardG,
                winit::keyboard::KeyCode::KeyH => Button::KeyboardH,
                winit::keyboard::KeyCode::KeyI => Button::KeyboardI,
                winit::keyboard::KeyCode::KeyJ => Button::KeyboardJ,
                winit::keyboard::KeyCode::KeyK => Button::KeyboardK,
                winit::keyboard::KeyCode::KeyL => Button::KeyboardL,
                winit::keyboard::KeyCode::KeyM => Button::KeyboardM,
                winit::keyboard::KeyCode::KeyN => Button::KeyboardN,
                winit::keyboard::KeyCode::KeyO => Button::KeyboardO,
                winit::keyboard::KeyCode::KeyP => Button::KeyboardP,
                winit::keyboard::KeyCode::KeyQ => Button::KeyboardQ,
                winit::keyboard::KeyCode::KeyR => Button::KeyboardR,
                winit::keyboard::KeyCode::KeyS => Button::KeyboardS,
                winit::keyboard::KeyCode::KeyT => Button::KeyboardT,
                winit::keyboard::KeyCode::KeyU => Button::KeyboardU,
                winit::keyboard::KeyCode::KeyV => Button::KeyboardV,
                winit::keyboard::KeyCode::KeyW => Button::KeyboardW,
                winit::keyboard::KeyCode::KeyX => Button::KeyboardX,
                winit::keyboard::KeyCode::KeyY => Button::KeyboardY,
                winit::keyboard::KeyCode::KeyZ => Button::KeyboardZ,
                winit::keyboard::KeyCode::Minus => Button::KeyboardMinus,
                winit::keyboard::KeyCode::Period => Button::KeyboardPeriod,
                winit::keyboard::KeyCode::Quote => Button::KeyboardQuote,
                winit::keyboard::KeyCode::Semicolon => Button::KeyboardSemicolon,
                winit::keyboard::KeyCode::Slash => Button::KeyboardSlash,
                winit::keyboard::KeyCode::AltLeft => Button::KeyboardAltLeft,
                winit::keyboard::KeyCode::AltRight => Button::KeyboardAltRight,
                winit::keyboard::KeyCode::Backspace => Button::KeyboardBackspace,
                winit::keyboard::KeyCode::CapsLock => Button::KeyboardCapsLock,
                winit::keyboard::KeyCode::ContextMenu => Button::KeyboardContextMenu,
                winit::keyboard::KeyCode::ControlLeft => Button::KeyboardControlLeft,
                winit::keyboard::KeyCode::ControlRight => Button::KeyboardControlRight,
                winit::keyboard::KeyCode::Enter => Button::KeyboardEnter,
                winit::keyboard::KeyCode::SuperLeft => Button::KeyboardSuperLeft,
                winit::keyboard::KeyCode::SuperRight => Button::KeyboardSuperRight,
                winit::keyboard::KeyCode::ShiftLeft => Button::KeyboardShiftLeft,
                winit::keyboard::KeyCode::ShiftRight => Button::KeyboardShiftRight,
                winit::keyboard::KeyCode::Space => Button::KeyboardSpace,
                winit::keyboard::KeyCode::Tab => Button::KeyboardTab,
                winit::keyboard::KeyCode::Convert => Button::KeyboardConvert,
                winit::keyboard::KeyCode::KanaMode => Button::KeyboardKanaMode,
                winit::keyboard::KeyCode::Lang1 => Button::KeyboardLang1,
                winit::keyboard::KeyCode::Lang2 => Button::KeyboardLang2,
                winit::keyboard::KeyCode::Lang3 => Button::KeyboardLang3,
                winit::keyboard::KeyCode::Lang4 => Button::KeyboardLang4,
                winit::keyboard::KeyCode::Lang5 => Button::KeyboardLang5,
                winit::keyboard::KeyCode::NonConvert => Button::KeyboardNonConvert,
                winit::keyboard::KeyCode::Delete => Button::KeyboardDelete,
                winit::keyboard::KeyCode::End => Button::KeyboardEnd,
                winit::keyboard::KeyCode::Help => Button::KeyboardHelp,
                winit::keyboard::KeyCode::Home => Button::KeyboardHome,
                winit::keyboard::KeyCode::Insert => Button::KeyboardInsert,
                winit::keyboard::KeyCode::PageDown => Button::KeyboardPageDown,
                winit::keyboard::KeyCode::PageUp => Button::KeyboardPageUp,
                winit::keyboard::KeyCode::ArrowDown => Button::KeyboardArrowDown,
                winit::keyboard::KeyCode::ArrowLeft => Button::KeyboardArrowLeft,
                winit::keyboard::KeyCode::ArrowRight => Button::KeyboardArrowRight,
                winit::keyboard::KeyCode::ArrowUp => Button::KeyboardArrowUp,
                winit::keyboard::KeyCode::NumLock => Button::KeyboardNumLock,
                winit::keyboard::KeyCode::Numpad0 => Button::Numpad0,
                winit::keyboard::KeyCode::Numpad1 => Button::Numpad1,
                winit::keyboard::KeyCode::Numpad2 => Button::Numpad2,
                winit::keyboard::KeyCode::Numpad3 => Button::Numpad3,
                winit::keyboard::KeyCode::Numpad4 => Button::Numpad4,
                winit::keyboard::KeyCode::Numpad5 => Button::Numpad5,
                winit::keyboard::KeyCode::Numpad6 => Button::Numpad6,
                winit::keyboard::KeyCode::Numpad7 => Button::Numpad7,
                winit::keyboard::KeyCode::Numpad8 => Button::Numpad8,
                winit::keyboard::KeyCode::Numpad9 => Button::Numpad9,
                winit::keyboard::KeyCode::NumpadAdd => Button::NumpadAdd,
                winit::keyboard::KeyCode::NumpadBackspace => Button::NumpadBackspace,
                winit::keyboard::KeyCode::NumpadClear => Button::NumpadClear,
                winit::keyboard::KeyCode::NumpadClearEntry => Button::NumpadClearEntry,
                winit::keyboard::KeyCode::NumpadComma => Button::NumpadComma,
                winit::keyboard::KeyCode::NumpadDecimal => Button::NumpadDecimal,
                winit::keyboard::KeyCode::NumpadDivide => Button::NumpadDivide,
                winit::keyboard::KeyCode::NumpadEnter => Button::NumpadEnter,
                winit::keyboard::KeyCode::NumpadEqual => Button::NumpadEqual,
                winit::keyboard::KeyCode::NumpadHash => Button::NumpadHash,
                winit::keyboard::KeyCode::NumpadMemoryAdd => Button::NumpadMemoryAdd,
                winit::keyboard::KeyCode::NumpadMemoryClear => Button::NumpadMemoryClear,
                winit::keyboard::KeyCode::NumpadMemoryRecall => Button::NumpadMemoryRecall,
                winit::keyboard::KeyCode::NumpadMemoryStore => Button::NumpadMemoryStore,
                winit::keyboard::KeyCode::NumpadMemorySubtract => Button::NumpadMemorySubtract,
                winit::keyboard::KeyCode::NumpadMultiply => Button::NumpadMultiply,
                winit::keyboard::KeyCode::NumpadParenLeft => Button::NumpadParenLeft,
                winit::keyboard::KeyCode::NumpadParenRight => Button::NumpadParenRight,
                winit::keyboard::KeyCode::NumpadStar => Button::NumpadStar,
                winit::keyboard::KeyCode::NumpadSubtract => Button::NumpadSubtract,
                winit::keyboard::KeyCode::Escape => Button::KeyboardEscape,
                winit::keyboard::KeyCode::Fn => Button::KeyboardFn,
                winit::keyboard::KeyCode::FnLock => Button::KeyboardFnLock,
                winit::keyboard::KeyCode::PrintScreen => Button::KeyboardPrintScreen,
                winit::keyboard::KeyCode::ScrollLock => Button::KeyboardScrollLock,
                winit::keyboard::KeyCode::Pause => Button::KeyboardPause,
                winit::keyboard::KeyCode::BrowserBack => Button::KeyboardBrowserBack,
                winit::keyboard::KeyCode::BrowserFavorites => Button::KeyboardBrowserFavorites,
                winit::keyboard::KeyCode::BrowserForward => Button::KeyboardBrowserForward,
                winit::keyboard::KeyCode::BrowserHome => Button::KeyboardBrowserHome,
                winit::keyboard::KeyCode::BrowserRefresh => Button::KeyboardBrowserRefresh,
                winit::keyboard::KeyCode::BrowserSearch => Button::KeyboardBrowserSearch,
                winit::keyboard::KeyCode::BrowserStop => Button::KeyboardBrowserStop,
                winit::keyboard::KeyCode::Eject => Button::KeyboardEject,
                winit::keyboard::KeyCode::LaunchApp1 => Button::KeyboardLaunchApp1,
                winit::keyboard::KeyCode::LaunchApp2 => Button::KeyboardLaunchApp2,
                winit::keyboard::KeyCode::LaunchMail => Button::KeyboardLaunchMail,
                winit::keyboard::KeyCode::MediaPlayPause => Button::KeyboardMediaPlayPause,
                winit::keyboard::KeyCode::MediaSelect => Button::KeyboardMediaSelect,
                winit::keyboard::KeyCode::MediaStop => Button::KeyboardMediaStop,
                winit::keyboard::KeyCode::MediaTrackNext => Button::KeyboardMediaTrackNext,
                winit::keyboard::KeyCode::MediaTrackPrevious => Button::KeyboardMediaTrackPrevious,
                winit::keyboard::KeyCode::Power => Button::KeyboardPower,
                winit::keyboard::KeyCode::Sleep => Button::KeyboardSleep,
                winit::keyboard::KeyCode::AudioVolumeDown => Button::KeyboardAudioVolumeDown,
                winit::keyboard::KeyCode::AudioVolumeMute => Button::KeyboardAudioVolumeMute,
                winit::keyboard::KeyCode::AudioVolumeUp => Button::KeyboardAudioVolumeUp,
                winit::keyboard::KeyCode::WakeUp => Button::KeyboardWakeUp,
                winit::keyboard::KeyCode::Meta => Button::KeyboardMeta,
                winit::keyboard::KeyCode::Hyper => Button::KeyboardHyper,
                winit::keyboard::KeyCode::Turbo => Button::KeyboardTurbo,
                winit::keyboard::KeyCode::Abort => Button::KeyboardAbort,
                winit::keyboard::KeyCode::Resume => Button::KeyboardResume,
                winit::keyboard::KeyCode::Suspend => Button::KeyboardSuspend,
                winit::keyboard::KeyCode::Again => Button::KeyboardAgain,
                winit::keyboard::KeyCode::Copy => Button::KeyboardCopy,
                winit::keyboard::KeyCode::Cut => Button::KeyboardCut,
                winit::keyboard::KeyCode::Find => Button::KeyboardFind,
                winit::keyboard::KeyCode::Open => Button::KeyboardOpen,
                winit::keyboard::KeyCode::Paste => Button::KeyboardPaste,
                winit::keyboard::KeyCode::Props => Button::KeyboardProps,
                winit::keyboard::KeyCode::Select => Button::KeyboardSelect,
                winit::keyboard::KeyCode::Undo => Button::KeyboardUndo,
                winit::keyboard::KeyCode::Hiragana => Button::KeyboardHiragana,
                winit::keyboard::KeyCode::Katakana => Button::KeyboardKatakana,
                winit::keyboard::KeyCode::F1 => Button::KeyboardF1,
                winit::keyboard::KeyCode::F2 => Button::KeyboardF2,
                winit::keyboard::KeyCode::F3 => Button::KeyboardF3,
                winit::keyboard::KeyCode::F4 => Button::KeyboardF4,
                winit::keyboard::KeyCode::F5 => Button::KeyboardF5,
                winit::keyboard::KeyCode::F6 => Button::KeyboardF6,
                winit::keyboard::KeyCode::F7 => Button::KeyboardF7,
                winit::keyboard::KeyCode::F8 => Button::KeyboardF8,
                winit::keyboard::KeyCode::F9 => Button::KeyboardF9,
                winit::keyboard::KeyCode::F10 => Button::KeyboardF10,
                winit::keyboard::KeyCode::F11 => Button::KeyboardF11,
                winit::keyboard::KeyCode::F12 => Button::KeyboardF12,
                winit::keyboard::KeyCode::F13 => Button::KeyboardF13,
                winit::keyboard::KeyCode::F14 => Button::KeyboardF14,
                winit::keyboard::KeyCode::F15 => Button::KeyboardF15,
                winit::keyboard::KeyCode::F16 => Button::KeyboardF16,
                winit::keyboard::KeyCode::F17 => Button::KeyboardF17,
                winit::keyboard::KeyCode::F18 => Button::KeyboardF18,
                winit::keyboard::KeyCode::F19 => Button::KeyboardF19,
                winit::keyboard::KeyCode::F20 => Button::KeyboardF20,
                winit::keyboard::KeyCode::F21 => Button::KeyboardF21,
                winit::keyboard::KeyCode::F22 => Button::KeyboardF22,
                winit::keyboard::KeyCode::F23 => Button::KeyboardF23,
                winit::keyboard::KeyCode::F24 => Button::KeyboardF24,
                winit::keyboard::KeyCode::F25 => Button::KeyboardF25,
                winit::keyboard::KeyCode::F26 => Button::KeyboardF26,
                winit::keyboard::KeyCode::F27 => Button::KeyboardF27,
                winit::keyboard::KeyCode::F28 => Button::KeyboardF28,
                winit::keyboard::KeyCode::F29 => Button::KeyboardF29,
                winit::keyboard::KeyCode::F30 => Button::KeyboardF30,
                winit::keyboard::KeyCode::F31 => Button::KeyboardF31,
                winit::keyboard::KeyCode::F32 => Button::KeyboardF32,
                winit::keyboard::KeyCode::F33 => Button::KeyboardF33,
                winit::keyboard::KeyCode::F34 => Button::KeyboardF34,
                winit::keyboard::KeyCode::F35 => Button::KeyboardF35,
                _ => Button::Unknown,
            },
            winit::keyboard::PhysicalKey::Unidentified(
                winit::keyboard::NativeKeyCode::Android(code),
            ) => Button::KeyboardAndroid(code),
            winit::keyboard::PhysicalKey::Unidentified(winit::keyboard::NativeKeyCode::MacOS(
                code,
            )) => Button::KeyboardMacOS(code),
            winit::keyboard::PhysicalKey::Unidentified(
                winit::keyboard::NativeKeyCode::Windows(code),
            ) => Button::KeyboardWindows(code),
            winit::keyboard::PhysicalKey::Unidentified(winit::keyboard::NativeKeyCode::Xkb(
                code,
            )) => Button::KeyboardXkb(code),
            winit::keyboard::PhysicalKey::Unidentified(
                winit::keyboard::NativeKeyCode::Unidentified,
            ) => Button::Unknown,
        }
    }
}

impl From<&winit::event::MouseButton> for Button {
    fn from(button: &winit::event::MouseButton) -> Self {
        match button {
            winit::event::MouseButton::Left => Button::MouseLeft,
            winit::event::MouseButton::Right => Button::MouseRight,
            winit::event::MouseButton::Middle => Button::MouseMiddle,
            winit::event::MouseButton::Forward => Button::MouseForward,
            winit::event::MouseButton::Back => Button::MouseBack,
            winit::event::MouseButton::Other(code) => Button::MouseOther(*code),
        }
    }
}
