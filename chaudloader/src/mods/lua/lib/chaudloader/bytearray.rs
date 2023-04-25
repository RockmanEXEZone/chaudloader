use byteorder::ByteOrder;
use mlua::ExternalError;

pub struct ByteArray(Vec<u8>);

impl mlua::UserData for ByteArray {
    fn add_methods<'lua, M: mlua::UserDataMethods<'lua, Self>>(methods: &mut M) {
        methods.add_meta_method(
            mlua::MetaMethod::Concat,
            |_, this, (other,): (mlua::UserDataRef<ByteArray>,)| {
                let mut out = vec![0u8; this.0.len() + other.0.len()];
                out[..this.0.len()].copy_from_slice(&this.0);
                out[this.0.len()..].copy_from_slice(&other.0);
                Ok(ByteArray(out))
            },
        );

        methods.add_meta_method(
            mlua::MetaMethod::Eq,
            |_, this, (other,): (mlua::UserDataRef<ByteArray>,)| Ok(this.0 == other.0),
        );

        methods.add_method("len", |_, this, (): ()| Ok(this.0.len()));

        methods.add_method("slice", |_, this, (i, n): (usize, usize)| {
            Ok(this.0[i..i + n].to_vec())
        });

        methods.add_method("pack", |lua, this, (): ()| Ok(lua.create_string(&this.0)?));

        methods.add_method("get", |lua, this, (i, n): (usize, usize)| {
            Ok(lua.create_string(
                &this
                    .0
                    .get(i..i + n)
                    .ok_or_else(|| anyhow::anyhow!("out of bounds").into_lua_err())?,
            )?)
        });

        methods.add_method_mut("set", |_, this, (i, s): (usize, mlua::String)| {
            let slice = this
                .0
                .get_mut(i..i + s.as_bytes().len())
                .ok_or_else(|| anyhow::anyhow!("out of bounds").into_lua_err())?;
            slice.copy_from_slice(s.as_bytes());
            Ok(())
        });

        methods.add_method("get_u8", |_, this, (i,): (usize,)| {
            Ok(*(this
                .0
                .get(i)
                .ok_or_else(|| anyhow::anyhow!("out of bounds").into_lua_err()))?)
        });

        methods.add_method_mut("set_u8", |_, this, (i, v): (usize, u8)| {
            *(this
                .0
                .get_mut(i)
                .ok_or_else(|| anyhow::anyhow!("out of bounds").into_lua_err())?) = v;
            Ok(())
        });

        methods.add_method("get_u16_le", |_, this, (i,): (usize,)| {
            Ok(byteorder::LittleEndian::read_u16(
                this.0
                    .get(i..i + std::mem::size_of::<u16>())
                    .ok_or_else(|| anyhow::anyhow!("out of bounds").into_lua_err())?,
            ))
        });

        methods.add_method_mut("set_u16_le", |_, this, (i, v): (usize, u16)| {
            byteorder::LittleEndian::write_u16(
                this.0
                    .get_mut(i..i + std::mem::size_of::<u16>())
                    .ok_or_else(|| anyhow::anyhow!("out of bounds").into_lua_err())?,
                v,
            );
            Ok(())
        });

        methods.add_method("get_u32_le", |_, this, (i,): (usize,)| {
            Ok(byteorder::LittleEndian::read_u32(
                this.0
                    .get(i..i + std::mem::size_of::<u32>())
                    .ok_or_else(|| anyhow::anyhow!("out of bounds").into_lua_err())?,
            ))
        });

        methods.add_method_mut("set_u32_le", |_, this, (i, v): (usize, u32)| {
            byteorder::LittleEndian::write_u32(
                this.0
                    .get_mut(i..i + std::mem::size_of::<u32>())
                    .ok_or_else(|| anyhow::anyhow!("out of bounds").into_lua_err())?,
                v,
            );
            Ok(())
        });

        methods.add_method("get_i8", |_, this, (i,): (usize,)| {
            Ok(*(this
                .0
                .get(i)
                .ok_or_else(|| anyhow::anyhow!("out of bounds").into_lua_err()))?
                as i8)
        });

        methods.add_method_mut("set_i8", |_, this, (i, v): (usize, i8)| {
            *(this
                .0
                .get_mut(i)
                .ok_or_else(|| anyhow::anyhow!("out of bounds").into_lua_err())?) = v as u8;
            Ok(())
        });

        methods.add_method("get_i16_le", |_, this, (i,): (usize,)| {
            Ok(byteorder::LittleEndian::read_i16(
                this.0
                    .get(i..i + std::mem::size_of::<i16>())
                    .ok_or_else(|| anyhow::anyhow!("out of bounds").into_lua_err())?,
            ))
        });

        methods.add_method_mut("set_i16_le", |_, this, (i, v): (usize, i16)| {
            byteorder::LittleEndian::write_i16(
                this.0
                    .get_mut(i..i + std::mem::size_of::<i16>())
                    .ok_or_else(|| anyhow::anyhow!("out of bounds").into_lua_err())?,
                v,
            );
            Ok(())
        });

        methods.add_method("get_i32_le", |_, this, (i,): (usize,)| {
            Ok(byteorder::LittleEndian::read_i32(
                this.0
                    .get(i..i + std::mem::size_of::<i32>())
                    .ok_or_else(|| anyhow::anyhow!("out of bounds").into_lua_err())?,
            ))
        });

        methods.add_method_mut("set_i32_le", |_, this, (i, v): (usize, i32)| {
            byteorder::LittleEndian::write_i32(
                this.0
                    .get_mut(i..i + std::mem::size_of::<u32>())
                    .ok_or_else(|| anyhow::anyhow!("out of bounds").into_lua_err())?,
                v,
            );
            Ok(())
        });
    }
}

pub fn new<'a>(lua: &'a mlua::Lua) -> Result<mlua::Value<'a>, mlua::Error> {
    Ok(mlua::Value::Function(lua.create_function({
        move |_, (raw,): (mlua::String,)| Ok(ByteArray(raw.as_bytes().to_vec()))
    })?))
}
