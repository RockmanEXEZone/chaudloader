use crate::{assets, hooks};
use retour::static_detour;

use std::os::windows::ffi::OsStrExt;
use std::os::windows::ffi::OsStringExt;

type NtCreateFileFunc = unsafe extern "system" fn(
    file_handle: winapi::shared::ntdef::PHANDLE,
    desired_access: winapi::um::winnt::ACCESS_MASK,
    object_attributes: winapi::shared::ntdef::POBJECT_ATTRIBUTES,
    io_status_block: ntapi::ntioapi::PIO_STATUS_BLOCK,
    allocation_size: winapi::shared::ntdef::PLARGE_INTEGER,
    file_attributes: winapi::shared::minwindef::ULONG,
    share_access: winapi::shared::ntdef::ULONG,
    create_disposition: winapi::shared::minwindef::ULONG,
    create_options: winapi::shared::minwindef::ULONG,
    ea_buffer: winapi::shared::ntdef::PVOID,
    ea_length: winapi::shared::minwindef::ULONG,
) -> winapi::shared::ntdef::NTSTATUS;

type DuplicateHandleFunc = unsafe extern "system" fn(
    h_source_process_handle: winapi::shared::ntdef::HANDLE,
    h_source_handle: winapi::shared::ntdef::HANDLE,
    h_target_process_handle: winapi::shared::ntdef::HANDLE,
    lp_target_handle: winapi::shared::minwindef::LPHANDLE,
    dw_desired_access: winapi::shared::minwindef::DWORD,
    b_inherit_handle: winapi::shared::minwindef::BOOL,
    dw_options: winapi::shared::minwindef::DWORD,
) -> winapi::shared::minwindef::BOOL;

type CloseHandleFunc = unsafe extern "system" fn(
    h_object: winapi::shared::ntdef::HANDLE,
) -> winapi::shared::minwindef::BOOL;

static_detour! {
    static NtCreateFileHook: unsafe extern "system" fn(
        /* file_handle: */ winapi::shared::ntdef::PHANDLE,
        /* desired_access: */ winapi::um::winnt::ACCESS_MASK,
        /* object_attributes: */ winapi::shared::ntdef::POBJECT_ATTRIBUTES,
        /* io_status_block: */ ntapi::ntioapi::PIO_STATUS_BLOCK,
        /* allocation_size: */ winapi::shared::ntdef::PLARGE_INTEGER,
        /* file_attributes: */ winapi::shared::minwindef::ULONG,
        /* share_access: */ winapi::shared::ntdef::ULONG,
        /* create_disposition: */ winapi::shared::minwindef::ULONG,
        /* create_options: */ winapi::shared::minwindef::ULONG,
        /* ea_buffer: */ winapi::shared::ntdef::PVOID,
        /* ea_length: */ winapi::shared::minwindef::ULONG
    ) -> winapi::shared::ntdef::NTSTATUS;

    static DuplicateHandleHook: unsafe extern "system" fn(
        /* h_source_process_handle: */ winapi::shared::ntdef::HANDLE,
        /* h_source_handle: */ winapi::shared::ntdef::HANDLE,
        /* h_target_process_handle: */ winapi::shared::ntdef::HANDLE,
        /* lp_target_handle: */ winapi::shared::minwindef::LPHANDLE,
        /* dw_desired_access: */ winapi::shared::minwindef::DWORD,
        /* b_inherit_handle: */ winapi::shared::minwindef::BOOL,
        /* dw_options: */ winapi::shared::minwindef::DWORD
    ) -> winapi::shared::minwindef::BOOL;

    static CloseHandleHook: unsafe extern "system" fn(
        /* h_object: */ winapi::shared::ntdef::HANDLE
    ) -> winapi::shared::minwindef::BOOL;
}

struct HooksDisableGuard {
    _nt_create_file_guard: hooks::HookDisableGuard<NtCreateFileFunc>,
    _duplicate_handle_guard: hooks::HookDisableGuard<DuplicateHandleFunc>,
    _close_handle_guard: hooks::HookDisableGuard<CloseHandleFunc>,
}

