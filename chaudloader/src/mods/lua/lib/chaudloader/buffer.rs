use std::io::Write;

use byteorder::{ByteOrder, WriteBytesExt};
use mlua::ExternalError;

#[derive(Copy, Clone)]
struct UQ16_16(u32);

impl UQ16_16 {
    fn from_f64(v: f64) -> Self {
        Self((v * (0x10000 as f64)) as u32)
    }

    fn from_u32(v: u32) -> Self {
        Self(v)
    }

    fn into_f64(self) -> f64 {
        (self.0 as f64) / (0x10000 as f64)
    }

    fn into_u32(self) -> u32 {
        self.0
    }
}

#[derive(Copy, Clone)]
struct IQ16_16(i32);

impl IQ16_16 {
    fn from_f64(v: f64) -> Self {
        Self((v * (0x10000 as f64)) as i32)
    }

    fn from_i32(v: i32) -> Self {
        Self(v)
    }

    fn into_f64(self) -> f64 {
        (self.0 as f64) / (0x10000 as f64)
    }

    fn into_i32(self) -> i32 {
        self.0
    }
}

pub struct Builder(Vec<u8>);

impl mlua::UserData for Builder {
    fn add_methods<'lua, M: mlua::UserDataMethods<'lua, Self>>(methods: &mut M) {
        methods.add_method("tell", |_, this, (): ()| Ok(this.0.len()));

        methods.add_method("build", |_, this, (): ()| Ok(Buffer::new(this.0.clone())));

        methods.add_method_mut("write", |_, this, (buf,): (mlua::UserDataRef<Buffer>,)| {
            this.0.write_all(&buf.borrow())?;
            Ok(())
        });

        methods.add_method_mut("write_string", |_, this, (s,): (mlua::String,)| {
            this.0.write_all(s.as_bytes())?;
            Ok(())
        });

        methods.add_method_mut("write_u8", |_, this, (v,): (u8,)| {
            this.0.write_u8(v)?;
            Ok(())
        });

        methods.add_method_mut("write_u16_le", |_, this, (v,): (u16,)| {
            this.0.write_u16::<byteorder::LittleEndian>(v)?;
            Ok(())
        });

        methods.add_method_mut("write_uq16_16_le", |_, this, (v,): (f64,)| {
            this.0
                .write_u32::<byteorder::LittleEndian>(UQ16_16::from_f64(v).into_u32())?;
            Ok(())
        });

        methods.add_method_mut("write_u32_le", |_, this, (v,): (u32,)| {
            this.0.write_u32::<byteorder::LittleEndian>(v)?;
            Ok(())
        });

        methods.add_method_mut("write_i8", |_, this, (v,): (i8,)| {
            this.0.write_i8(v)?;
            Ok(())
        });

        methods.add_method_mut("write_i16_le", |_, this, (v,): (i16,)| {
            this.0.write_i16::<byteorder::LittleEndian>(v)?;
            Ok(())
        });

        methods.add_method_mut("write_i32_le", |_, this, (v,): (i32,)| {
            this.0.write_i32::<byteorder::LittleEndian>(v)?;
            Ok(())
        });

        methods.add_method_mut("write_iq16_16_le", |_, this, (v,): (f64,)| {
            this.0
                .write_i32::<byteorder::LittleEndian>(IQ16_16::from_f64(v).into_i32())?;
            Ok(())
        });
    }
}

pub struct Buffer {
    vec: std::rc::Rc<std::cell::RefCell<Vec<u8>>>,
    range: std::ops::Range<usize>,
}

impl Buffer {
    pub fn new(v: Vec<u8>) -> Self {
        let len = v.len();
        Self {
            vec: std::rc::Rc::new(std::cell::RefCell::new(v)),
            range: 0..len,
        }
    }

