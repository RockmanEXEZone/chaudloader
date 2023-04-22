mod exedat;
mod mpak;

use crate::{assets, mods};
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
    name: &'a str,
    state: std::sync::Arc<std::sync::Mutex<mods::State>>,
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
                let path = ensure_path_is_safe(&std::path::PathBuf::from_str(&path).unwrap())
                    .ok_or_else(|| anyhow::anyhow!("cannot read files outside of mod directory"))
                    .map_err(|e| e.to_lua_err())?;
                Ok(lua.create_string(&std::fs::read(mod_path.join(path))?)?)
            }
        })?,
    )?;

    table.set(
        "init_mod_dll",
        lua.create_function({
            let mod_path = mod_path.clone();
            let state = std::sync::Arc::clone(&state);
            move |_, (path, buf): (String, mlua::String)| {
                let path = ensure_path_is_safe(&std::path::PathBuf::from_str(&path).unwrap())
                    .ok_or_else(|| anyhow::anyhow!("cannot read files outside of mod directory"))
                    .map_err(|e| e.to_lua_err())?;
                let mut state = state.lock().unwrap();
                type ChaudLoaderInitFn =
                    unsafe extern "system" fn(userdata: *const u8, n: usize) -> bool;
                let dll = unsafe {
                    let dll = windows_libloader::ModuleHandle::load(&mod_path.join(&path))
                        .ok_or_else(|| anyhow::anyhow!("DLL {} failed to load", path.display()))
                        .map_err(|e| e.to_lua_err())?;
                    let init_fn = std::mem::transmute::<_, ChaudLoaderInitFn>(
                        dll.get_symbol_address("ChaudLoaderInit")
                            .ok_or_else(|| {
                                anyhow::anyhow!(
                                    "ChaudLoaderInit not found in DLL {}",
                                    path.display()
                                )
                            })
                            .map_err(|e| e.to_lua_err())?,
                    );
                    let buf = buf.as_bytes();
                    if !init_fn(buf.as_ptr(), buf.len()) {
                        return Err(anyhow::anyhow!(
                            "ChaudLoaderInit for DLL {} returned false",
                            path.display()
                        )
                        .to_lua_err());
                    }
                    dll
                };
                state.add_dll(path, dll);
                Ok(())
            }
        })?,
    )?;

    table.set("ExeDat", exedat::new(lua, overlays)?)?;
    table.set("Mpak", mpak::new(lua)?)?;

    Ok(mlua::Value::Table(table))
}
