use crate::assets;

struct Mpak(std::rc::Rc<std::cell::RefCell<assets::mpak::Mpak>>);

impl mlua::UserData for Mpak {
    fn add_methods<'lua, M: mlua::UserDataMethods<'lua, Self>>(methods: &mut M) {
        methods.add_meta_method(
            mlua::MetaMethod::NewIndex,
            |_, this, (rom_addr, contents): (u32, Option<mlua::String>)| {
                let mut this: std::cell::RefMut<assets::mpak::Mpak> = this.0.borrow_mut();
                if let Some(contents) = contents {
                    this.insert(rom_addr, contents.as_bytes().to_vec());
                } else {
                    this.remove(rom_addr);
                }
                Ok(())
            },
        );

        methods.add_meta_method(mlua::MetaMethod::Index, |lua, this, (rom_addr,): (u32,)| {
            let this = this.0.borrow();
            let entry = if let Some(contents) = this.get(rom_addr) {
                contents
            } else {
                return Ok(None);
            };
            Ok(Some(lua.create_string(entry)?))
        });

        methods.add_method("to_raw", |lua, this, (): ()| {
            let this = this.0.borrow();
            let mut map_contents = vec![];
            let mut mpak_contents = vec![];
            this.write_into(&mut map_contents, &mut mpak_contents)?;
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
            Ok(Mpak(std::rc::Rc::new(std::cell::RefCell::new(
                assets::mpak::Mpak::read_from(
                    std::io::Cursor::new(map_contents.as_bytes()),
                    std::io::Cursor::new(mpak_contents.as_bytes()),
                )?,
            ))))
        }
    })?))
}
