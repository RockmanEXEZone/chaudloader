use crate::{assets, mods, path};
use mlua::ExternalError;

pub mod chaudloader;

fn load<'lua>(
    lua: &'lua mlua::Lua,
    state: &mut mods::State,
    r#unsafe: bool,
    name: &str,
    mod_path: &std::path::Path,
    path: &std::path::Path,
) -> Result<mlua::Value<'lua>, anyhow::Error> {
    let path = path::ensure_safe(path)
        .ok_or_else(|| anyhow::anyhow!("cannot read files outside of mod directory"))
        .map_err(|e| e.into_lua_err())?;

    let full_path = mod_path.join(&path);

    let extension = path.extension().map(|v| v.to_string_lossy());
    Ok(
        if extension.as_ref().map(|ext| ext == "lua").unwrap_or(false) {
            lua.load(&std::fs::read_to_string(&full_path)?)
                .set_name(&format!("={}", path.display()))
                .set_mode(mlua::ChunkMode::Text)
                .call::<_, mlua::Value>(())?
        } else if extension.as_ref().map(|ext| ext == "dll").unwrap_or(false) {
            std::fs::metadata(&full_path)?;

            if !r#unsafe {
                return Err(anyhow::anyhow!(
                    "in order to load DLLs, you must mark your mod as unsafe!",
                ));
            }

            let luaopen = unsafe {
                let mh = windows_libloader::ModuleHandle::load(&full_path)
                    .map_err(|e| e.into_lua_err())?;
                let symbol_name = format!("luaopen_{}", name.replace(".", "_"));
                let luaopen = std::mem::transmute::<_, mlua::lua_CFunction>(
                    mh.get_symbol_address(&symbol_name).map_err(|e| {
                        anyhow::format_err!("failed to find symbol {}: {}", symbol_name, e)
                            .into_lua_err()
                    })?,
                );
                state.dlls.insert(full_path, mh);
                lua.create_c_function(luaopen)?
            };

            luaopen.call(())?
        } else if let Some(extension) = extension {
            return Err(anyhow::anyhow!("unknown file type: {}", extension));
        } else {
            return Err(anyhow::anyhow!("unknown file type with no extension"));
        },
    )
}

pub fn set_globals(
    lua: &mlua::Lua,
    game_env: &mods::GameEnv,
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
                let mut state = state.borrow_mut();

                let path = std::path::Path::new(&name).to_path_buf();
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

                let v = match (|| {
                    const EXTENSIONS: &[&str] = &["lua", "dll"];

                    let mut errs: Vec<(std::path::PathBuf, String, anyhow::Error)> = vec![];

                    if path
                        .extension()
                        .as_ref()
                        .map(|ext| ext.to_string_lossy())
                        .map(|ext| EXTENSIONS.contains(&ext.as_ref()))
                        .unwrap_or(false)
                    {
                        // Try load via exact path.
                        if let Some(filename) = {
                            let mut path = path.clone();
                            path.set_extension("");
                            path
                        }
                        .file_name()
                        {
                            let name = filename.to_str().unwrap().to_string();

                            match load(lua, &mut state, r#unsafe, &name, &mod_path, &path) {
                                Ok(v) => return Ok(v),
                                Err(err) => {
                                    errs.push((path.clone(), name.clone(), err));
                                }
                            }
                        } else {
                            errs.push((
                                path.clone(),
                                "".to_string(),
                                anyhow::anyhow!("could not determine module name for {}", name),
                            ));
                        }
                    }

                    if let Some(filename) = path.file_name() {
                        // Try load via short name.
                        let name = filename.to_str().unwrap().to_string();

                        for ext in EXTENSIONS {
                            let mut path = path.clone();
                            path.as_mut_os_string().push(".");
                            path.as_mut_os_string().push(ext);

                            match load(lua, &mut state, r#unsafe, &name, &mod_path, &path) {
                                Ok(v) => return Ok(v),
                                Err(err) => {
                                    errs.push((path.clone(), name.clone(), err));
                                }
                            }
                        }
                    } else {
                        errs.push((
                            path.clone(),
                            "".to_string(),
                            anyhow::anyhow!("could not determine module name for {}", name),
                        ));
                    }

                    if path
                        .parent()
                        .map(|v| v.as_os_str().is_empty())
                        .unwrap_or(false)
                    {
                        // Try load via dotted.
                        let path = std::path::Path::new(&name.replace(".", "/")).to_path_buf();

                        for ext in EXTENSIONS {
                            let mut path = path.clone();
                            path.as_mut_os_string().push(".");
                            path.as_mut_os_string().push(ext);

                            match load(lua, &mut state, r#unsafe, &name, &mod_path, &path) {
                                Ok(v) => return Ok(v),
                                Err(err) => {
                                    errs.push((path.clone(), name.clone(), err));
                                }
                            }
                        }
                    }

                    Err(anyhow::format_err!(
                        "failed to load package. tried:\n{}",
                        errs.into_iter()
                            .map(|(path, name, err)| format!(
                                " - {} (package name: {}): {}",
                                path.display(),
                                name,
                                err
                            ))
                            .collect::<Vec<_>>()
                            .join("\n")
                    ))
                })() {
                    Ok(v) => v,
                    Err(e) => {
                        return Err(e.into_lua_err());
                    }
                };

                let v = match v.clone() {
                    mlua::Value::Nil => mlua::Value::Boolean(true),
                    v => v,
                };

                loaded_modules.raw_set(cache_key, v.clone())?;

                Ok(v)
            }
        })?,
    )?;

    globals.set(
        "chaudloader",
        chaudloader::new(&lua, game_env, name, info, state, overlays)?,
    )?;

    lua.load(include_str!("compat.lua"))
        .set_name("=<builtin>\\compat.lua")
        .set_mode(mlua::ChunkMode::Text)
        .exec()?;

    Ok(())
}
