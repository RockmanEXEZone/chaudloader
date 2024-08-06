use crate::{mods, path, assets};
use mlua::ExternalError;

fn replace_wem(hash: u32, wem_path: std::path::PathBuf, language_id: u32) -> Result<(), mlua::Error> {
    if !wem_path.exists() {
        return Err(anyhow::anyhow!("{} does not exist", wem_path.display()).into_lua_err());
    }
    let mut mod_audio = mods::MODAUDIOFILES.get().unwrap().lock().unwrap();
    if let Some(old_replacement) = mod_audio.wems.insert(hash, mods::WemFile{
        path: wem_path,
        language_id: language_id,
    }) {
        log::warn!("{} is already replaced with {}. Replacing again with {}.", &hash, old_replacement.path.display(), mod_audio.wems[&hash].path.display());
    }
    return Ok(());
}

pub fn new<'a>(
    lua: &'a mlua::Lua,
    mod_path: &std::path::Path,
) -> Result<mlua::Value<'a>, mlua::Error> {
    let table = lua.create_table()?;

    table.set(
        "load_pck",
        lua.create_function({
            let mod_path = mod_path.to_path_buf();
            move |_, (path,): (String,)| {
                match path.as_str() {
                    "Vol1.pck" | "Vol2.pck" | "DLC1.pck" | "DLC2.pck" | "chaudloader.pck" => {
                        return Err(anyhow::anyhow!("cannot use the names: Vol1.pck, Vol2.pck, DLC1.pck, DLC2.pck, or chaudloader.pck").into_lua_err());
                    },
                    _ => {
                        let path = path::ensure_safe(std::path::Path::new(&path))
                            .ok_or_else(|| anyhow::anyhow!("cannot read files outside of mod directory"))
                            .map_err(|e| e.into_lua_err())?;

                        let pck_path = mod_path.join(&path);
                        if !pck_path.exists() {
                            return Err(anyhow::anyhow!("{} does not exist", pck_path.display()).into_lua_err());
                        }
                        let mut mod_audio = mods::MODAUDIOFILES.get().unwrap().lock().unwrap();
                        // Check if a pck with this file name is already being loaded because the pck load function only takes the base file name.
                        let base_filename = pck_path.file_name().unwrap().to_os_string();
                        if mod_audio.pcks.contains(&base_filename) {
                            return Err(anyhow::anyhow!("a pck file named {} is already being loaded", base_filename.to_str().unwrap()).into_lua_err());
                        } else {
                            // Copy this pck to the audio folder so it can be loaded
                            let dst_pck_path = std::path::PathBuf::from("..\\exe\\audio").join(&base_filename);
                            let mut assets_replacer = assets::REPLACER.get().unwrap().lock().unwrap();
                            assets_replacer.add(
                                &dst_pck_path,
                                move |writer| {
                                    writer.write_all(std::fs::read(&pck_path)?.as_slice())
                                });
                            mod_audio.pcks.push(base_filename);
                            return Ok(());
                        }
                    }
                }
            }
        })?,
    )?;

    table.set(
        "replace_wem",
        lua.create_function({
            let mod_path = mod_path.to_path_buf();
            move |_, (hash, path, ): (u32, String,)| {
                let path = path::ensure_safe(std::path::Path::new(&path))
                .ok_or_else(|| anyhow::anyhow!("cannot read files outside of mod directory"))
                .map_err(|e| e.into_lua_err())?;
        
            let wem_path = mod_path.join(&path);
                replace_wem(hash, wem_path, 0)
            }
        })?,
    )?;

    table.set(
        "replace_wem_language",
        lua.create_function({
            let mod_path = mod_path.to_path_buf();
            move |_, (hash, path, language_id): (u32, String, u32)| {
                let path = path::ensure_safe(std::path::Path::new(&path))
                .ok_or_else(|| anyhow::anyhow!("cannot read files outside of mod directory"))
                .map_err(|e| e.into_lua_err())?;
        
            let wem_path = mod_path.join(&path);
                replace_wem(hash, wem_path, language_id)
            }
        })?,
    )?;

    Ok(mlua::Value::Table(table))
}
