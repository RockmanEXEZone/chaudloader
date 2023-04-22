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

struct ExeDat(std::sync::Arc<std::sync::Mutex<assets::zipdat::Overlay>>);

impl mlua::UserData for ExeDat {
    fn add_methods<'lua, M: mlua::UserDataMethods<'lua, Self>>(methods: &mut M) {
        methods.add_method("read_file", |lua, this, (path,): (String,)| {
            let mut overlay = this.0.lock().unwrap();
            Ok(Some(lua.create_string(
                &overlay.read(&path).map_err(|e| e.to_lua_err())?.to_vec(),
            )?))
        });

        methods.add_method(
            "write_file",
            |lua, this, (path, contents): (String, mlua::String)| {
                let mut overlay = this.0.lock().unwrap();
                overlay
                    .write(&path, contents.as_bytes().to_vec())
                    .map_err(|e| e.to_lua_err())?;
                Ok(())
            },
        );
    }
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

    table.set(
        "read_mod_file",
        lua.create_function({
            let mod_path = std::path::Path::new("mods").join(mod_name);
            move |lua, (path,): (String,)| {
                let path = ensure_path_is_safe(&std::path::PathBuf::from_str(&path).unwrap())
                    .ok_or_else(|| anyhow::anyhow!("cannot read files outside of mod directory"))
                    .map_err(|e| e.to_lua_err())?;
                Ok(lua.create_string(&std::fs::read(mod_path.join(path))?)?)
            }
        })?,
    )?;

    table.set(
        "ExeDat",
        lua.create_function({
            move |lua, (name,): (String,)| {
                let overlay = if let Some(overlay) = overlays.get(&name) {
                    std::sync::Arc::clone(&overlay)
                } else {
                    return Err(anyhow::format_err!("no such dat file: {}", name).to_lua_err());
                };
                Ok(ExeDat(overlay))
            }
        })?,
    )?;

    Ok(table)
}
