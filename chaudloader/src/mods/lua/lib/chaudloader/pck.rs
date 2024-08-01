use crate::{mods, path};
use mlua::ExternalError;

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
                    "Vol1.pck" | "Vol2.pck" | "DLC1.pck" | "DLC2.pck" => {
                        return Err(anyhow::anyhow!("cannot use the names Vol1.pck, Vol2.pck, DLC1.pck, or DLC2.pck").into_lua_err());
                    },
                    _ => {
                        let path = path::ensure_safe(std::path::Path::new(&path))
                        .ok_or_else(|| anyhow::anyhow!("cannot read files outside of mod directory"))
                        .map_err(|e| e.into_lua_err())?;

                        let pck_path = mod_path.join(&path);
                        if !pck_path.exists() {
                            return Err(anyhow::anyhow!("{} does not exist", pck_path.to_str().unwrap()).into_lua_err())
                        }
                        // Copy this pck to the audio folder so it can be loaded
                        let base_filename = pck_path.file_name().unwrap();
                        let dst_pck_path = std::path::PathBuf::from("audio").join(base_filename);
                        std::fs::copy(&pck_path, &dst_pck_path)
                        .map_err(|e| e.into_lua_err())?;

                        let mut mod_audio = mods::MODAUDIOFILES.get().unwrap().lock().unwrap();
                        mod_audio.pcks.push(base_filename.to_os_string());
                        return Ok(());
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
                if !wem_path.exists() {
                    return Err(anyhow::anyhow!("{} does not exist", wem_path.to_str().unwrap()).into_lua_err())
                }
                let mut mod_audio = mods::MODAUDIOFILES.get().unwrap().lock().unwrap();
                if mod_audio.wems.insert(hash, wem_path).is_some() {
                    log::warn!("{} is already replaced. Replacing again.", &hash);
                }
                return Ok(());
            }
        })?,
    )?;

    Ok(mlua::Value::Table(table))
}
