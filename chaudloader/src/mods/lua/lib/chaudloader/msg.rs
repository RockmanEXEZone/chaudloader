use crate::assets;

pub fn new<'a>(lua: &'a mlua::Lua) -> Result<mlua::Value<'a>, mlua::Error> {
    let table = lua.create_table()?;

    table.set(
        "unpack",
        lua.create_function(|lua, (raw,): (mlua::String,)| {
            Ok(assets::msg::unpack(std::io::Cursor::new(raw.as_bytes()))?
                .into_iter()
                .map(|v| lua.create_string(&v))
                .collect::<Result<Vec<_>, _>>()?)
        })?,
    )?;

    table.set(
        "pack",
        lua.create_function(|lua, (entries,): (Vec<mlua::String>,)| {
            let mut buf = vec![];
            let entries = entries.iter().map(|v| v.as_bytes()).collect::<Vec<_>>();
            assets::msg::pack(&entries, &mut buf)?;
            Ok(lua.create_string(&buf)?)
        })?,
    )?;

    Ok(mlua::Value::Table(table))
}
