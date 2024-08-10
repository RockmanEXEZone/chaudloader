mod buffer;
mod exedat;
mod modfiles;
mod mpak;
mod msg;
mod pck;
mod r#unsafe;

use crate::{assets, mods};
use mlua::ExternalError;

fn new_game_env<'a>(
    lua: &'a mlua::Lua,
    env: &'a mods::GameEnv,
) -> Result<mlua::Value<'a>, mlua::Error> {
    let sections_table = lua.create_table()?;

    if let Some(text_section) = env.sections.text {
        let text_section_table = lua.create_table()?;
        text_section_table.set("address", text_section.as_ptr() as usize)?;
        text_section_table.set("size", text_section.len())?;
        sections_table.set("text", text_section_table)?;
    } else {
        sections_table.set("text", mlua::Value::Nil)?;
    }

    let game_env_table = lua.create_table()?;
    game_env_table.set("name", serde_plain::to_string(&env.volume).unwrap())?;
    game_env_table.set("exe_crc32", env.exe_crc32)?;
    game_env_table.set("sections", sections_table)?;
    Ok(mlua::Value::Table(game_env_table))
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
    table.set("buffer", buffer::new(lua)?)?;
    table.set("msg", msg::new(lua)?)?;
    table.set("modfiles", modfiles::new(lua, &mod_path)?)?;
    table.set("pck", pck::new(lua, &mod_path)?)?;

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
