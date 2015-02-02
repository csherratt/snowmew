//   Copyright 2014 Colin Sherratt
//
//   Licensed under the Apache License, Version 2.0 (the "License");
//   you may not use this file except in compliance with the License.
//   You may obtain a copy of the License at
//
//       http://www.apache.org/licenses/LICENSE-2.0
//
//   Unless required by applicable law or agreed to in writing, software
//   distributed under the License is distributed on an "AS IS" BASIS,
//   WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
//   See the License for the specific language governing permissions and
//   limitations under the License.

use glfw;

#[derive(Clone, Debug, Eq, PartialEq, Hash, RustcDecodable, RustcEncodable, Copy)]
pub enum Button {
    KeyboardSpace,
    KeyboardApostrophe,
    KeyboardComma,
    KeyboardMinus,
    KeyboardPeriod,
    KeyboardSlash,
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
    KeyboardSemicolon,
    KeyboardEqual,
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
    KeyboardLeftBracket,
    KeyboardBackslash,
    KeyboardRightBracket,
    KeyboardGraveAccent,
    KeyboardWorld1,
    KeyboardWorld2,
    KeyboardEscape,
    KeyboardEnter,
    KeyboardTab,
    KeyboardBackspace,
    KeyboardInsert,
    KeyboardDelete,
    KeyboardRight,
    KeyboardLeft,
    KeyboardDown,
    KeyboardUp,
    KeyboardPageUp,
    KeyboardPageDown,
    KeyboardHome,
    KeyboardEnd,
    KeyboardCapsLock,
    KeyboardScrollLock,
    KeyboardNumLock,
    KeyboardPrintScreen,
    KeyboardPause,
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
    KeyboardKp0,
    KeyboardKp1,
    KeyboardKp2,
    KeyboardKp3,
    KeyboardKp4,
    KeyboardKp5,
    KeyboardKp6,
    KeyboardKp7,
    KeyboardKp8,
    KeyboardKp9,
    KeyboardKpDecimal,
    KeyboardKpDivide,
    KeyboardKpMultiply,
    KeyboardKpSubtract,
    KeyboardKpAdd,
    KeyboardKpEnter,
    KeyboardKpEqual,
    KeyboardLeftShift,
    KeyboardLeftControl,
    KeyboardLeftAlt,
    KeyboardLeftSuper,
    KeyboardRightShift,
    KeyboardRightControl,
    KeyboardRightAlt,
    KeyboardRightSuper,
    KeyboardMenu,
    MouseLeft,
    MouseRight,
    MouseCenter,
    MouseExt0,
    MouseExt1,
    MouseExt2,
    MouseExt3,
    MouseExt4
}

