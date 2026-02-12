use windows_sys::Win32::{
    Foundation::{CloseHandle, LocalFree, HANDLE},
    Security::{
        Authorization::ConvertSidToStringSidW, GetTokenInformation, TokenUser, TOKEN_QUERY,
    },
    Storage::FileSystem::GetLogicalDrives,
    System::Threading::{GetCurrentProcess, OpenProcessToken},
};

use crate::error::{AppError, AppResult};

pub fn get_current_user_sid() -> AppResult<String> {
    let mut token: HANDLE = std::ptr::null_mut();

    unsafe {
        if OpenProcessToken(GetCurrentProcess(), TOKEN_QUERY, &mut token) == 0 {
            return Err(AppError::PermissionDenied(
                "Failed to open process token".to_string(),
            ));
        }

        let mut size: u32 = 0;
        GetTokenInformation(token, TokenUser, std::ptr::null_mut(), 0, &mut size);

        let mut buffer = vec![0u8; size as usize];
        if GetTokenInformation(
            token,
            TokenUser,
            buffer.as_mut_ptr() as *mut _,
            size,
            &mut size,
        ) == 0
        {
            CloseHandle(token);
            return Err(AppError::FileSystemError(
                "Failed to get token information".to_string(),
            ));
        }

        let token_user = &*(buffer.as_ptr() as *const windows_sys::Win32::Security::TOKEN_USER);
        let sid = token_user.User.Sid;

        let mut sid_string: *mut u16 = std::ptr::null_mut();
        if ConvertSidToStringSidW(sid, &mut sid_string) == 0 {
            CloseHandle(token);
            return Err(AppError::FileSystemError(
                "Failed to convert SID to string".to_string(),
            ));
        }

        let len = (0..).take_while(|&i| *sid_string.offset(i) != 0).count();
        let slice = std::slice::from_raw_parts(sid_string, len);
        let result = String::from_utf16_lossy(slice);

        LocalFree(sid_string as *mut _);
        CloseHandle(token);

        Ok(result)
    }
}

pub fn get_available_drives() -> Vec<String> {
    let mut drives = Vec::new();

    let drive_mask = unsafe { GetLogicalDrives() };

    for i in 0..26 {
        if drive_mask & (1 << i) != 0 {
            let drive_letter = (b'A' + i) as char;
            drives.push(format!("{}:\\", drive_letter));
        }
    }

    drives
}
