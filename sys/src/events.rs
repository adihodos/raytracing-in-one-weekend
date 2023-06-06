#![allow(dead_code)]

use super::keysyms::KeySymbol;
use num_derive::{FromPrimitive, ToPrimitive};

#[derive(Copy, Debug, Clone, PartialEq, Eq, Hash, FromPrimitive, ToPrimitive)]
#[repr(u8)]
pub enum ActionType {
    Press,
    Release,
}

#[derive(Copy, Debug, Clone, PartialEq, Eq, Hash, FromPrimitive, ToPrimitive)]
#[repr(u8)]
pub enum MouseButtonId {
    Button1 = 0,
    Button2,
    Button3,
    Button4,
    Button5,
}

#[derive(Copy, Debug, Clone)]
pub struct KeyEventData {
    /// < X pointer position (client coords)
    pub pointer_x: i32,
    /// < Y pointer position (client coords)
    pub pointer_y: i32,
    /// < Code of key that generated the event.
    pub keycode: KeySymbol,
    /// < Press or release
    pub type_: ActionType,

    /// < Active modifiers
    pub button1: bool,
    pub button2: bool,
    pub button3: bool,
    pub button4: bool,
    pub button5: bool,
    pub shift: bool,
    pub control: bool,
    pub name: [u8; 32],
}

impl std::default::Default for KeyEventData {
    fn default() -> Self {
        Self {
            pointer_x: 0,
            pointer_y: 0,
            keycode: KeySymbol::Unknown,
            type_: ActionType::Press,
            button1: false,
            button2: false,
            button3: false,
            button4: false,
            button5: false,
            shift: false,
            control: false,
            name: [0u8; 32],
        }
    }
}

#[derive(Copy, Debug, Clone)]
pub struct MouseButtonEventData {
    /// < X pointer position (client coords)
    pub pointer_x: i32,
    /// < Y pointer position (client coords)
    pub pointer_y: i32,
    /// < Code of button that generated the event.
    pub button: MouseButtonId,
    /// < Press or release
    pub type_: ActionType,
    /// < Active modifiers
    pub button1: bool,
    pub button2: bool,
    pub button3: bool,
    pub button4: bool,
    pub button5: bool,
    pub shift: bool,
    pub control: bool,
}

impl std::default::Default for MouseButtonEventData {
    fn default() -> Self {
        Self {
            pointer_x: 0,
            pointer_y: 0,
            button: MouseButtonId::Button1,
            type_: ActionType::Press,
            button1: false,
            button2: false,
            button3: false,
            button4: false,
            button5: false,
            shift: false,
            control: false,
        }
    }
}

#[derive(Copy, Debug, Clone)]
pub struct MouseWheelEventData {
    /// < X pointer position (client coords)
    pub pointer_x: i32,
    /// < Y pointer position (client coords)
    pub pointer_y: i32,
    /// < Amount of movement.
    pub delta: i32,
    /// < Active modifiers
    pub button1: bool,
    pub button2: bool,
    pub button3: bool,
    pub button4: bool,
    pub button5: bool,
    pub shift: bool,
    pub control: bool,
}

impl std::default::Default for MouseWheelEventData {
    fn default() -> Self {
        Self {
            pointer_x: 0,
            pointer_y: 0,
            delta: 0,
            button1: false,
            button2: false,
            button3: false,
            button4: false,
            button5: false,
            shift: false,
            control: false,
        }
    }
}

#[derive(Copy, Debug, Clone)]
pub struct MouseMotionEventData {
    /// < X pointer position (client coords)
    pub pointer_x: i32,
    /// < Y pointer position (client coords)
    pub pointer_y: i32,
    /// < Active modifiers
    pub button1: bool,
    pub button2: bool,
    pub button3: bool,
    pub button4: bool,
    pub button5: bool,
    pub shift: bool,
    pub control: bool,
}

impl std::default::Default for MouseMotionEventData {
    fn default() -> Self {
        Self {
            pointer_x: 0,
            pointer_y: 0,
            button1: false,
            button2: false,
            button3: false,
            button4: false,
            button5: false,
            shift: false,
            control: false,
        }
    }
}

#[derive(Copy, Debug, Clone)]
pub struct WindowConfigureEventData {
    pub width: i32,
    pub height: i32,
}

impl std::default::Default for WindowConfigureEventData {
    fn default() -> Self {
        Self {
            width: 0,
            height: 0,
        }
    }
}

#[derive(Copy, Debug, Clone)]
pub struct LoopEventData {
    pub surface_width: i32,
    pub surface_height: i32,
    pub window_width: i32,
    pub window_height: i32,
}

impl std::default::Default for LoopEventData {
    fn default() -> Self {
        Self {
            surface_width: 0,
            surface_height: 0,
            window_width: 0,
            window_height: 0,
        }
    }
}

#[derive(Copy, Clone, Debug)]
pub enum InputEventData {
    MouseButton(MouseButtonEventData),
    MouseWheel(MouseWheelEventData),
    MouseMotion(MouseMotionEventData),
    Key(KeyEventData),
}

#[derive(Copy, Debug, Clone)]
pub enum Event {
    Input(InputEventData),
    Configure(WindowConfigureEventData),
    Loop(LoopEventData),
    InputBegin,
    InputEnd,
}

pub type EventHandlerFn = dyn Fn(&Event);