impl HooksDisableGuard {
    unsafe fn new() -> Result<Self, retour::Error> {
        Ok(Self {
            _nt_create_file_guard: hooks::HookDisableGuard::new(&NtCreateFileHook)?,
            _duplicate_handle_guard: hooks::HookDisableGuard::new(&DuplicateHandleHook)?,
            _close_handle_guard: hooks::HookDisableGuard::new(&CloseHandleHook)?,
        })
    }
}

static HANDLE_TRACKER: std::sync::LazyLock<std::sync::Mutex<HandleTracker>> =
    std::sync::LazyLock::new(|| {
        std::sync::Mutex::new(HandleTracker {
            handle_to_path: std::collections::HashMap::new(),
            path_to_handles: std::collections::HashMap::new(),
        })
    });

struct HandleTracker {
    handle_to_path: std::collections::HashMap<usize, std::path::PathBuf>,
    path_to_handles:
        std::collections::HashMap<std::path::PathBuf, std::collections::HashSet<usize>>,
}

impl HandleTracker {
    fn insert(&mut self, path: &std::path::Path, handle: winapi::shared::ntdef::HANDLE) {
        let handle = unsafe { std::mem::transmute(handle) };
        self.handle_to_path.insert(handle, path.to_path_buf());
        self.path_to_handles
            .entry(path.to_path_buf())
            .or_insert_with(|| std::collections::HashSet::new())
            .insert(handle);
    }

    fn dupe(
        &mut self,
        src_handle: winapi::shared::ntdef::HANDLE,
        dest_handle: winapi::shared::ntdef::HANDLE,
    ) -> bool {
        let src_handle = unsafe { std::mem::transmute(src_handle) };
        let path = if let Some(path) = self.handle_to_path.get(&src_handle).cloned() {
            path
        } else {
            return false;
        };

        let dest_handle = unsafe { std::mem::transmute(dest_handle) };

        self.handle_to_path.insert(dest_handle, path.clone());
        self.path_to_handles
            .get_mut(&path)
            .unwrap()
            .insert(dest_handle);

        true
    }

    fn remove(
        &mut self,
        handle: winapi::shared::ntdef::HANDLE,
    ) -> Option<(std::path::PathBuf, usize)> {
        let handle = unsafe { std::mem::transmute(handle) };
        let path = if let Some(path) = self.handle_to_path.remove(&handle) {
            path
        } else {
            return None;
        };

        let mut handles_entry = match self.path_to_handles.entry(path) {
            std::collections::hash_map::Entry::Occupied(entry) => entry,
            std::collections::hash_map::Entry::Vacant(_) => unreachable!(),
        };

        handles_entry.get_mut().remove(&handle);
        let path = handles_entry.key().to_path_buf();

        Some((
            path,
            if handles_entry.get().is_empty() {
                handles_entry.remove();
                0
            } else {
                handles_entry.get().len()
            },
        ))
    }
}

