mod exedat;
mod mpak;
mod r#unsafe;

use crate::{assets, mods, path};
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

    table.set(
        "read_mod_file",
        lua.create_function({
            let mod_path = mod_path.clone();
            move |lua, (path,): (String,)| {
                let path = path::ensure_safe(std::path::Path::new(&path))
                    .ok_or_else(|| anyhow::anyhow!("cannot read files outside of mod directory"))
                    .map_err(|e| e.into_lua_err())?;
                Ok(lua.create_string(&std::fs::read(mod_path.join(path))?)?)
            }
        })?,
    )?;

    table.set("GAME_ENV", new_game_env(lua, game_env)?)?;
    table.set("MOD_ENV", new_mod_env(lua, name)?)?;

    table.set("ExeDat", exedat::new(lua, overlays)?)?;
    table.set("Mpak", mpak::new(lua)?)?;

    if info.r#unsafe {
        table.set(
            "unsafe",
            r#unsafe::new(lua, &mod_path, std::rc::Rc::clone(&state))?,
        )?;
    }

    Ok(mlua::Value::Table(table))
}
