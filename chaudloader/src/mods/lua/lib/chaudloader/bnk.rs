use crate::{assets, mods, path};
use mlua::ExternalError;

pub fn new<'a>(
    lua: &'a mlua::Lua,
    mod_path: &std::path::Path,
) -> Result<mlua::Value<'a>, mlua::Error> {
    let table = lua.create_table()?;

    table.set(
        "load_bnk",
        lua.create_function({
            let mod_path = mod_path.to_path_buf();
            move |_, (path,): (String,)| {
                let path = path::ensure_safe(std::path::Path::new(&path))
                .ok_or_else(|| anyhow::anyhow!("cannot read files outside of mod directory"))
                .map_err(|e| e.into_lua_err())?;

                let bnk_path = mod_path.join(&path);
                if !bnk_path.exists() {
                    return Err(anyhow::anyhow!("{} does not exist", bnk_path.display()).into_lua_err());
                }

                let base_filename = bnk_path.file_name().unwrap().to_str().unwrap();
                const INVALID_BNK_NAMES: &[&str] = &["Init.bnk","Global.bnk","Vol1Global.bnk","Vol2Global.bnk","Voice1.bnk","DLC1.bnk","DLC2.bnk","EXE1.bnk","EXE2.bnk","EXE3.bnk","EXE4.bnk","EXE5.bnk","EXE6.bnk", "chaudloader.bnk"];
                if INVALID_BNK_NAMES.contains(&base_filename) {
                    Err(anyhow::anyhow!("cannot use the names: Init.bnk, Global.bnk, Vol1Global.bnk, Vol2Global.bnk, Voice1.bnk, DLC1.bnk, DLC2.bnk, EXE1.bnk, EXE2.bnk, EXE3.bnk, EXE4.bnk, EXE5.bnk, or EXE6.bnk").into_lua_err())
                }
                else {
                    let mut mod_audio = mods::MODAUDIOFILES.get().unwrap().lock().unwrap();
                    // Check if a bnk with this file name is already being loaded because the bnk load function only takes the base file name.
                    let base_filename_osstr = std::ffi::OsString::from(base_filename);
                    if mod_audio.bnks.contains(&base_filename_osstr) {
                        Err(anyhow::anyhow!("a bnk file named {} is already being loaded", base_filename).into_lua_err())
                    } else {
                        // The game will only try to load bnk files from the audio folder.
                        // Use the asset replacer to reroute it to the mod's folder.
                        let dst_bnk_path = std::path::PathBuf::from("..\\exe\\audio").join(base_filename);
                        let mut assets_replacer = assets::REPLACER.get().unwrap().lock().unwrap();
                        assets_replacer.add_path(&dst_bnk_path, &bnk_path);
                        mod_audio.bnks.push(base_filename_osstr);
                        Ok(())
                    }
                }
            }
        })?,
    )?;

    Ok(mlua::Value::Table(table))
}
