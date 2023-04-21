use crate::{assets, mods};

mod bnlc_mod_loader_lib;

fn set_globals(
    lua: &mlua::Lua,
    mod_name: &str,
    overlays: std::sync::Arc<
        std::sync::Mutex<std::collections::HashMap<String, assets::zipdat::Overlay>>,
    >,
) -> Result<(), mlua::Error> {
    let globals = lua.globals();

    globals.set(
        "print",
        lua.create_function({
            let mod_name = mod_name.to_string();
            move |lua, args: mlua::Variadic<mlua::Value>| {
                log::info!(
                    "[mod: {}] {}",
                    mod_name,
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
        bnlc_mod_loader_lib::new(&lua, mod_name, overlays)?,
    )?;

    Ok(())
}

pub fn new(
    mod_name: &str,
    _mod_info: &mods::Info,
    overlays: std::sync::Arc<
        std::sync::Mutex<std::collections::HashMap<String, assets::zipdat::Overlay>>,
    >,
) -> Result<mlua::Lua, mlua::Error> {
    let lua = mlua::Lua::new();
    set_globals(&lua, &mod_name, overlays)?;
    Ok(lua)
}
