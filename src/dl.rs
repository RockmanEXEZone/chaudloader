use std::os::windows::ffi::OsStrExt;

pub struct ModuleHandle(winapi::shared::minwindef::HMODULE);

impl ModuleHandle {
    pub unsafe fn get(module: &str) -> Option<Self> {
        let module = module
            .encode_utf16()
            .chain(std::iter::once(0))
            .collect::<Vec<u16>>();
        let hmodule = winapi::um::libloaderapi::GetModuleHandleW(module.as_ptr());
        if hmodule.is_null() {
            None
        } else {
            Some(ModuleHandle(hmodule))
        }
    }

    pub unsafe fn load(path: &std::path::Path) -> Option<Self> {
        let path_w = path
            .as_os_str()
            .encode_wide()
            .chain(std::iter::once(0))
            .collect::<Vec<_>>();
        let hmodule = winapi::um::libloaderapi::LoadLibraryW(path_w.as_ptr());
        if hmodule.is_null() {
            None
        } else {
            Some(ModuleHandle(hmodule))
        }
    }

    pub unsafe fn get_symbol_address(
        &self,
        symbol: &str,
    ) -> Option<winapi::shared::minwindef::FARPROC> {
        let symbol_cstr = std::ffi::CString::new(symbol).unwrap();
        let farproc = winapi::um::libloaderapi::GetProcAddress(self.0, symbol_cstr.as_ptr());
        if farproc.is_null() {
            None
        } else {
            Some(farproc)
        }
    }
}

unsafe impl Send for ModuleHandle {}
unsafe impl Sync for ModuleHandle {}
