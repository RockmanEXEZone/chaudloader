//! Dynamic linker functions.
#![cfg(windows)]

use std::os::windows::ffi::OsStrExt;
use std::os::windows::ffi::OsStringExt;

pub fn get_system_directory() -> std::path::PathBuf {
    unsafe {
        let n = winapi::um::sysinfoapi::GetSystemDirectoryW(std::ptr::null_mut(), 0) as usize;
        let mut buf = vec![0u16; n];
        // https://learn.microsoft.com/en-us/windows/win32/api/sysinfoapi/nf-sysinfoapi-getsystemdirectoryw: If the function succeeds, the return value is the length, in TCHARs, of the string copied to the buffer, not including the terminating null character. If the length is greater than the size of the buffer, the return value is the size of the buffer required to hold the path, including the terminating null character.
        //
        // wtf but okay.
        assert_eq!(
            winapi::um::sysinfoapi::GetSystemDirectoryW(buf.as_mut_ptr(), n as u32) as usize,
            n - 1
        );
        std::path::Path::new(&std::ffi::OsString::from_wide(&buf[..n - 1])).to_owned()
    }
}

/// A module handle is a handle to a DLL.
pub struct ModuleHandle {
    hmodule: winapi::shared::minwindef::HMODULE,
    needs_free: bool,
}

impl ModuleHandle {
    /// Gets an already loaded module by its name.
    pub unsafe fn get(module_name: &str) -> Option<Self> {
        let module_name_w = module_name
            .encode_utf16()
            .chain(std::iter::once(0))
            .collect::<Vec<u16>>();
        let hmodule = winapi::um::libloaderapi::GetModuleHandleW(module_name_w.as_ptr());
        if hmodule.is_null() {
            None
        } else {
            Some(ModuleHandle {
                hmodule,
                needs_free: false,
            })
        }
    }

    /// Loads a DLL by path.
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
            Some(ModuleHandle {
                hmodule,
                needs_free: true,
            })
        }
    }

    /// Gets a symbol address as a farproc, if it exists in the module.
    pub unsafe fn get_symbol_address(
        &self,
        symbol: &str,
    ) -> Option<winapi::shared::minwindef::FARPROC> {
        let symbol_cstr = std::ffi::CString::new(symbol).unwrap();
        let farproc = winapi::um::libloaderapi::GetProcAddress(self.hmodule, symbol_cstr.as_ptr());
        if farproc.is_null() {
            None
        } else {
            Some(farproc)
        }
    }
}

impl Drop for ModuleHandle {
    fn drop(&mut self) {
        if self.needs_free {
            unsafe {
                winapi::um::libloaderapi::FreeLibrary(self.hmodule);
            }
        }
    }
}

unsafe impl Send for ModuleHandle {}
unsafe impl Sync for ModuleHandle {}