fn from_glfw_key(key: glfw::Key) -> Button {
    return match key {
        glfw::Key::Space => Button::KeyboardSpace,
        glfw::Key::Apostrophe => Button::KeyboardApostrophe,
        glfw::Key::Comma => Button::KeyboardComma,
        glfw::Key::Minus => Button::KeyboardMinus,
        glfw::Key::Period => Button::KeyboardPeriod,
        glfw::Key::Slash => Button::KeyboardSlash,
        glfw::Key::Num0 => Button::Keyboard0,
        glfw::Key::Num1 => Button::Keyboard1,
        glfw::Key::Num2 => Button::Keyboard2,
        glfw::Key::Num3 => Button::Keyboard3,
        glfw::Key::Num4 => Button::Keyboard4,
        glfw::Key::Num5 => Button::Keyboard5,
        glfw::Key::Num6 => Button::Keyboard6,
        glfw::Key::Num7 => Button::Keyboard7,
        glfw::Key::Num8 => Button::Keyboard8,
        glfw::Key::Num9 => Button::Keyboard9,
        glfw::Key::Semicolon => Button::KeyboardSemicolon,
        glfw::Key::Equal => Button::KeyboardEqual,
        glfw::Key::A => Button::KeyboardA,
        glfw::Key::B => Button::KeyboardB,
        glfw::Key::C => Button::KeyboardC,
        glfw::Key::D => Button::KeyboardD,
        glfw::Key::E => Button::KeyboardE,
        glfw::Key::F => Button::KeyboardF,
        glfw::Key::G => Button::KeyboardG,
        glfw::Key::H => Button::KeyboardH,
        glfw::Key::I => Button::KeyboardI,
        glfw::Key::J => Button::KeyboardJ,
        glfw::Key::K => Button::KeyboardK,
        glfw::Key::L => Button::KeyboardL,
        glfw::Key::M => Button::KeyboardM,
        glfw::Key::N => Button::KeyboardN,
        glfw::Key::O => Button::KeyboardO,
        glfw::Key::P => Button::KeyboardP,
        glfw::Key::Q => Button::KeyboardQ,
        glfw::Key::R => Button::KeyboardR,
        glfw::Key::S => Button::KeyboardS,
        glfw::Key::T => Button::KeyboardT,
        glfw::Key::U => Button::KeyboardU,
        glfw::Key::V => Button::KeyboardV,
        glfw::Key::W => Button::KeyboardW,
        glfw::Key::X => Button::KeyboardX,
        glfw::Key::Y => Button::KeyboardY,
        glfw::Key::Z => Button::KeyboardZ,
        glfw::Key::LeftBracket => Button::KeyboardLeftBracket,
        glfw::Key::Backslash => Button::KeyboardBackslash,
        glfw::Key::RightBracket => Button::KeyboardRightBracket,
        glfw::Key::GraveAccent => Button::KeyboardGraveAccent,
        glfw::Key::World1 => Button::KeyboardWorld1,
        glfw::Key::World2 => Button::KeyboardWorld2,
        glfw::Key::Escape => Button::KeyboardEscape,
        glfw::Key::Enter => Button::KeyboardEnter,
        glfw::Key::Tab => Button::KeyboardTab,
        glfw::Key::Backspace => Button::KeyboardBackspace,
        glfw::Key::Insert => Button::KeyboardInsert,
        glfw::Key::Delete => Button::KeyboardDelete,
        glfw::Key::Right => Button::KeyboardRight,
        glfw::Key::Left => Button::KeyboardLeft,
        glfw::Key::Down => Button::KeyboardDown,
        glfw::Key::Up => Button::KeyboardUp,
        glfw::Key::PageUp => Button::KeyboardPageUp,
        glfw::Key::PageDown => Button::KeyboardPageDown,
        glfw::Key::Home => Button::KeyboardHome,
        glfw::Key::End => Button::KeyboardEnd,
        glfw::Key::CapsLock => Button::KeyboardCapsLock,
        glfw::Key::ScrollLock => Button::KeyboardScrollLock,
        glfw::Key::NumLock => Button::KeyboardNumLock,
        glfw::Key::PrintScreen => Button::KeyboardPrintScreen,
        glfw::Key::Pause => Button::KeyboardPause,
        glfw::Key::F1 => Button::KeyboardF1,
        glfw::Key::F2 => Button::KeyboardF2,
        glfw::Key::F3 => Button::KeyboardF3,
        glfw::Key::F4 => Button::KeyboardF4,
        glfw::Key::F5 => Button::KeyboardF5,
        glfw::Key::F6 => Button::KeyboardF6,
        glfw::Key::F7 => Button::KeyboardF7,
        glfw::Key::F8 => Button::KeyboardF8,
        glfw::Key::F9 => Button::KeyboardF9,
        glfw::Key::F10 => Button::KeyboardF10,
        glfw::Key::F11 => Button::KeyboardF11,
        glfw::Key::F12 => Button::KeyboardF12,
        glfw::Key::F13 => Button::KeyboardF13,
        glfw::Key::F14 => Button::KeyboardF14,
        glfw::Key::F15 => Button::KeyboardF15,
        glfw::Key::F16 => Button::KeyboardF16,
        glfw::Key::F17 => Button::KeyboardF17,
        glfw::Key::F18 => Button::KeyboardF18,
        glfw::Key::F19 => Button::KeyboardF19,
        glfw::Key::F20 => Button::KeyboardF20,
        glfw::Key::F21 => Button::KeyboardF21,
        glfw::Key::F22 => Button::KeyboardF22,
        glfw::Key::F23 => Button::KeyboardF23,
        glfw::Key::F24 => Button::KeyboardF24,
        glfw::Key::F25 => Button::KeyboardF25,
        glfw::Key::Kp0 => Button::KeyboardKp0,
        glfw::Key::Kp1 => Button::KeyboardKp1,
        glfw::Key::Kp2 => Button::KeyboardKp2,
        glfw::Key::Kp3 => Button::KeyboardKp3,
        glfw::Key::Kp4 => Button::KeyboardKp4,
        glfw::Key::Kp5 => Button::KeyboardKp5,
        glfw::Key::Kp6 => Button::KeyboardKp6,
        glfw::Key::Kp7 => Button::KeyboardKp7,
        glfw::Key::Kp8 => Button::KeyboardKp8,
        glfw::Key::Kp9 => Button::KeyboardKp9,
        glfw::Key::KpDecimal => Button::KeyboardKpDecimal,
        glfw::Key::KpDivide => Button::KeyboardKpDivide,
        glfw::Key::KpMultiply => Button::KeyboardKpMultiply,
        glfw::Key::KpSubtract => Button::KeyboardKpSubtract,
        glfw::Key::KpAdd => Button::KeyboardKpAdd,
        glfw::Key::KpEnter => Button::KeyboardKpEnter,
        glfw::Key::KpEqual => Button::KeyboardKpEqual,
        glfw::Key::LeftShift => Button::KeyboardLeftShift,
        glfw::Key::LeftControl => Button::KeyboardLeftControl,
        glfw::Key::LeftAlt => Button::KeyboardLeftAlt,
        glfw::Key::LeftSuper => Button::KeyboardLeftSuper,
        glfw::Key::RightShift => Button::KeyboardRightShift,
        glfw::Key::RightControl => Button::KeyboardRightControl,
        glfw::Key::RightAlt => Button::KeyboardRightAlt,
        glfw::Key::RightSuper => Button::KeyboardRightSuper,
        glfw::Key::Menu => Button::KeyboardMenu,
    };
}

