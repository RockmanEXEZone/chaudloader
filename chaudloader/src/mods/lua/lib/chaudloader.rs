mod exedat;
mod mpak;
mod r#unsafe;

use crate::{assets, mods, path};
use mlua::ExternalError;
use std::str::FromStr;

fn new_env<'a>(lua: &'a mlua::Lua, env: &'a mods::Env) -> Result<mlua::Value<'a>, mlua::Error> {
    let table = lua.create_table()?;
    table.set(
        "game_volume",
        serde_plain::to_string(&env.game_volume).unwrap(),
    )?;
    table.set("exe_sha256", hex::encode(&env.exe_sha256))?;
    Ok(mlua::Value::Table(table))
}

pub fn new<'a>(
    lua: &'a mlua::Lua,
    env: &mods::Env,
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
                let path = path::ensure_safe(&std::path::PathBuf::from_str(&path).unwrap())
                    .ok_or_else(|| anyhow::anyhow!("cannot read files outside of mod directory"))
                    .map_err(|e| e.into_lua_err())?;
                Ok(lua.create_string(&std::fs::read(mod_path.join(path))?)?)
            }
        })?,
    )?;

    table.set("ENV", new_env(lua, env)?)?;

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
