use crate::{assets, mods};

pub mod bnlc_mod_loader;
pub mod chaudloader;

pub fn set_globals(
    lua: &mlua::Lua,
    name: &str,
    state: std::rc::Rc<std::cell::RefCell<mods::State>>,
    overlays: std::collections::HashMap<
        String,
        std::rc::Rc<std::cell::RefCell<assets::exedat::Overlay>>,
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
        bnlc_mod_loader::new(&lua, name, overlays.clone())?,
    )?;

    globals.set(
        "chaudloader",
        chaudloader::new(&lua, name, state, overlays)?,
    )?;

    Ok(())
}