    pub fn borrow<'a>(&'a self) -> std::cell::Ref<'a, [u8]> {
        std::cell::Ref::map(self.vec.borrow(), |v| &v[self.range.clone()])
    }

    pub fn borrow_mut<'a>(&'a self) -> std::cell::RefMut<'a, [u8]> {
        std::cell::RefMut::map(self.vec.borrow_mut(), |v| &mut v[self.range.clone()])
    }

    pub fn slice(&self, range: std::ops::Range<usize>) -> Option<Self> {
        self.vec.borrow().get(range.clone())?;
        Some(Self {
            vec: std::rc::Rc::clone(&self.vec),
            range,
        })
    }
}

impl mlua::UserData for Buffer {
    fn add_methods<'lua, M: mlua::UserDataMethods<'lua, Self>>(methods: &mut M) {
        methods.add_meta_method(
            mlua::MetaMethod::Concat,
            |_, this, (other,): (mlua::UserDataRef<Buffer>,)| {
                let this = this.borrow();
                let other = other.borrow();

                let mut out = vec![0u8; this.len() + other.len()];
                out[..this.len()].copy_from_slice(&this);
                out[this.len()..].copy_from_slice(&other);
                Ok(Buffer::new(out))
            },
        );

        methods.add_meta_method(
            mlua::MetaMethod::Eq,
            |_, this, (other,): (mlua::UserDataRef<Buffer>,)| Ok(*this.borrow() == *other.borrow()),
        );

        methods.add_method("len", |_, this, (): ()| Ok(this.range.len()));

        methods.add_method("to_string", |lua, this, (): ()| {
            let this = this.borrow();
            lua.create_string(&*this)
        });

        methods.add_method("clone", |_, this, (): ()| {
            let this = this.borrow();
            Ok(Buffer::new(this.to_vec()))
        });

        methods.add_method("slice", |_, this, (i, n): (usize, usize)| {
            this.slice(i..i + n)
                .ok_or_else(|| anyhow::anyhow!("out of bounds").into_lua_err())
        });

        methods.add_method("get", |_, this, (i, n): (usize, usize)| {
            let this = this.borrow();
            Ok(Buffer::new(
                this.get(i..i + n)
                    .ok_or_else(|| anyhow::anyhow!("out of bounds").into_lua_err())?
                    .to_vec(),
            ))
        });

        methods.add_method_mut(
            "set",
            |_, this, (i, buf): (usize, mlua::UserDataRef<Buffer>)| {
                let mut this = this.borrow_mut();
                let buf = buf.borrow();
                let slice = this
                    .get_mut(i..i + buf.len())
                    .ok_or_else(|| anyhow::anyhow!("out of bounds").into_lua_err())?;
                slice.copy_from_slice(&buf);
                Ok(())
            },
        );

        methods.add_method("get_string", |lua, this, (i, n): (usize, usize)| {
            let this = this.borrow();
            lua.create_string(
                this.get(i..i + n)
                    .ok_or_else(|| anyhow::anyhow!("out of bounds").into_lua_err())?,
            )
        });

        methods.add_method_mut("set_string", |_, this, (i, s): (usize, mlua::String)| {
            let mut this = this.borrow_mut();
            let slice = this
                .get_mut(i..i + s.as_bytes().len())
                .ok_or_else(|| anyhow::anyhow!("out of bounds").into_lua_err())?;
            slice.copy_from_slice(s.as_bytes());
            Ok(())
        });

        methods.add_method("get_u8", |_, this, (i,): (usize,)| {
            let this = this.borrow();
            Ok(*(this
                .get(i)
                .ok_or_else(|| anyhow::anyhow!("out of bounds").into_lua_err()))?)
        });

        methods.add_method_mut("set_u8", |_, this, (i, v): (usize, u8)| {
            let mut this = this.borrow_mut();
            *(this
                .get_mut(i)
                .ok_or_else(|| anyhow::anyhow!("out of bounds").into_lua_err())?) = v;
            Ok(())
        });

        methods.add_method("get_u16_le", |_, this, (i,): (usize,)| {
            let this = this.borrow();
            Ok(byteorder::LittleEndian::read_u16(
                this.get(i..i + std::mem::size_of::<u16>())
                    .ok_or_else(|| anyhow::anyhow!("out of bounds").into_lua_err())?,
            ))
        });

        methods.add_method_mut("set_u16_le", |_, this, (i, v): (usize, u16)| {
            let mut this = this.borrow_mut();
            byteorder::LittleEndian::write_u16(
                this.get_mut(i..i + std::mem::size_of::<u16>())
                    .ok_or_else(|| anyhow::anyhow!("out of bounds").into_lua_err())?,
                v,
            );
            Ok(())
        });

        methods.add_method("get_u32_le", |_, this, (i,): (usize,)| {
            let this = this.borrow();
            Ok(byteorder::LittleEndian::read_u32(
                this.get(i..i + std::mem::size_of::<u32>())
                    .ok_or_else(|| anyhow::anyhow!("out of bounds").into_lua_err())?,
            ))
        });

        methods.add_method_mut("set_u32_le", |_, this, (i, v): (usize, u32)| {
            let mut this = this.borrow_mut();
            byteorder::LittleEndian::write_u32(
                this.get_mut(i..i + std::mem::size_of::<u32>())
                    .ok_or_else(|| anyhow::anyhow!("out of bounds").into_lua_err())?,
                v,
            );
            Ok(())
        });

        methods.add_method("get_uq16_16_le", |_, this, (i,): (usize,)| {
            let this = this.borrow();
            Ok(UQ16_16::from_u32(byteorder::LittleEndian::read_u32(
                this.get(i..i + std::mem::size_of::<u32>())
                    .ok_or_else(|| anyhow::anyhow!("out of bounds").into_lua_err())?,
            ))
            .into_f64())
        });

        methods.add_method_mut("set_uq16_16_le", |_, this, (i, v): (usize, f64)| {
            let mut this = this.borrow_mut();
            byteorder::LittleEndian::write_u32(
                this.get_mut(i..i + std::mem::size_of::<u32>())
                    .ok_or_else(|| anyhow::anyhow!("out of bounds").into_lua_err())?,
                UQ16_16::from_f64(v).into_u32(),
            );
            Ok(())
        });

        methods.add_method("get_i8", |_, this, (i,): (usize,)| {
            let this = this.borrow();
            Ok(*(this
                .get(i)
                .ok_or_else(|| anyhow::anyhow!("out of bounds").into_lua_err()))?
                as i8)
        });

        methods.add_method_mut("set_i8", |_, this, (i, v): (usize, i8)| {
            let mut this = this.borrow_mut();
            *(this
                .get_mut(i)
                .ok_or_else(|| anyhow::anyhow!("out of bounds").into_lua_err())?) = v as u8;
            Ok(())
        });

        methods.add_method("get_i16_le", |_, this, (i,): (usize,)| {
            let this = this.borrow();
            Ok(byteorder::LittleEndian::read_i16(
                this.get(i..i + std::mem::size_of::<i16>())
                    .ok_or_else(|| anyhow::anyhow!("out of bounds").into_lua_err())?,
            ))
        });

        methods.add_method_mut("set_i16_le", |_, this, (i, v): (usize, i16)| {
            let mut this = this.borrow_mut();
            byteorder::LittleEndian::write_i16(
                this.get_mut(i..i + std::mem::size_of::<i16>())
                    .ok_or_else(|| anyhow::anyhow!("out of bounds").into_lua_err())?,
                v,
            );
            Ok(())
        });

        methods.add_method("get_i32_le", |_, this, (i,): (usize,)| {
            let this = this.borrow();
            Ok(byteorder::LittleEndian::read_i32(
                this.get(i..i + std::mem::size_of::<i32>())
                    .ok_or_else(|| anyhow::anyhow!("out of bounds").into_lua_err())?,
            ))
        });

        methods.add_method_mut("set_i32_le", |_, this, (i, v): (usize, i32)| {
            let mut this = this.borrow_mut();
            byteorder::LittleEndian::write_i32(
                this.get_mut(i..i + std::mem::size_of::<u32>())
                    .ok_or_else(|| anyhow::anyhow!("out of bounds").into_lua_err())?,
                v,
            );
            Ok(())
        });

        methods.add_method("get_iq16_16_le", |_, this, (i,): (usize,)| {
            let this = this.borrow();
            Ok(IQ16_16::from_i32(byteorder::LittleEndian::read_i32(
                this.get(i..i + std::mem::size_of::<i32>())
                    .ok_or_else(|| anyhow::anyhow!("out of bounds").into_lua_err())?,
            ))
            .into_f64())
        });

        methods.add_method_mut("set_iq16_16_le", |_, this, (i, v): (usize, f64)| {
            let mut this = this.borrow_mut();
            byteorder::LittleEndian::write_i32(
                this.get_mut(i..i + std::mem::size_of::<i32>())
                    .ok_or_else(|| anyhow::anyhow!("out of bounds").into_lua_err())?,
                IQ16_16::from_f64(v).into_i32(),
            );
            Ok(())
        });
    }
}

