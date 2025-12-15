use std::os::windows::io::{AsRawHandle, FromRawHandle};

struct HandleRestorer {
    std_handle: winapi::shared::minwindef::DWORD,
    handle: winapi::um::winnt::HANDLE,
}

impl HandleRestorer {
    unsafe fn new(
        std_handle: winapi::shared::minwindef::DWORD,
    ) -> Result<Self, get_last_error::Win32Error> {
        let handle = unsafe { winapi::um::processenv::GetStdHandle(std_handle) };
        if handle == winapi::um::handleapi::INVALID_HANDLE_VALUE {
            return Err(get_last_error::Win32Error::get_last_error());
        }
        Ok(Self { std_handle, handle })
    }
}

impl Drop for HandleRestorer {
    fn drop(&mut self) {
        unsafe {
            assert!(
                winapi::um::processenv::SetStdHandle(self.std_handle, self.handle)
                    == winapi::shared::minwindef::TRUE,
                "{}",
                get_last_error::Win32Error::get_last_error()
            );
        }
    }
}

unsafe impl Send for HandleRestorer {}

pub struct Console {
    // These fields must be dropped first, so we don't keep writing to the hijacked pipe.
    _old_stdout: HandleRestorer,
    _old_stderr: HandleRestorer,
    _write_pipe: std::fs::File,
    read_pipe: std::fs::File,
}

impl std::io::Read for Console {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        self.read_pipe.read(buf)
    }
}

impl Console {
    pub fn hijack() -> Result<Self, get_last_error::Win32Error> {
        unsafe {
            let mut read_pipe = std::ptr::null_mut();
            let mut write_pipe = std::ptr::null_mut();

            if winapi::um::namedpipeapi::CreatePipe(
                &mut read_pipe,
                &mut write_pipe,
                std::ptr::null_mut(),
                0,
            ) != winapi::shared::minwindef::TRUE
            {
                return Err(get_last_error::Win32Error::get_last_error());
            }

            let read_pipe = std::fs::File::from_raw_handle(read_pipe);
            let write_pipe = std::fs::File::from_raw_handle(write_pipe);

            let old_stdout = HandleRestorer::new(winapi::um::winbase::STD_OUTPUT_HANDLE)?;
            if winapi::um::processenv::SetStdHandle(
                winapi::um::winbase::STD_OUTPUT_HANDLE,
                write_pipe.as_raw_handle(),
            ) != winapi::shared::minwindef::TRUE
            {
                return Err(get_last_error::Win32Error::get_last_error());
            }

            let old_stderr = HandleRestorer::new(winapi::um::winbase::STD_ERROR_HANDLE)?;
            if winapi::um::processenv::SetStdHandle(
                winapi::um::winbase::STD_ERROR_HANDLE,
                write_pipe.as_raw_handle(),
            ) != winapi::shared::minwindef::TRUE
            {
                return Err(get_last_error::Win32Error::get_last_error());
            }

            Ok(Console {
                _old_stdout: old_stdout,
                _old_stderr: old_stderr,
                read_pipe,
                _write_pipe: write_pipe,
            })
        }
    }
}
