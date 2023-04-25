use crate::path;
use mlua::ExternalError;

pub fn new<'a>(
    lua: &'a mlua::Lua,
    mod_path: &std::path::Path,
) -> Result<mlua::Value<'a>, mlua::Error> {
    let table = lua.create_table()?;

    table.set(
        "read_file",
        lua.create_function({
            let mod_path = mod_path.to_path_buf();
            move |lua, (path,): (String,)| {
                let path = path::ensure_safe(std::path::Path::new(&path))
                    .ok_or_else(|| anyhow::anyhow!("cannot read files outside of mod directory"))
                    .map_err(|e| e.into_lua_err())?;
                Ok(lua.create_string(&std::fs::read(mod_path.join(path))?)?)
            }
        })?,
    )?;

    table.set(
        "list_directory",
        lua.create_function({
            let mod_path = mod_path.to_path_buf();
            move |lua, (path,): (String,)| {
                let path = path::ensure_safe(std::path::Path::new(&path))
                    .ok_or_else(|| anyhow::anyhow!("cannot read files outside of mod directory"))
                    .map_err(|e| e.into_lua_err())?;
                Ok(lua.create_sequence_from(
                    std::fs::read_dir(mod_path.join(path))?
                        .into_iter()
                        .map(|entry| {
                            let path = entry?.path();
                            let file_name = path.file_name().ok_or_else(|| {
                                anyhow::format_err!("failed to decipher path: {}", path.display())
                            })?;
                            Ok(file_name
                                .to_str()
                                .ok_or_else(|| {
                                    anyhow::format_err!(
                                        "failed to decipher file name: {}",
                                        file_name.to_string_lossy()
                                    )
                                })?
                                .to_string())
                        })
                        .collect::<Result<Vec<_>, anyhow::Error>>()
                        .map_err(|e| e.into_lua_err())?,
                )?)
            }
        })?,
    )?;

    table.set(
        "get_file_metadata",
        lua.create_function({
            let mod_path = mod_path.to_path_buf();
            move |lua, (path,): (String,)| {
                let path = path::ensure_safe(std::path::Path::new(&path))
                    .ok_or_else(|| anyhow::anyhow!("cannot read files outside of mod directory"))
                    .map_err(|e| e.into_lua_err())?;
                let metadata = std::fs::metadata(mod_path.join(path))?;
                Ok(lua.create_table_from([
                    (
                        "type",
                        mlua::Value::String(lua.create_string(if metadata.is_dir() {
                            "dir"
                        } else {
                            "file"
                        })?),
                    ),
                    ("size", mlua::Value::Integer(metadata.len() as i64)),
                ])?)
            }
        })?,
    )?;

    Ok(mlua::Value::Table(table))
}
