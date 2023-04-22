use crate::assets;
use mlua::ExternalError;
use std::str::FromStr;

fn ensure_path_is_safe(path: &std::path::Path) -> Option<std::path::PathBuf> {
    let path = clean_path::clean(path);

    match path.components().next() {
        Some(std::path::Component::ParentDir)
        | Some(std::path::Component::RootDir)
        | Some(std::path::Component::Prefix(..)) => {
            return None;
        }
        _ => {}
    }

    Some(path)
}

pub fn new<'a>(
    lua: &'a mlua::Lua,
    mod_name: &'a str,
    overlays: std::collections::HashMap<
        String,
        std::sync::Arc<std::sync::Mutex<assets::zipdat::Overlay>>,
    >,
) -> Result<mlua::Table<'a>, mlua::Error> {
    let table = lua.create_table()?;
    let overlays = std::sync::Arc::new(overlays);

    let mod_path = std::path::Path::new("mods").join(mod_name);

    table.set(
        "write_exe_dat_contents",
        lua.create_function({
            let overlays = std::sync::Arc::clone(&overlays);
            move |_, (dat_filename, path, contents): (String, String, mlua::String)| {
                let overlay = overlays
                    .get(&dat_filename)
                    .ok_or_else(|| anyhow::format_err!("no such dat file: {}", dat_filename))
                    .map_err(|e| e.to_lua_err())?;
                let mut overlay = overlay.lock().unwrap();
                overlay
                    .write(&path, contents.as_bytes().to_vec())
                    .map_err(|e| e.to_lua_err())?;
                Ok(())
            }
        })?,
    )?;

    table.set(
        "read_exe_dat_contents",
        lua.create_function({
            let overlays = std::sync::Arc::clone(&overlays);
            move |lua: &mlua::Lua, (dat_filename, path): (String, String)| {
                let overlay = overlays
                    .get(&dat_filename)
                    .ok_or_else(|| anyhow::format_err!("no such dat file: {}", dat_filename))
                    .map_err(|e| e.to_lua_err())?;
                let mut overlay = overlay.lock().unwrap();
                Ok(Some(lua.create_string(
                    &overlay.read(&path).map_err(|e| e.to_lua_err())?.to_vec(),
                )?))
            }
        })?,
    )?;

    table.set(
        "read_mod_contents",
        lua.create_function({
            let mod_path = mod_path.clone();
            move |lua, (path,): (String,)| {
                let path = ensure_path_is_safe(&std::path::PathBuf::from_str(&path).unwrap())
                    .ok_or_else(|| anyhow::anyhow!("cannot read files outside of mod directory"))
                    .map_err(|e| e.to_lua_err())?;
                Ok(lua.create_string(&std::fs::read(mod_path.join(path))?)?)
            }
        })?,
    )?;

    Ok(table)
}
