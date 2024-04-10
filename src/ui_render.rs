
use mlua::Value;
use tui::{backend::CrosstermBackend, layout::{self, Rect}, style::{Modifier, Style}, widgets::{self, Block, Borders, ListState, Paragraph, StatefulWidget, TableState, Widget, Wrap}, Frame, Terminal};
use crate::*;

pub fn draw_ui(lua: &Lua) {
    do_the_lua(lua, &["ui"], |innards| {
        let ui = &innards[2];

        if let Ok(ui_elements) = ui.get::<_, Table>("UiElements") {             
            terminal().draw(|f| {
                for ui_element in ui_elements.pairs::<Value, Table>() {
                    let (_, ui_element) = ui_element.unwrap();

                    let ui_element = lua_table_to_ui_element(ui_element);
                    match ui_element {
                        Ok((widget_type, rect, widget_state)) => {
                            render_ui_element(f, widget_type, rect, widget_state);
                        },
                        Err(e) => write_to_debug(format!("{:?}", e)),
                    }
                }
            }).unwrap();

            unsafe { UI_REDRAW_QUEUED = false; }
        }

    });
}

fn render_ui_element(f: &mut Frame<'_, CrosstermBackend<Writer>>, widget_type: WidgetType, rect: Rect, widget_state: Option<WidgetState>) {
    match widget_type {
        WidgetType::Block(w) => {
            f.render_widget(w, rect);
        },
        WidgetType::Tabs(w) => {
            f.render_widget(w, rect);
        },
        WidgetType::List(w) => {
            match widget_state {
                Some(WidgetState::List(mut s)) => {
                    f.render_stateful_widget(w, rect, &mut s)
                },
                _ => f.render_widget(w, rect),
            }
        },
        WidgetType::Table(w) => {
            match widget_state {
                Some(WidgetState::Table(mut s)) => {
                    f.render_stateful_widget(w, rect, &mut s)
                },
                _ => f.render_widget(w, rect),
            }
        },
        WidgetType::Paragraph(w) => {
            f.render_widget(w, rect);
        },
        WidgetType::Chart(w) => {
            f.render_widget(w, rect);
        },
        WidgetType::BarChart(w) => {
            f.render_widget(w, rect);
        },
        WidgetType::Gauge(w) => {
            f.render_widget(w, rect);
        },
        WidgetType::Sparkline(w) => {
            f.render_widget(w, rect);
        },
        WidgetType::Clear(w) => {
            f.render_widget(w, rect);
        },
    }
}

