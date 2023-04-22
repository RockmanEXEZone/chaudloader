use mlua::ExternalError;

pub fn new<'a>(lua: &'a mlua::Lua) -> Result<mlua::Value<'a>, mlua::Error> {
    let table = lua.create_table()?;

    table.set(
        "read_process_memory",
        lua.create_function(|lua, (addr, len): (usize, usize)| {
            let mut buf = vec![0u8; len];
            let mut number_of_bytes_read: winapi::shared::basetsd::SIZE_T = 0;
            unsafe {
                let current_process = winapi::um::processthreadsapi::GetCurrentProcess();
                if winapi::um::memoryapi::ReadProcessMemory(
                    current_process,
                    addr as winapi::shared::minwindef::LPCVOID,
                    buf.as_mut_ptr() as winapi::shared::minwindef::LPVOID,
                    buf.len() as winapi::shared::basetsd::SIZE_T,
                    &mut number_of_bytes_read as *mut winapi::shared::basetsd::SIZE_T,
                ) != winapi::shared::minwindef::TRUE
                {
                    return Err(
                        anyhow::format_err!("ReadProcessMemory returned false").into_lua_err()
                    );
                }
            }
            buf.drain(number_of_bytes_read as usize..);
            Ok(lua.create_string(buf.as_slice())?)
        })?,
    )?;

    table.set(
        "write_process_memory",
        lua.create_function(|_, (addr, buf): (usize, mlua::String)| {
            let mut number_of_bytes_written: winapi::shared::basetsd::SIZE_T = 0;
            unsafe {
                let current_process = winapi::um::processthreadsapi::GetCurrentProcess();
                if winapi::um::memoryapi::WriteProcessMemory(
                    current_process,
                    addr as winapi::shared::minwindef::LPVOID,
                    buf.as_bytes().as_ptr() as winapi::shared::minwindef::LPVOID,
                    buf.as_bytes().len() as winapi::shared::basetsd::SIZE_T,
                    &mut number_of_bytes_written as *mut winapi::shared::basetsd::SIZE_T,
                ) != winapi::shared::minwindef::TRUE
                {
                    return Err(
                        anyhow::format_err!("ReadProcessMemory returned false").into_lua_err()
                    );
                }
            }
            Ok(number_of_bytes_written)
        })?,
    )?;

    Ok(mlua::Value::Table(table))
}
