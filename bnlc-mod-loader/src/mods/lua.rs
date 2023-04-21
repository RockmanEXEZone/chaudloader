use std::{io::Read, str::FromStr};

use mlua::ExternalError;

use crate::assets;

fn set_globals(
    lua: &mlua::Lua,
    mod_name: &std::ffi::OsStr,
    overlays: std::sync::Arc<
        std::sync::Mutex<std::collections::HashMap<String, assets::dat::Overlay>>,
    >,
) -> Result<(), mlua::Error> {
    let globals = lua.globals();

    globals.set(
        "print",
        lua.create_function({
            let mod_name = mod_name.to_os_string();
            move |lua, args: mlua::Variadic<mlua::Value>| {
                log::info!(
                    "[mod: {}] {}",
                    mod_name.to_string_lossy(),
                    args.iter()
                        .map(|v| lua
                            .coerce_string(v.clone())
                            .ok()
                            .flatten()
                            .map(|v| v.to_string_lossy().to_string())
                            .unwrap_or_else(|| format!("[{}]", v.type_name())))
                        .collect::<Vec<_>>()
                        .join(" ")
                );
                Ok(())
            }
        })?,
    )?;

    globals.set(
        "bnlc_mod_loader",
        make_bnlc_mod_loader_table(&lua, mod_name, overlays)?,
    )?;

    Ok(())
}

fn make_bnlc_mod_loader_table<'a>(
    lua: &'a mlua::Lua,
    mod_name: &'a std::ffi::OsStr,
    overlays: std::sync::Arc<
        std::sync::Mutex<std::collections::HashMap<String, assets::dat::Overlay>>,
    >,
) -> Result<mlua::Table<'a>, mlua::Error> {
    let table = lua.create_table()?;

    let mod_path = std::path::Path::new("mods").join(mod_name);

    table.set(
        "write_dat_contents",
        lua.create_function({
            let overlays = std::sync::Arc::clone(&overlays);
            move |_, (dat_filename, asset_filename, contents): (String, String, mlua::String)| {
                let mut overlays = overlays.lock().unwrap();
                let overlay = if let Some(overlay) = overlays.get_mut(&dat_filename) {
                    overlay
                } else {
                    return Err(
                        anyhow::format_err!("no such dat file: {}", dat_filename).to_lua_err()
                    );
                };
                overlay
                    .write(&asset_filename, contents.as_bytes().to_vec())
                    .map_err(|e| e.to_lua_err())?;
                Ok(())
            }
        })?,
    )?;

    table.set(
        "read_dat_contents",
        lua.create_function({
            let overlays = std::sync::Arc::clone(&overlays);
            move |lua, (dat_filename, asset_filename): (String, String)| {
                let mut overlays = overlays.lock().unwrap();
                let overlay = if let Some(overlay) = overlays.get_mut(&dat_filename) {
                    overlay
                } else {
                    return Ok(None);
                };

                Ok(Some(
                    lua.create_string(
                        &overlay
                            .read(&asset_filename)
                            .map_err(|e| e.to_lua_err())?
                            .to_vec(),
                    )?,
                ))
            }
        })?,
    )?;

    table.set(
        "read_mod_file",
        lua.create_function({
            let mod_path = mod_path.clone();
            move |lua, (path,): (String,)| {
                let path = std::path::PathBuf::from_str(&path).unwrap();

                // TODO: Do this in a less janky way.
                if path
                    .components()
                    .into_iter()
                    .any(|x| x == std::path::Component::ParentDir)
                {
                    return Err(
                        anyhow::anyhow!("cannot read files outside of mod directory").to_lua_err(),
                    );
                }

                let real_path = mod_path.join(path);

                let mut f = std::fs::File::open(real_path)?;
                let mut buf = vec![];
                f.read_to_end(&mut buf)?;

                Ok(lua.create_string(&buf)?)
            }
        })?,
    )?;

    Ok(table)
}

pub fn new(
    mod_name: std::ffi::OsString,
    overlays: std::sync::Arc<
        std::sync::Mutex<std::collections::HashMap<String, assets::dat::Overlay>>,
    >,
) -> Result<mlua::Lua, mlua::Error> {
    let lua = mlua::Lua::new();
    set_globals(&lua, &mod_name, overlays)?;
    Ok(lua)
}
