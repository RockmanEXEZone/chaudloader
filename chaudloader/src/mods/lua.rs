mod lib;

use crate::{assets, mods};

pub fn new(
    name: &str,
    _info: &mods::Info,
    state: std::sync::Arc<std::sync::Mutex<mods::State>>,
    overlays: std::collections::HashMap<
        String,
        std::rc::Rc<std::cell::RefCell<assets::exedat::Overlay>>,
    >,
) -> Result<mlua::Lua, mlua::Error> {
    let lua = mlua::Lua::new();
    lib::set_globals(&lua, &name, state, overlays)?;
    Ok(lua)
}
