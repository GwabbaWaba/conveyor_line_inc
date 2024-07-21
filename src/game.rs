use std::{borrow::Borrow, cell::{Ref, RefCell, RefMut}, collections::HashMap, io::{stdout, Stdout}, ops::{Deref, DerefMut}, rc::Rc, sync::{Arc, Mutex, MutexGuard, RwLock, RwLockWriteGuard}};

use mlua::Lua;
use ratatui::{backend::CrosstermBackend, Terminal};

use crate::{setup_lua, DynErrResult};

pub type CrossTerminal = Terminal<CrosstermBackend<Stdout>>;

pub struct Game {
    lua: RwLock<Lua>,
    terminal: Rc<RwLock<CrossTerminal>>
}
pub struct GameWrapper(Arc<Game>);

impl Game {
    pub fn new() -> DynErrResult<GameWrapper> {
        Ok(GameWrapper(Arc::new(
            Self {
                lua: RwLock::new(Lua::new()),
                terminal: Rc::new(RwLock::new(Terminal::new(CrosstermBackend::new(stdout()))?))
            }
        ))) // <- lisp dev's stash
    }

}

impl GameWrapper {
    #[inline]
    pub fn arc_clone(&self) -> Self {
        GameWrapper(Arc::clone(&self.0))
    }
    
    pub fn lua(&self) -> RwLockWriteGuard<'_, Lua> {
        self.0.lua.write().unwrap()
    }
    pub fn terminal(&self) -> RwLockWriteGuard<'_, CrossTerminal> {
        self.0.terminal.write().unwrap()
    }
    pub fn terminal_lock(&self) -> Rc<RwLock<CrossTerminal>> {
        Rc::clone(&self.0.terminal)
    }
}
impl Deref for GameWrapper {
    type Target = Arc<Game>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
impl DerefMut for GameWrapper {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}