pub fn from_glfw_mouse_button(key: glfw::MouseButton) -> Button {
    return match key {
        glfw::MouseButton::Button1 => Button::MouseLeft,
        glfw::MouseButton::Button2 => Button::MouseRight,
        glfw::MouseButton::Button3 => Button::MouseCenter,
        glfw::MouseButton::Button4 => Button::MouseExt0,
        glfw::MouseButton::Button5 => Button::MouseExt1,
        glfw::MouseButton::Button6 => Button::MouseExt2,
        glfw::MouseButton::Button7 => Button::MouseExt3,
        glfw::MouseButton::Button8 => Button::MouseExt4,
    };
}

#[derive(Clone, Debug, PartialEq, Copy)]
pub enum Event {
    ButtonDown(Button),
    ButtonUp(Button),
    Move(f64, f64),
    Scroll(f64, f64),
    Cadance(f64)
}

#[derive(Clone, Debug, PartialEq, Copy)]
pub enum WindowEvent {
    MouseOver(bool),
    Position(i32, i32),
    Size(u32, u32)
}

#[derive(Clone, Debug, PartialEq, Copy)]
pub enum EventGroup {
    Game(Event),
    Window(WindowEvent),
    Nop
}

impl Event {
    pub fn from_glfw(evt: glfw::WindowEvent) -> EventGroup {
        match evt {
            glfw::WindowEvent::MouseButton(button, glfw::Action::Press, _) => {
                EventGroup::Game(Event::ButtonDown(from_glfw_mouse_button(button)))
            }
            glfw::WindowEvent::MouseButton(button, glfw::Action::Release, _) => {
                EventGroup::Game(Event::ButtonUp(from_glfw_mouse_button(button)))
            }
            glfw::WindowEvent::Key(button, _, glfw::Action::Press, _) => {
                EventGroup::Game(Event::ButtonDown(from_glfw_key(button)))
            }
            glfw::WindowEvent::Key(button, _, glfw::Action::Release, _) => {
                EventGroup::Game(Event::ButtonUp(from_glfw_key(button)))
            }
            glfw::WindowEvent::CursorPos(x, y) => {
                EventGroup::Game(Event::Move(x, y))
            }
            glfw::WindowEvent::Scroll(x, y) => {
                EventGroup::Game(Event::Scroll(x, y))
            }
            glfw::WindowEvent::CursorEnter(x) => {
                EventGroup::Window(WindowEvent::MouseOver(x))
            }
            glfw::WindowEvent::Pos(x, y) => {
                EventGroup::Window(WindowEvent::Position(x as i32, y as i32))
            }
            glfw::WindowEvent::FramebufferSize(x, y) => {
                EventGroup::Window(WindowEvent::Size(x as u32, y as u32))
            }
            x => {
                println!("unhandled {:?}", x);
                EventGroup::Nop
            }
        }
    }
}
