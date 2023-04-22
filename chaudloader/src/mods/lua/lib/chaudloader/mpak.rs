use crate::assets;

struct Mpak(assets::mpak::Mpak);

impl mlua::UserData for Mpak {
    fn add_methods<'lua, M: mlua::UserDataMethods<'lua, Self>>(methods: &mut M) {
        methods.add_method_mut(
            "insert",
            |_, this, (rom_addr, contents): (u32, mlua::String)| {
                this.0.insert(rom_addr, contents.as_bytes().to_vec());
                Ok(())
            },
        );

        methods.add_method("get", |lua, this, (rom_addr,): (u32,)| {
            let entry = if let Some(contents) = this.0.get(rom_addr) {
                contents
            } else {
                return Ok(None);
            };
            Ok(Some(lua.create_string(entry)?))
        });

        methods.add_method("to_raw", |lua, this, (): ()| {
            let mut map_contents = vec![];
            let mut mpak_contents = vec![];
            this.0.write_into(&mut map_contents, &mut mpak_contents)?;
            Ok((
                lua.create_string(&map_contents)?,
                lua.create_string(&mpak_contents)?,
            ))
        });
    }
}

pub fn new<'a>(lua: &'a mlua::Lua) -> Result<mlua::Value<'a>, mlua::Error> {
    Ok(mlua::Value::Function(lua.create_function({
        move |_, (map_contents, mpak_contents): (mlua::String, mlua::String)| {
            Ok(Mpak(assets::mpak::Mpak::read_from(
                std::io::Cursor::new(map_contents.as_bytes()),
                std::io::Cursor::new(mpak_contents.as_bytes()),
            )?))
        }
    })?))
}