pub enum WidgetType<'a> {
    Block(widgets::Block<'a>),
    Tabs(widgets::Tabs<'a>),
    List(widgets::List<'a>),
    Table(widgets::Table<'a>),
    Paragraph(widgets::Paragraph<'a>),
    Chart(widgets::Chart<'a>),
    BarChart(widgets::BarChart<'a>),
    Gauge(widgets::Gauge<'a>),
    Sparkline(widgets::Sparkline<'a>),
    Clear(widgets::Clear)
}

pub enum WidgetState {
    List(ListState),
    Table(TableState)
}

const LUA_TO_UI_ERR: mlua::Error = mlua::Error::FromLuaConversionError { from: "str", to: "ui", message: None };

pub fn str_to_ui_alignment(a: &str) -> Result<tui::layout::Alignment, mlua::Error> {
    Ok(match a {
        "center" => {tui::layout::Alignment::Center},
        "right"  => {tui::layout::Alignment::Right},
        "left"   => {tui::layout::Alignment::Left},

        _ => return Err(LUA_TO_UI_ERR)
    })
}

pub fn lua_table_to_text_modifier(table: Table) -> Result<Modifier, mlua::Error> {
    let mut rusty_modifiers = Modifier::empty();
        
    for pair in table.pairs::<String, bool>() {
        let (modifier, enabled) = pair?;
        let modifier: &str = &modifier.to_lowercase();

        let modifier = match modifier {
            "bold"        => { Modifier::BOLD },
            "dim"         => { Modifier::DIM },
            "italic"      => { Modifier::ITALIC },
            "underlined"  => { Modifier::UNDERLINED },
            "slowBlink"   => { Modifier::SLOW_BLINK },
            "rapidBlink"  => { Modifier::RAPID_BLINK },
            "reversed"    => { Modifier::REVERSED },
            "hidden"      => { Modifier::HIDDEN },
            "crossedOut"  => { Modifier::CROSSED_OUT },

             _ => return Err(LUA_TO_UI_ERR)
        };
        match enabled {
            true => rusty_modifiers.insert(modifier),
            false => rusty_modifiers.remove(modifier),
        }
    }

    Ok(rusty_modifiers)
}

pub fn lua_table_to_color(table: Table) -> Result<tui::style::Color, mlua::Error> {
    Ok(tui::style::Color::Rgb(
        table.get::<_, u8>(1)?,
        table.get::<_, u8>(2)?,
        table.get::<_, u8>(3)?
    ))
}

pub fn lua_table_to_ui_style(table: Table) -> Result<Style, mlua::Error> {
    let mut style = Style::default();
    if let Ok(bg) = table.get::<_, Table>("bg") {
        style = style.bg(lua_table_to_color(bg)?);
    }
    if let Ok(fg) = table.get::<_, Table>("fg") {
        style = style.fg(lua_table_to_color(fg)?);
    }
    if let Ok(modifiers) = table.get::<_, Table>("modifiers") {
        style = style.add_modifier(lua_table_to_text_modifier(modifiers)?);
    }
    
    
    Ok(style)
}

pub fn lua_table_to_ui_element(table: Table) -> Result<(WidgetType, Rect, Option<WidgetState>), mlua::Error> {
    let data = table.get::<_, Table>("data")?;

    let (widget_type, rect, mut widget_state);
    widget_state = None;

    let lua_rect = table.get::<_, Table>("rect")?;
    rect = Rect::new(
        lua_rect.get::<_, u16>("x")?,
        lua_rect.get::<_, u16>("y")?,
        lua_rect.get::<_, u16>("width")?,
        lua_rect.get::<_, u16>("height")?,
    );

    match table.get::<_, String>("type")?.as_str() {
        "block" => {
            let mut block = Block::default();

            if let Ok(title) = data.get::<_, String>("title") { 
                block = block.title(title); 
            }
            if let Ok(title_alignment) = data.get::<_, String>("titleAlignment") {
                block = block.title_alignment(str_to_ui_alignment(title_alignment.to_lowercase().as_str())?);
            }
            if let Ok(style) = data.get::<_, Table>("style") {
                block = block.style(lua_table_to_ui_style(style)?);
            }

            if let Ok(borders) = data.get::<_, Table>("borders") {
                let mut rusty_borders = Borders::empty();
        
                for pair in borders.pairs::<String, bool>() {
                    let (border, enabled) = pair?;
                    let border: &str = &border.to_lowercase();
                
                    let border = match border {
                        "all"    => { Borders::ALL },
                        "bottom" => { Borders::BOTTOM },
                        "left"   => { Borders::LEFT },
                        "top"    => { Borders::TOP },
                        "none"   => { Borders::NONE },

                        _ => return Err(LUA_TO_UI_ERR)
                    };
                    match enabled {
                        true => rusty_borders.insert(border),
                        false => rusty_borders.remove(border),
                    }
                }
            
                block = block.borders(rusty_borders);
            }

            if let Ok(border_type) = data.get::<_, String>("borderType") {
                block = match border_type.as_str() {
                    "plain" => block.border_type(widgets::BorderType::Plain),
                    "double" => block.border_type(widgets::BorderType::Double),
                    "thick" => block.border_type(widgets::BorderType::Thick),
                    "rounded" => block.border_type(widgets::BorderType::Rounded),
                    _ => return Err(LUA_TO_UI_ERR),
                }
            }
            
            widget_type = WidgetType::Block(block);
        },
        "tabs" => {
            todo!()
        },
        "list" => {
            widget_state = Some(todo!());
        },
        "table" => {
            widget_state = Some(todo!());
        },
        "paragraph" => {
            todo!()
        },
        "chart" => {
            todo!()
        },
        "barchart" => {
            todo!()
        },
        "gauge" => {
            todo!()
        },
        "sparkline" => {
            todo!()
        },
        _ => unimplemented!(),
    }
    
    Ok((widget_type, rect, widget_state))
}