pub fn new<'a>(lua: &'a mlua::Lua) -> Result<mlua::Value<'a>, mlua::Error> {
    let table = lua.create_table()?;

    table.set(
        "from_string",
        lua.create_function(|_, (raw,): (mlua::String,)| Ok(Buffer::new(raw.as_bytes().to_vec())))?,
    )?;

    table.set(
        "from_u8_table",
        lua.create_function(|_, (raw,): (Vec<u8>,)| Ok(Buffer::new(raw)))?,
    )?;

    table.set(
        "filled",
        lua.create_function(|_, (v, n): (u8, usize)| Ok(Buffer::new(vec![v; n])))?,
    )?;

    table.set(
        "empty",
        lua.create_function(|_, (): ()| Ok(Buffer::new(vec![])))?,
    )?;

    table.set(
        "new_builder",
        lua.create_function(|_, (): ()| Ok(Builder(vec![])))?,
    )?;

    Ok(mlua::Value::Table(table))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_buffer_get_lua() {
        let lua = mlua::Lua::new();
        let globals = lua.globals();
        globals.set("buffer", new(&lua).unwrap()).unwrap();
        assert_eq!(
            &*lua
                .load(
                    r#"
local buf = buffer.from_string("hello")
return buf:get(1, 3)
"#,
                )
                .eval::<mlua::UserDataRef<Buffer>>()
                .unwrap()
                .borrow(),
            &b"ell"[..]
        );
    }

    #[test]
    fn test_buffer_set_lua() {
        let lua = mlua::Lua::new();
        let globals = lua.globals();
        globals.set("buffer", new(&lua).unwrap()).unwrap();
        assert_eq!(
            &*lua
                .load(
                    r#"
local buf = buffer.from_string("hello")
buf:set(1, buffer.from_string("w"))
return buf
"#,
                )
                .eval::<mlua::UserDataRef<Buffer>>()
                .unwrap()
                .borrow(),
            &b"hwllo"[..]
        );
    }

    #[test]
    fn test_buffer_set_lua_slice() {
        let lua = mlua::Lua::new();
        let globals = lua.globals();
        globals.set("buffer", new(&lua).unwrap()).unwrap();
        assert_eq!(
            &*lua
                .load(
                    r#"
local buf = buffer.from_string("hello")
local slice = buf:slice(1, 3)
slice:set(1, buffer.from_string("n"))
return buf
"#,
                )
                .eval::<mlua::UserDataRef<Buffer>>()
                .unwrap()
                .borrow(),
            &b"henlo"[..]
        );
    }
}
