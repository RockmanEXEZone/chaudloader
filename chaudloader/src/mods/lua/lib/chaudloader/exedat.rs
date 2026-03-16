use crate::{assets, mods::lua::lib::chaudloader::buffer::Buffer};
use mlua::ExternalError;

struct ExeDat(std::rc::Rc<std::cell::RefCell<assets::exedat::Overlay>>);

impl mlua::UserData for ExeDat {
    fn add_methods<'lua, M: mlua::UserDataMethods<'lua, Self>>(methods: &mut M) {
        methods.add_method("read_file", |_, this, (path,): (String,)| {
            let mut this = this.0.borrow_mut();
            Ok(Some(Buffer::new(
                this.read(&path).map_err(|e| e.into_lua_err())?.to_vec(),
            )))
        });

        methods.add_method(
            "write_file",
            |_, this, (path, contents): (String, mlua::UserDataRef<Buffer>)| {
                let mut this = this.0.borrow_mut();
                this.write(&path, contents.borrow().to_vec())
                    .map_err(|e| e.into_lua_err())?;
                Ok(())
            },
        );

        methods.add_method(
            "write_asset",
            |_, this, (name, contents): (String, mlua::UserDataRef<Buffer>)| {
                let mut this = this.0.borrow_mut();
                let root_folder = this.get_root_folder_name();

                let name_crc = crc32fast::hash(name.as_bytes());

                let rom_dic_path = std::format!("{root_folder}/rom.dic");
                let rom_srl_path = std::format!("{root_folder}/rom.srl");
                let mut rom_dic = this.read(&rom_dic_path)?.to_vec();
                let mut rom_srl = this.read(&rom_srl_path)?.to_vec();

                // Go through the dic
                let mut offset = 0;
                while offset < rom_dic.len() {
                    let entry_crc = u32::from_le_bytes(rom_dic[offset..offset+4].try_into().map_err(|_| {
                        anyhow::anyhow!("unexpected end of dic file").into_lua_err()
                    })?);
                    if entry_crc == 0 {
                        // rom_e.dic has symbols after the dic entries, crc of 0 marks the end of dic entries
                        break;
                    }

                    // Check if we have enough bytes to read this entry so we can just unwrap
                    if rom_dic.len() - offset < 24 {
                        return Err(anyhow::anyhow!("unexpected end of dic file").into_lua_err());
                    }

                    if entry_crc == name_crc {
                        let old_addr = u32::from_le_bytes(rom_dic[offset+4..offset+8].try_into().unwrap());
                        let old_size = u32::from_le_bytes(rom_dic[offset+20..offset+24].try_into().unwrap());
                        let new_addr = 0x8000000 + rom_srl.len();
                        let new_size = contents.borrow().len();
                        log::info!("replacing asset {name} @ 0x{old_addr:08X}, size 0x{old_size:X} -> 0x{new_addr:08X}, size 0x{new_size:X}");

                        // Append the new asset to the end of the ROM
                        rom_srl.extend(contents.borrow().to_vec());
                        // Pad to multiple of 4
                        while rom_srl.len() % 4 != 0 {
                            rom_srl.push(0);
                        }

                        // Write the new address and size into the dic
                        rom_dic.splice(offset+4..offset+8, u32::to_le_bytes(new_addr.try_into().unwrap()));
                        rom_dic.splice(offset+20..offset+24, u32::to_le_bytes(new_size.try_into().unwrap()));

                        this.write(&rom_dic_path, rom_dic)?;
                        this.write(&rom_srl_path, rom_srl)?;

                        return Ok(new_addr);
                    }

                    // Go to next entry
                    offset += 24;
                }

                Err(anyhow::anyhow!("unknown asset {name}").into_lua_err())
            },
        )
    }
}

pub fn new<'a>(
    lua: &'a mlua::Lua,
    overlays: std::collections::HashMap<
        String,
        std::rc::Rc<std::cell::RefCell<assets::exedat::Overlay>>,
    >,
) -> Result<mlua::Value<'a>, mlua::Error> {
    let table = lua.create_table()?;

    table.set(
        "open",
        lua.create_function({
            move |_, (name,): (String,)| {
                let overlay = if let Some(overlay) = overlays.get(&name) {
                    std::rc::Rc::clone(overlay)
                } else {
                    return Err(anyhow::format_err!("no such dat file: {}", name).into_lua_err());
                };
                Ok(ExeDat(overlay))
            }
        })?,
    )?;

    Ok(mlua::Value::Table(table))
}
