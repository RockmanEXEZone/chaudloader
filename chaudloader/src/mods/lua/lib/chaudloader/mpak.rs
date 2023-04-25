use crate::{assets, mods::lua::lib::chaudloader::buffer::Buffer};

struct Mpak(std::rc::Rc<std::cell::RefCell<assets::mpak::Mpak>>);

impl mlua::UserData for Mpak {
    fn add_methods<'lua, M: mlua::UserDataMethods<'lua, Self>>(methods: &mut M) {
        methods.add_meta_method(
            mlua::MetaMethod::NewIndex,
            |_, this, (rom_addr, contents): (u32, Option<mlua::UserDataRef<Buffer>>)| {
                let mut this: std::cell::RefMut<assets::mpak::Mpak> = this.0.borrow_mut();
                if let Some(contents) = contents {
                    this.insert(rom_addr, contents.as_slice().to_vec());
                } else {
                    this.remove(rom_addr);
                }
                Ok(())
            },
        );

        methods.add_meta_method(mlua::MetaMethod::Index, |_, this, (rom_addr,): (u32,)| {
            let this = this.0.borrow();
            let entry = if let Some(contents) = this.get(rom_addr) {
                contents
            } else {
                return Ok(None);
            };
            Ok(Some(Buffer::new(entry.to_vec())))
        });

        methods.add_meta_method(mlua::MetaMethod::Pairs, |lua, this, (): ()| {
            Ok(lua.create_function({
                let i = std::rc::Rc::new(std::cell::RefCell::new(0usize));
                let this = std::rc::Rc::clone(&this.0);
                move |_, (): ()| {
                    let mut i = i.borrow_mut();
                    let this = this.borrow();
                    let (k, v) = if let Some((k, v)) = this.get_index(*i) {
                        (k, v)
                    } else {
                        return Ok((None, None));
                    };
                    *i = *i + 1;
                    Ok((Some(k), Some(Buffer::new(v.to_vec()))))
                }
            })?)
        });

        methods.add_method("pack", |_, this, (): ()| {
            let this = this.0.borrow();
            let mut map_contents = vec![];
            let mut mpak_contents = vec![];
            this.write_into(&mut map_contents, &mut mpak_contents)?;
            Ok((
                Buffer::new(map_contents.to_vec()),
                Buffer::new(mpak_contents.to_vec()),
            ))
        });
    }
}

pub fn new<'a>(lua: &'a mlua::Lua) -> Result<mlua::Value<'a>, mlua::Error> {
    let table = lua.create_table()?;

    table.set(
        "unpack",
        lua.create_function({
            move |_,
                  (map_contents, mpak_contents): (
                mlua::UserDataRef<Buffer>,
                mlua::UserDataRef<Buffer>,
            )| {
                Ok(Mpak(std::rc::Rc::new(std::cell::RefCell::new(
                    assets::mpak::Mpak::read_from(
                        std::io::Cursor::new(map_contents.as_slice()),
                        std::io::Cursor::new(mpak_contents.as_slice()),
                    )?,
                ))))
            }
        })?,
    )?;

    Ok(mlua::Value::Table(table))
}