unsafe fn on_nt_create_file(
    file_handle: winapi::shared::ntdef::PHANDLE,
    desired_access: winapi::um::winnt::ACCESS_MASK,
    object_attributes: winapi::shared::ntdef::POBJECT_ATTRIBUTES,
    io_status_block: ntapi::ntioapi::PIO_STATUS_BLOCK,
    allocation_size: winapi::shared::ntdef::PLARGE_INTEGER,
    file_attributes: winapi::shared::minwindef::ULONG,
    share_access: winapi::shared::ntdef::ULONG,
    create_disposition: winapi::shared::minwindef::ULONG,
    create_options: winapi::shared::minwindef::ULONG,
    ea_buffer: winapi::shared::ntdef::PVOID,
    ea_length: winapi::shared::minwindef::ULONG,
) -> winapi::shared::ntdef::NTSTATUS {
    let _hook_disable_guard: HooksDisableGuard = HooksDisableGuard::new().unwrap();

    if (*(*object_attributes).ObjectName).Length == 0 {
        // We are not even opening a named file.
        return NtCreateFileHook.call(
            file_handle,
            desired_access,
            object_attributes,
            io_status_block,
            allocation_size,
            file_attributes,
            share_access,
            create_disposition,
            create_options,
            ea_buffer,
            ea_length,
        );
    }

    let object_name = std::ffi::OsString::from_wide(std::slice::from_raw_parts(
        (*(*object_attributes).ObjectName).Buffer,
        (*(*object_attributes).ObjectName).Length as usize
            / std::mem::size_of::<winapi::shared::ntdef::WCHAR>(),
    ));
    let path = std::path::Path::new(&object_name);

    // Handle tracker must be locked before asset replacer to avoid lock inversion.
    let mut handle_tracker = HANDLE_TRACKER.lock().unwrap();
    let mut assets_replacer = assets::REPLACER.get().unwrap().lock().unwrap();

    let new_path = if let Some(new_path) = assets_replacer.get(path).unwrap() {
        new_path
    } else {
        // There is no appropriate replacement for this file.
        return NtCreateFileHook.call(
            file_handle,
            desired_access,
            object_attributes,
            io_status_block,
            allocation_size,
            file_attributes,
            share_access,
            create_disposition,
            create_options,
            ea_buffer,
            ea_length,
        );
    };

    let mut path_wstr = {
        // Path needs to be converted into an NT Object Manager path (\??\...): this is not the same as a UNC path (\\?\...)...
        let mut oss = std::ffi::OsString::from("\\??\\");
        oss.push(new_path.as_os_str());
        oss.as_os_str().encode_wide().collect::<Vec<_>>()
    };
    let path_wstr_byte_len = path_wstr.len() * std::mem::size_of::<winapi::shared::ntdef::WCHAR>();

    let status = NtCreateFileHook.call(
        file_handle,
        desired_access,
        &mut winapi::shared::ntdef::OBJECT_ATTRIBUTES {
            RootDirectory: std::ptr::null_mut(),
            ObjectName: &mut winapi::shared::ntdef::UNICODE_STRING {
                Buffer: path_wstr.as_mut_ptr(),
                Length: path_wstr_byte_len as winapi::shared::ntdef::USHORT,
                MaximumLength: path_wstr_byte_len as winapi::shared::ntdef::USHORT,
            },
            ..*object_attributes
        },
        io_status_block,
        allocation_size,
        file_attributes,
        share_access,
        create_disposition,
        create_options,
        ea_buffer,
        ea_length,
    );

    // If we failed to open the file we created ourselves, just abort.
    assert_eq!(status, winapi::shared::ntstatus::STATUS_SUCCESS);

    let handle = unsafe { *file_handle };

    handle_tracker.insert(path, handle);
    log::info!(
        "NtCreateFile: read to {} was redirected -> {} (handle: {:p}, open handles: {})",
        path.display(),
        new_path.display(),
        handle,
        handle_tracker
            .path_to_handles
            .get(path)
            .map(|v| v.len())
            .unwrap_or(0)
    );

    winapi::shared::ntstatus::STATUS_SUCCESS
}

unsafe fn on_duplicate_handle(
    h_source_process_handle: winapi::shared::ntdef::HANDLE,
    h_source_handle: winapi::shared::ntdef::HANDLE,
    h_target_process_handle: winapi::shared::ntdef::HANDLE,
    lp_target_handle: winapi::shared::minwindef::LPHANDLE,
    dw_desired_access: winapi::shared::minwindef::DWORD,
    b_inherit_handle: winapi::shared::minwindef::BOOL,
    dw_options: winapi::shared::minwindef::DWORD,
) -> winapi::shared::minwindef::BOOL {
    let _hook_disable_guard: HooksDisableGuard = HooksDisableGuard::new().unwrap();

    if DuplicateHandleHook.call(
        h_source_process_handle,
        h_source_handle,
        h_target_process_handle,
        lp_target_handle,
        dw_desired_access,
        b_inherit_handle,
        dw_options,
    ) == winapi::shared::minwindef::FALSE
    {
        return winapi::shared::minwindef::FALSE;
    }

    let mut handle_tracker: std::sync::MutexGuard<HandleTracker> = HANDLE_TRACKER.lock().unwrap();

    let target_handle = *lp_target_handle;

    if handle_tracker.dupe(h_source_handle, target_handle) {
        log::info!(
            "DuplicateHandle: tracked handle duped: {:p} -> {:p}",
            h_source_handle,
            target_handle
        );
    }

    winapi::shared::minwindef::TRUE
}

