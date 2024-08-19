use convert_case::Casing;
use crossterm::event::{Event, KeyCode, KeyEvent, KeyEventKind, MouseEvent};
use itertools::Itertools;
use mlua::{UserData, UserDataFields, UserDataMethods, Value};

pub struct EventWrapper(
    pub Vec<Event>
);
struct KeyEventWrapper(
    Vec<KeyEvent>
);
struct MouseEventWrapper(
    Vec<MouseEvent>
);

impl UserData for EventWrapper {
    fn add_fields<'lua, F: UserDataFields<'lua, Self>>(fields: &mut F) {
        fields.add_field_method_get("focus_gained", |_, this| {
            Ok(Value::Boolean(this.0.contains(&Event::FocusGained)))
        });
        fields.add_field_method_get("focus_lost", |_, this| {
            Ok(Value::Boolean(this.0.contains(&Event::FocusLost)))
        });

        fields.add_field_method_get("key", |_, this| {
            let key_events = this.0.iter()
                .filter_map(|e| {
                    match e {
                        Event::Key(ke) if ke.kind == KeyEventKind::Press => 
                            Some(ke.to_owned()),
                        _ => None
                    }
                })
                .collect_vec();
                    
            Ok(KeyEventWrapper(key_events))
        });
        fields.add_field_method_get("mouse", |_, this| {
            let mouse_events = this.0.iter()
                .filter_map(|e| if let Event::Mouse(me) = e {Some(me.to_owned())} else {None})
                .collect_vec();
            
            Ok(MouseEventWrapper(mouse_events))
        });
    }
}

impl UserData for KeyEventWrapper {
    fn add_methods<'lua, M: UserDataMethods<'lua, Self>>(methods: &mut M) {
        methods.add_method("is_pressed", |_, this, mut key: String| {
            if key.len() > 1 {
                key = key.to_case(convert_case::Case::Pascal);
            }
            Ok(this.0.iter().any(|ke| {
                key == match ke.code {
                    KeyCode::Char(' ') => "Space".to_owned(),
                    KeyCode::Char(c) => c.to_string(),
                    KeyCode::F(n) => format!("F{}", n),
                    KeyCode::Modifier(modifier) => format!("{:?}", modifier),
                    KeyCode::Media(media) => format!("{:?}", media),
                    code => format!("{:?}", code)
                }
            }))
        });
    }
}

impl UserData for MouseEventWrapper {
    fn add_methods<'lua, M: UserDataMethods<'lua, Self>>(methods: &mut M) {
        methods.add_method("placeholder", |_, _this, ()| {
            Ok(0)
        });
    }
}
