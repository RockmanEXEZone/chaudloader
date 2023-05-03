use std::os::windows::io::FromRawHandle;

pub fn hijack() -> Result<impl std::io::Read + Send + 'static, get_last_error::Win32Error> {
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

        if winapi::um::processenv::SetStdHandle(winapi::um::winbase::STD_OUTPUT_HANDLE, write_pipe)
            != winapi::shared::minwindef::TRUE
        {
            return Err(get_last_error::Win32Error::get_last_error());
        }

        if winapi::um::processenv::SetStdHandle(winapi::um::winbase::STD_ERROR_HANDLE, write_pipe)
            != winapi::shared::minwindef::TRUE
        {
            return Err(get_last_error::Win32Error::get_last_error());
        }

        Ok(std::fs::File::from_raw_handle(read_pipe))
    }
}