unsafe fn on_close_handle(
    h_object: winapi::shared::ntdef::HANDLE,
) -> winapi::shared::minwindef::BOOL {
    let _hook_disable_guard: HooksDisableGuard = HooksDisableGuard::new().unwrap();

    if CloseHandleHook.call(h_object) == winapi::shared::minwindef::FALSE {
        return winapi::shared::minwindef::FALSE;
    }

    // Handle tracker must be locked before asset replacer to avoid lock inversion.
    let mut handle_tracker: std::sync::MutexGuard<HandleTracker> = HANDLE_TRACKER.lock().unwrap();
    let mut assets_replacer = assets::REPLACER.get().unwrap().lock().unwrap();

    if let Some((path, refcount)) = handle_tracker.remove(h_object) {
        if refcount == 0 {
            let purged_path = assets_replacer.purge(&path).unwrap();
            if let Some(purged_path) = purged_path {
                log::info!(
                    "CloseHandle: last handle to {} was closed, purged: {}",
                    path.display(),
                    purged_path.display()
                );
            } else {
                log::warn!(
                "CloseHandle: last handle to {} was closed, but path was not found in asset replacer!",
                path.display(),
            );
            }
        }
    }

    winapi::shared::minwindef::TRUE
}

/// Install hooks into the process.
pub unsafe fn install() -> Result<(), anyhow::Error> {
    static KERNEL32: std::sync::LazyLock<windows_libloader::ModuleHandle> =
        std::sync::LazyLock::new(|| unsafe {
            windows_libloader::ModuleHandle::get("kernel32.dll").unwrap()
        });

    static NTDLL: std::sync::LazyLock<windows_libloader::ModuleHandle> =
        std::sync::LazyLock::new(|| unsafe {
            windows_libloader::ModuleHandle::get("ntdll.dll").unwrap()
        });

    unsafe {
        NtCreateFileHook
            .initialize(
                std::mem::transmute(NTDLL.get_symbol_address("NtCreateFile").unwrap()),
                |file_handle,
                 desired_access,
                 object_attributes,
                 io_status_block,
                 allocation_size,
                 file_attributes,
                 share_access,
                 create_disposition,
                 create_options,
                 ea_buffer,
                 ea_length| {
                    on_nt_create_file(
                        file_handle,
                        desired_access,
                        object_attributes,
                        io_status_block,
                        allocation_size,
                        file_attributes,
                        share_access,
                        create_disposition,
                        create_options,
                        ea_buffer,
                        ea_length,
                    )
                },
            )?
            .enable()?;

        DuplicateHandleHook
            .initialize(
                std::mem::transmute(KERNEL32.get_symbol_address("DuplicateHandle").unwrap()),
                |h_source_process_handle,
                 h_source_handle,
                 h_target_process_handle,
                 lp_target_handle,
                 dw_desired_access,
                 b_inherit_handle,
                 dw_options| {
                    on_duplicate_handle(
                        h_source_process_handle,
                        h_source_handle,
                        h_target_process_handle,
                        lp_target_handle,
                        dw_desired_access,
                        b_inherit_handle,
                        dw_options,
                    )
                },
            )?
            .enable()?;

        CloseHandleHook
            .initialize(
                std::mem::transmute(KERNEL32.get_symbol_address("CloseHandle").unwrap()),
                |h_object| on_close_handle(h_object),
            )?
            .enable()?;
    }

    Ok(())
}
