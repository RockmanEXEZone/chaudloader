mod lib;

use crate::{assets, mods};

pub fn new(
    name: &str,
    game_env: &mods::GameEnv,
    info: &mods::Info,
    state: std::rc::Rc<std::cell::RefCell<mods::State>>,
    overlays: std::collections::HashMap<
        String,
        std::rc::Rc<std::cell::RefCell<assets::exedat::Overlay>>,
    >,
) -> Result<mlua::Lua, mlua::Error> {
    let lua = if info.r#unsafe {
        unsafe { mlua::Lua::unsafe_new() }
    } else {
        mlua::Lua::new()
    };
    lib::set_globals(&lua, game_env, &name, info, state, overlays)?;
    Ok(lua)
}
