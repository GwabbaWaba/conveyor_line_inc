use std::{cell::Cell, rc::Rc, sync::RwLock};
use crossterm::{execute, terminal};
use paste::paste;
use mlua::{AnyUserData, Lua, Table, UserData, UserDataMethods, Value};
use ratatui::layout::{Constraint, Direction, Layout, Rect};

use crate::{CrossTerminal, DynErrResult, GameWrapper};



pub struct TerminalWrapper (
    pub Rc<RwLock<CrossTerminal>>
);

macro_rules! WidgetWrapper {
    ($widget_type: ident) => {
        paste! {
            pub struct [<$widget_type Wrapper>]<'a> (pub ratatui::widgets::$widget_type<'a>);
            impl<'a> UserData for [<$widget_type Wrapper>]<'a> {}
        }
    };
    ($widget_type: ident, $imple: block) => {
        paste! {
            pub struct [<$widget_type Wrapper>]<'a> (pub ratatui::widgets::$widget_type<'a>);
            impl<'a> UserData for [<$widget_type Wrapper>]<'a> $imple
        }
    };
}

WidgetWrapper!(Block);
WidgetWrapper!(Paragraph);


impl UserData for TerminalWrapper {
    fn add_methods<'lua, M: UserDataMethods<'lua, Self>>(methods: &mut M) {
        methods.add_method_mut("set_text_wrap", |_, this, does_wrap: bool| {
            let mut this  = this.0.write().unwrap();
            let backend = this.backend_mut();
            if does_wrap {
                execute!(
                    backend,
                    terminal::EnableLineWrap
                )?;
            } else {
                execute!(
                    backend,
                    terminal::DisableLineWrap
                )?;
            };

            Ok(())
        });

        methods.add_method("window_size", |lua, this, ()| {
            let lua_window_size = lua.create_table()?;
            let rs_window_size = this.0.write().unwrap().size()?;
            lua_window_size.set("width", rs_window_size.width)?;
            lua_window_size.set("height", rs_window_size.height)?;

            Ok(lua_window_size)
        });

        methods.add_method("draw", |lua, _, (render_item, size): (Value, Option<Table>)| {
            let args = lua.create_table()?;
            args.push(render_item)?;
            args.push(size)?;
            lua.globals().get::<_, Table>("core")?.get::<_, Table>("draw_buffer")?.push(args)?;
            Ok(())
        });
    }
}

fn constraint_from_table(table: Table) -> mlua::Result<Constraint> {
    let constraint = match table.get::<_, String>(1)?.as_str() {
        "percentage" | "%" => Constraint::Percentage(table.get(2)?),
        "min" => Constraint::Min(table.get(2)?),
        "max" => Constraint::Max(table.get(2)?),

        "ratio" => Constraint::Ratio(table.get(2)?, table.get(3)?),

        _ => unimplemented!()
    };
    Ok(constraint)
}

fn direction_from_string(string: String) -> Direction {
    match string.to_lowercase().as_str() {
        "vertical" | ":" => Direction::Vertical,
        "horizontal" | ".." => Direction::Horizontal,
        _ => unimplemented!()
    }
}

pub fn flush_draw_buffer<'a>(lua: &Lua, game: &'a GameWrapper) -> mlua::Result<()> {
    let core = lua.globals().get::<_, Table>("core")?;
    let lua_draw_buffer = core.get::<_, Table>("draw_buffer")?;
    let mut rs_draw_buffer = Vec::new();

    for entry in lua_draw_buffer.pairs::<Value, Table>() {
        let (_, draw_args) = entry?;
        let (render_item, size) = (
            draw_args.get::<_, Value>(1)?, 
            draw_args.get::<_, Option<Table>>(2)?
        );
        let f_size = terminal::size()?;
        let size = if let Some(size) = size {
            Rect{
                x: size.get("x").unwrap_or(0),
                y: size.get("y").unwrap_or(0),
                width: size.get("width").unwrap_or(f_size.1),
                height: size.get("height").unwrap_or(f_size.0),
            }
        } else {
            Rect::new(0, 0, f_size.1, f_size.0)
        };

        let _ = build_rs_draw_buffer(&lua, render_item, &size, &mut rs_draw_buffer);
    }
    let mut terminal = game.terminal();
    terminal.draw(move |f| {
        macro_rules! if_ {
            ($widget: ident, $size: ident, $widget_type: ident) => {
                paste! {
                    if let Ok(w) = $widget.borrow::<[<$widget_type Wrapper>]>() {
                        f.render_widget(&w.0, $size);
                    }
                }
            };
        }
        for entry in rs_draw_buffer {
            let size = entry.size.clamp(f.size());
            let widget = entry.widget.into_inner();
            let widget = widget.as_userdata().unwrap();

            if_!(widget, size, Block);
            if_!(widget, size, Paragraph);
        }
    })?;

    core.set("draw_buffer", lua.create_table()?)?;
    Ok(())
}

struct DrawBuffer<'a> {
    widget: Cell<Value<'a>>,
    size: Rect
}
fn build_rs_draw_buffer<'lua>(lua: &'lua Lua, render_item: Value<'lua>, size: &Rect, holder: &mut Vec<DrawBuffer<'lua>>) -> DynErrResult<()> {
    match render_item {
        Value::Table(layout) => {
            let direction = layout.get::<_, String>("direction").unwrap_or("vertical".to_owned());

            let mut constraints = Vec::new();
            let mut widgets = Vec::new();
            for layout in layout.clone().pairs::<f64, Table>() {
                let specs = if let Ok((_, specs)) = layout {specs} else { continue; };
                let (constraint, widget) = (
                    specs.get::<_, Table>(1)?,
                    specs.get::<_, AnyUserData>(2)?,
                );
                constraints.push(constraint_from_table(constraint)?);
                widgets.push(Value::UserData(widget));
            }
    
            let mut render_items = Vec::new();
            let layout = Layout::new(direction_from_string(direction), constraints).split(*size);
            for (i, size) in layout.iter().enumerate() {
                render_items.push(build_rs_draw_buffer(lua, widgets[i].clone(), size, holder));
            }
        },
        Value::UserData(widget) => {
            holder.push(DrawBuffer { widget: Cell::new(Value::UserData(widget.to_owned())), size: size.to_owned() });
        },
        _ => unreachable!()
    }
    Ok(())
}