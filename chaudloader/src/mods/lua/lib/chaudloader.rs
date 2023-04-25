mod bytearray;
mod exedat;
mod modfiles;
mod mpak;
mod msg;
mod r#unsafe;

use crate::{assets, mods};
use mlua::ExternalError;

fn new_game_env<'a>(
    lua: &'a mlua::Lua,
    env: &'a mods::GameEnv,
) -> Result<mlua::Value<'a>, mlua::Error> {
    let table = lua.create_table()?;
    table.set("name", serde_plain::to_string(&env.volume).unwrap())?;
    table.set("exe_sha256", hex::encode(&env.exe_sha256))?;
    Ok(mlua::Value::Table(table))
}

fn new_mod_env<'a>(lua: &'a mlua::Lua, name: &'a str) -> Result<mlua::Value<'a>, mlua::Error> {
    let table = lua.create_table()?;
    let mod_path = std::path::Path::new("mods").join(name);
    table.set("name", name)?;
    table.set(
        "path",
        mod_path.as_os_str().to_str().ok_or_else(|| {
            anyhow::format_err!("failed to decipher mod path: {}", mod_path.display())
                .into_lua_err()
        })?,
    )?;
    Ok(mlua::Value::Table(table))
}

pub fn new<'a>(
    lua: &'a mlua::Lua,
    game_env: &mods::GameEnv,
    name: &'a str,
    info: &mods::Info,
    state: std::rc::Rc<std::cell::RefCell<mods::State>>,
    overlays: std::collections::HashMap<
        String,
        std::rc::Rc<std::cell::RefCell<assets::exedat::Overlay>>,
    >,
) -> Result<mlua::Value<'a>, mlua::Error> {
    let table = lua.create_table()?;
    let mod_path = std::path::Path::new("mods").join(name);

    table.set("GAME_ENV", new_game_env(lua, game_env)?)?;
    table.set("MOD_ENV", new_mod_env(lua, name)?)?;

    table.set("exedat", exedat::new(lua, overlays)?)?;
    table.set("mpak", mpak::new(lua)?)?;
    table.set("bytearray", bytearray::new(lua)?)?;
    table.set("msg", msg::new(lua)?)?;
    table.set("modfiles", modfiles::new(lua, &mod_path)?)?;

    if info.r#unsafe {
        table.set(
            "unsafe",
            r#unsafe::new(lua, &mod_path, std::rc::Rc::clone(&state))?,
        )?;
    }

    table.set(
        "util",
        lua.load(include_str!("chaudloader/util.lua"))
            .set_name("=<builtin>\\chaudloader\\util.lua")
            .set_mode(mlua::ChunkMode::Text)
            .eval::<mlua::Value>()?,
    )?;

    Ok(mlua::Value::Table(table))
}
