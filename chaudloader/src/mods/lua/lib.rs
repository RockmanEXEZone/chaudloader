use crate::{assets, mods, path};
use mlua::ExternalError;

pub mod bnlc_mod_loader;
pub mod chaudloader;

pub fn set_globals(
    lua: &mlua::Lua,
    env: &mods::Env,
    name: &str,
    info: &mods::Info,
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

    let loaded = lua.create_registry_value(lua.create_table()?)?;
    let mod_path = std::path::Path::new("mods").join(&name);

    globals.set(
        "require",
        lua.create_function({
            let state = std::rc::Rc::clone(&state);
            let mod_path = mod_path.clone();
            let r#unsafe = info.r#unsafe;
            move |lua, name: String| {
                let path = path::ensure_safe(std::path::Path::new(&name))
                    .ok_or_else(|| anyhow::anyhow!("cannot read files outside of mod directory"))
                    .map_err(|e| e.into_lua_err())?;

                let cache_key = path
                    .as_os_str()
                    .to_str()
                    .ok_or_else(|| {
                        anyhow::anyhow!("cannot deciper path: {}", path.display()).into_lua_err()
                    })?
                    .to_string();

                let loaded_modules = lua.registry_value::<mlua::Table>(&loaded)?;
                if let Some(v) = loaded_modules.raw_get(cache_key.clone())? {
                    return Ok(v);
                }

                let (mut source, mut source_name) = (None, std::path::PathBuf::new());
                for path in [path.clone(), {
                    let mut path = path.clone();
                    path.as_mut_os_string().push(".lua");
                    path
                }] {
                    match std::fs::read_to_string(&mod_path.join(&path)) {
                        Ok(s) => {
                            source = Some(s);
                            source_name = path.to_path_buf();
                        }
                        Err(e) if e.kind() == std::io::ErrorKind::NotFound => continue,
                        Err(e) => {
                            return Err(e.into());
                        }
                    }
                }

                let value = if let Some(source) = source {
                    lua.load(&source)
                        .set_name(&format!("={}", source_name.display()))
                        .set_mode(mlua::ChunkMode::Text)
                        .call::<_, mlua::Value>(())?
                } else if r#unsafe {
                    // Try load unsafe.
                    let lib_name = path
                        .file_name()
                        .ok_or_else(|| {
                            anyhow::anyhow!("cannot deciper library name: {}", path.display())
                                .into_lua_err()
                        })?
                        .to_str()
                        .ok_or_else(|| {
                            anyhow::anyhow!("cannot deciper path: {}", path.display())
                                .into_lua_err()
                        })?;

                    let mut state = state.borrow_mut();
                    let mut path = path.clone();
                    path.as_mut_os_string().push(".dll");

                    let luaopen = unsafe {
                        let mh = windows_libloader::ModuleHandle::load(&mod_path.join(&path))
                            .ok_or(mlua::Error::RuntimeError(format!(
                                "cannot find '{}' (also tried DLL)",
                                name
                            )))?;
                        let symbol_name = format!("luaopen_{}", lib_name);
                        let luaopen = std::mem::transmute::<_, mlua::lua_CFunction>(
                            mh.get_symbol_address(&symbol_name).ok_or(
                                mlua::Error::RuntimeError(format!(
                                    "cannot find symbol {} in {}",
                                    symbol_name,
                                    path.display()
                                )),
                            )?,
                        );
                        state.dlls.insert(path, mh);
                        lua.create_c_function(luaopen)?
                    };

                    luaopen.call(())?
                } else {
                    return Err(mlua::Error::RuntimeError(format!("cannot find '{}'", name)));
                };

                loaded_modules.raw_set(
                    cache_key,
                    match value.clone() {
                        mlua::Value::Nil => mlua::Value::Boolean(true),
                        v => v,
                    },
                )?;

                Ok(value)
            }
        })?,
    )?;

    globals.set(
        "bnlc_mod_loader",
        bnlc_mod_loader::new(&lua, name, overlays.clone())?,
    )?;

    globals.set(
        "chaudloader",
        chaudloader::new(&lua, env, name, info, state, overlays)?,
    )?;

    Ok(())
}
