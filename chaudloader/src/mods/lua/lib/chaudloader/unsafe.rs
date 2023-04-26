use crate::{
    mods::{self, lua::lib::chaudloader::buffer::Buffer},
    path,
};
use mlua::ExternalError;

pub fn new<'a>(
    lua: &'a mlua::Lua,
    mod_path: &std::path::Path,
    state: std::rc::Rc<std::cell::RefCell<mods::State>>,
) -> Result<mlua::Value<'a>, mlua::Error> {
    let table = lua.create_table()?;

    table.set(
        "read_process_memory",
        lua.create_function(|_, (addr, len): (usize, usize)| {
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
            Ok(Buffer::new(buf))
        })?,
    )?;

    table.set(
        "write_process_memory",
        lua.create_function(|_, (addr, buf): (usize, mlua::UserDataRef<Buffer>)| {
            let mut number_of_bytes_written: winapi::shared::basetsd::SIZE_T = 0;
            unsafe {
                let current_process = winapi::um::processthreadsapi::GetCurrentProcess();
                if winapi::um::memoryapi::WriteProcessMemory(
                    current_process,
                    addr as winapi::shared::minwindef::LPVOID,
                    buf.as_slice().as_ptr() as winapi::shared::minwindef::LPVOID,
                    buf.as_slice().len() as winapi::shared::basetsd::SIZE_T,
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

    table.set(
        "alloc_executable_memory",
        lua.create_function(|_, (buf,): (mlua::UserDataRef<Buffer>,)| unsafe {
            // We allocate the page with read/write, then set it to execute after our copy is complete. This means we comply with W^X requirements.
            let out_buf = winapi::um::memoryapi::VirtualAlloc(
                std::ptr::null_mut(),
                buf.as_slice().len(),
                winapi::um::winnt::MEM_COMMIT,
                winapi::um::winnt::PAGE_READWRITE,
            );
            if out_buf.is_null() {
                return Err(anyhow::anyhow!("VirtualAlloc returned null").into_lua_err());
            }

            std::slice::from_raw_parts_mut::<'_, u8>(
                std::mem::transmute(out_buf),
                buf.as_slice().len(),
            )
            .copy_from_slice(buf.as_slice());

            let mut dummy = 0;
            if winapi::um::memoryapi::VirtualProtect(
                out_buf,
                buf.as_slice().len(),
                winapi::um::winnt::PAGE_EXECUTE_READ,
                &mut dummy,
            ) != winapi::shared::minwindef::TRUE
            {
                // Failing to free the memory is a memory leak!
                assert_eq!(
                    winapi::um::memoryapi::VirtualFree(out_buf, 0, winapi::um::winnt::MEM_FREE),
                    winapi::shared::minwindef::TRUE
                );
                return Err(anyhow::anyhow!("VirtualProtect returned false").into_lua_err());
            }
            Ok(std::mem::transmute::<_, usize>(out_buf))
        })?,
    )?;

    table.set(
        "free_executable_memory",
        lua.create_function(|_, (addr,): (usize,)| unsafe {
            if winapi::um::memoryapi::VirtualFree(
                std::mem::transmute(addr),
                0,
                winapi::um::winnt::MEM_FREE,
            ) != winapi::shared::minwindef::TRUE
            {
                return Err(anyhow::anyhow!("VirtualFree returned false").into_lua_err());
            }
            Ok(())
        })?,
    )?;

    table.set(
        "init_mod_dll",
        lua.create_function({
            let mod_path = mod_path.to_path_buf();
            let state = std::rc::Rc::clone(&state);
            move |_, (path, buf): (String, mlua::String)| {
                let path = path::ensure_safe(std::path::Path::new(&path))
                    .ok_or_else(|| anyhow::anyhow!("cannot read files outside of mod directory"))
                    .map_err(|e| e.into_lua_err())?;
                let mut state = state.borrow_mut();
                type ChaudloaderInitFn =
                    unsafe extern "system" fn(userdata: *const u8, n: usize) -> bool;
                let dll = unsafe {
                    let dll = windows_libloader::ModuleHandle::load(&mod_path.join(&path))
                        .ok_or_else(|| anyhow::anyhow!("DLL {} failed to load", path.display()))
                        .map_err(|e| e.into_lua_err())?;
                    let init_fn = std::mem::transmute::<_, ChaudloaderInitFn>(
                        dll.get_symbol_address("chaudloader_init")
                            .ok_or_else(|| {
                                anyhow::anyhow!(
                                    "ChaudLoaderInit not found in DLL {}",
                                    path.display()
                                )
                            })
                            .map_err(|e| e.into_lua_err())?,
                    );
                    let buf = buf.as_bytes();
                    if !init_fn(buf.as_ptr(), buf.len()) {
                        return Err(anyhow::anyhow!(
                            "ChaudLoaderInit for DLL {} returned false",
                            path.display()
                        )
                        .into_lua_err());
                    }
                    dll
                };
                state.dlls.insert(path, dll);
                Ok(())
            }
        })?,
    )?;

    Ok(mlua::Value::Table(table))
}
