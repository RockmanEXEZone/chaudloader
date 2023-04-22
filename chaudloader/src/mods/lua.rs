use crate::{assets, mods};

mod bnlc_mod_loader_lib;
mod chaudloader_lib;

fn set_globals(
    lua: &mlua::Lua,
    name: &str,
    state: std::sync::Arc<std::sync::Mutex<mods::State>>,
    overlays: std::collections::HashMap<
        String,
        std::sync::Arc<std::sync::Mutex<assets::zipdat::Overlay>>,
    >,
) -> Result<(), mlua::Error> {
    let globals = lua.globals();

    globals.set(
        "print",
        lua.create_function({
            let name = name.to_string();
            move |lua, args: mlua::Variadic<mlua::Value>| {
                log::info!(
                    "[mod: {}] {}",
                    name,
                    args.iter()
                        .map(|v| lua
                            .coerce_string(v.clone())
                            .ok()
                            .flatten()
                            .map(|v| v.to_string_lossy().to_string())
                            .unwrap_or_else(|| format!("{}: {:p}", v.type_name(), v.to_pointer())))
                        .collect::<Vec<_>>()
                        .join("\t")
                );
                Ok(())
            }
        })?,
    )?;

    globals.set(
        "bnlc_mod_loader",
        bnlc_mod_loader_lib::new(&lua, name, overlays.clone())?,
    )?;

    globals.set(
        "chaudloader",
        chaudloader_lib::new(&lua, name, state, overlays)?,
    )?;

    Ok(())
}

pub fn new(
    name: &str,
    _info: &mods::Info,
    state: std::sync::Arc<std::sync::Mutex<mods::State>>,
    overlays: std::collections::HashMap<
        String,
        std::sync::Arc<std::sync::Mutex<assets::zipdat::Overlay>>,
    >,
) -> Result<mlua::Lua, mlua::Error> {
    let lua = mlua::Lua::new();
    set_globals(&lua, &name, state, overlays)?;
    Ok(lua)
}
