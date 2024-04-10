use crossterm::{cursor, event::{KeyCode, KeyEvent, KeyEventKind, KeyEventState, KeyModifiers, MouseButton, MouseEvent, MouseEventKind}, execute};
use linked_hash_map::LinkedHashMap;
use mlua::Lua;

use crate::{call_key_events, exit_successful, terminal};

pub fn do_key_event(lua: &Lua, event: &AlmostKeyEvent, type_mode: &mut bool) {
    match (event.code, &mut *type_mode) {
        (KeyCode::Char('c'), _) if event.modifiers == KeyModifiers::CONTROL => {
            exit_successful().expect("exited, but not successfully");
        },
        (KeyCode::F(1) | KeyCode::Tab, false) => {
            *type_mode = true;
        },
        (KeyCode::F(1) | KeyCode::Tab, true) => {
            *type_mode = false;
        },
        (_, true) => {
            /* action_from_input(_) ~*/
        }
        (_, false) => {
            call_key_events(lua, &actualize_key_event(event));
        }
    }
}

pub fn determine_real_state(key_event: &AlmostKeyEvent) -> KeyEventState {
match key_event.state {
    KeyEventState::NONE => {
        if let KeyCode::Char(c) = key_event.code {
            if (c.is_uppercase() && !key_event.modifiers.contains(KeyModifiers::SHIFT)) ||
               (c.is_lowercase() && key_event.modifiers.contains(KeyModifiers::SHIFT)) {
                return KeyEventState::CAPS_LOCK;
            }
        }
        return KeyEventState::NONE;
    },
    _ => { key_event.state }
}
}

pub fn do_key_events(lua: &Lua, key_events: &mut LinkedHashMap<AlmostKeyEvent, KeyEventKind>, type_mode: &mut bool) {
    let mut removals = Vec::new();

    let mut last_modifiers = KeyModifiers::NONE;
    let mut last_state = KeyEventState::NONE;
    for (key_event, release_status) in key_events.iter() {
        let this_modifiers = key_event.modifiers;
        let this_state = key_event.state;

        let go_through = 
            *release_status != KeyEventKind::Release && ((
                this_modifiers == last_modifiers ||
                !last_modifiers.contains(this_modifiers) ||
                last_modifiers == KeyModifiers::NONE
            ) &&
            (
                this_state == last_state ||
                !last_state.contains(this_state) ||
                last_state == KeyEventState::NONE
            ));

        let mut queue_removal = true;
        if go_through {
            queue_removal = key_event.state.contains(KeyEventState::CAPS_LOCK);
            do_key_event(lua, key_event, type_mode);
        }

        if queue_removal {
            removals.push(key_event.clone());
        }

        last_modifiers = this_modifiers;
        last_state = this_state;
    }


    for removal in removals {
        key_events.remove(&removal);
    }
}

pub fn do_mouse_events(lua: &Lua, mouse_events: &mut LinkedHashMap<AlmostMouseEvent, MouseEventKind>) {
    let mut removals = Vec::new();
    
    for (mouse_event, click_status) in mouse_events.iter() {
        do_mouse_event(lua, &actualize_mouse_event(mouse_event, click_status));
        
        removals.push(mouse_event.clone());
    }
    
    for removal in removals {
        mouse_events.remove(&removal);
    }
}

pub fn do_mouse_event(lua: &Lua, event: &MouseEvent) {
    match (event.kind, event.modifiers) {
        (MouseEventKind::Down(MouseButton::Left) | MouseEventKind::Drag(MouseButton::Left), _) => {
            execute!(
                terminal().backend_mut(),
                cursor::MoveTo(event.column, event.row)
            ).unwrap();
            println!(".");
        },
    
        _ => {}
    }
}

/// Represents almost a key event.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Eq, PartialEq, PartialOrd, Hash, Clone)]
pub struct AlmostKeyEvent {
    pub code: KeyCode,
    pub modifiers: KeyModifiers,
    pub state: KeyEventState,
}

/// Represents almost a mouse event.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Eq, PartialEq, PartialOrd, Hash, Clone)]
pub struct AlmostMouseEvent {
    pub column: u16,
    pub row: u16,
    pub modifiers: KeyModifiers,
}

pub fn almostify_key_event(key_event: &KeyEvent) -> AlmostKeyEvent {
    AlmostKeyEvent {
        code: key_event.code,
        modifiers: key_event.modifiers,
        state: key_event.state,
    }
}

pub fn almostify_mouse_event(mouse_event: &MouseEvent) -> AlmostMouseEvent {
    AlmostMouseEvent {
        column: mouse_event.column,
        row: mouse_event.row,
        modifiers: mouse_event.modifiers,
    }
}

pub fn actualize_mouse_event(almost_mouse_event: &AlmostMouseEvent, mouse_event_kind: &MouseEventKind) -> MouseEvent {
    MouseEvent {
        kind: mouse_event_kind.clone(),
        column: almost_mouse_event.column,
        row: almost_mouse_event.row,
        modifiers: almost_mouse_event.modifiers,
    }
}

pub fn actualize_key_event(almost_key_event: &AlmostKeyEvent) -> KeyEvent {
    KeyEvent { 
        code: almost_key_event.code, 
        modifiers: almost_key_event.modifiers, 
        kind: KeyEventKind::Press, 
        state: almost_key_event.state 
    }
}