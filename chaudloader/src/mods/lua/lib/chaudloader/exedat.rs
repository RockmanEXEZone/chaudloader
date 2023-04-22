use crate::assets;
use mlua::ExternalError;

struct ExeDat(std::sync::Arc<std::sync::Mutex<assets::exedat::Overlay>>);

impl mlua::UserData for ExeDat {
    fn add_methods<'lua, M: mlua::UserDataMethods<'lua, Self>>(methods: &mut M) {
        methods.add_method("read_file", |lua, this, (path,): (String,)| {
            let mut overlay = this.0.lock().unwrap();
            Ok(Some(lua.create_string(
                &overlay.read(&path).map_err(|e| e.to_lua_err())?.to_vec(),
            )?))
        });

        methods.add_method(
            "write_file",
            |_, this, (path, contents): (String, mlua::String)| {
                let mut overlay = this.0.lock().unwrap();
                overlay
                    .write(&path, contents.as_bytes().to_vec())
                    .map_err(|e| e.to_lua_err())?;
                Ok(())
            },
        );
    }
}

pub fn new<'a>(
    lua: &'a mlua::Lua,
    overlays: std::collections::HashMap<
        String,
        std::sync::Arc<std::sync::Mutex<assets::exedat::Overlay>>,
    >,
) -> Result<mlua::Value<'a>, mlua::Error> {
    Ok(mlua::Value::Function(lua.create_function({
        move |_, (name,): (String,)| {
            let overlay = if let Some(overlay) = overlays.get(&name) {
                std::sync::Arc::clone(&overlay)
            } else {
                return Err(anyhow::format_err!("no such dat file: {}", name).to_lua_err());
            };
            Ok(ExeDat(overlay))
        }
    })?))
}
