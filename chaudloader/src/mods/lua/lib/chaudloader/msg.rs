use crate::{assets, mods::lua::lib::chaudloader::buffer::Buffer};

pub fn new<'a>(lua: &'a mlua::Lua) -> Result<mlua::Value<'a>, mlua::Error> {
    let table = lua.create_table()?;

    table.set(
        "unpack",
        lua.create_function(|_, (raw,): (mlua::UserDataRef<Buffer>,)| {
            Ok(assets::msg::unpack(std::io::Cursor::new(raw.as_slice()))?
                .into_iter()
                .map(|v| Buffer::new(v))
                .collect::<Vec<_>>())
        })?,
    )?;

    table.set(
        "pack",
        lua.create_function(|_, (entries,): (Vec<mlua::UserDataRef<Buffer>>,)| {
            let mut buf = vec![];
            let entries = entries.iter().map(|v| v.as_slice()).collect::<Vec<_>>();
            assets::msg::pack(&entries, &mut buf)?;
            Ok(Buffer::new(buf))
        })?,
    )?;

    Ok(mlua::Value::Table(table))
}
