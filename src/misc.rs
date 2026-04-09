use windows::{Win32::{Foundation::{CloseHandle, ERROR_INVALID_PARAMETER, ERROR_SERVICE_ALREADY_RUNNING, ERROR_SERVICE_DOES_NOT_EXIST, ERROR_SERVICE_EXISTS, ERROR_SUCCESS, GetLastError, WAIT_ABANDONED, WAIT_OBJECT_0, WIN32_ERROR}, System::{Registry::{HKEY, HKEY_LOCAL_MACHINE, KEY_SET_VALUE, REG_DWORD, REG_OPEN_CREATE_OPTIONS, REG_SZ, RegCloseKey, RegCreateKeyExA, RegSetValueExA, RegSetValueExW}, Services::{CloseServiceHandle, CreateServiceW, DeleteService, OpenSCManagerW, OpenServiceW, SC_MANAGER_ALL_ACCESS, SERVICE_ALL_ACCESS, SERVICE_DEMAND_START, SERVICE_ERROR_NORMAL, SERVICE_KERNEL_DRIVER, StartServiceW}, Threading::{CreateMutexW, INFINITE, ReleaseMutex, WaitForSingleObject}}}, core::{BOOL, Error, PCWSTR, s, w}};
use windows::Win32::System::Threading::{GetCurrentProcess, IsWow64Process};
use std::{ffi::OsStr, mem::{offset_of, size_of}, os::windows::ffi::OsStrExt};
use log::*;
use crate::{constants::*, filter::WinDivertFilterRaw, *};

pub const fn sanity_checks() -> bool {
    if size_of::<WinDivertAddress>() != 80 {
        return false;
    }

    if size_of::<WinDivertDataNetwork>() != 8 {
        return false;
    }

    if offset_of!(WinDivertDataFlow, protocol) != 56 {
        return false;
    }

    if offset_of!(WinDivertDataSocket, protocol) != 56 {
        return false;
    }

    if offset_of!(WinDivertDataReflect, priority) != 24 {
        return false;
    }
    
    if size_of::<WinDivertFilterRaw>() != 24 {
        return false;
    }

    if offset_of!(WinDivertAddress, data) != 16 {

        return false;
    }

    true
}

fn win_divert_use_32_bit() -> bool {
    if size_of::<usize>() == 8 {
        return false;
    }
    
    let mut is_wow64 = BOOL::default();
    
    unsafe {
        let current_process = GetCurrentProcess();
        match IsWow64Process(current_process, &mut is_wow64) {
            Ok(_) => !is_wow64.as_bool(),
            Err(_) => false,
        }
    }
}

fn win_divert_str_len(s: &[u16], maxlen: usize) -> Option<usize> {
    let len = s.iter()
        .take_while(|&&c| c != 0)
        .count();
    
    if len > maxlen {
        None
    } else {
        Some(len)
    }
}

fn win_divert_get_driver_file_name() -> Option<PCWSTR> {
    let is_32bit = win_divert_use_32_bit();
    
    let driver_name = if is_32bit {
        WINDIVERT_32_SYS
    } else {
        WINDIVERT_64_SYS
    };
    
    // let exe_path = std::env::current_exe().ok()?;
    // let dir_path = exe_path.parent()?;
    // let driver_path = dir_path.join(driver_name);
    
    let driver_path_wide: Vec<u16> = OsStr::new(&driver_name)
        .encode_wide()
        .chain(Some(0))
        .collect();
    let driver_name = PCWSTR::from_raw(driver_path_wide.as_ptr());

    Some(driver_name)
}

pub fn try_install_driver() -> windows::core::Result<bool> {
    
    debug!("Trying to install driver");

    let handle = unsafe {
        CreateMutexW(None, false, WINDIVERT_MUTEX_NAME)?
    };

    if handle.is_invalid() {
        return Ok(false)
    }

    match unsafe { WaitForSingleObject(handle, INFINITE) } {
        WAIT_OBJECT_0 | WAIT_ABANDONED => {},
        _ => return Ok(false)
    }

    debug!("Opening service manager");
    let manager = unsafe{
        OpenSCManagerW(None, None, SC_MANAGER_ALL_ACCESS)?
    };

    if manager.is_invalid() {
        unsafe {
            ReleaseMutex(handle)?;
            CloseHandle(handle)?;
        }

        return Ok(false)
    }

    debug!("Opening windivert service");
    let mut service_result = unsafe {
        OpenServiceW(manager, WINDIVERT_DRIVER_NAME, SERVICE_ALL_ACCESS)
    };

    match service_result {
        Ok(service) => {
            debug!("Service windivert exists");
            unsafe {
                CloseServiceHandle(service)?;
                CloseServiceHandle(manager)?;
                ReleaseMutex(handle)?;
                CloseHandle(handle)?;
            }
            return Ok(true);
        }
        Err(err) => {
            let error_code = err.code();
            if error_code == ERROR_SERVICE_DOES_NOT_EXIST.into() {
                debug!("Service does not exist, will create it");
            } else {
                debug!("Unexpected error opening service: {:?}", err);
                unsafe {
                    CloseServiceHandle(manager)?;
                    ReleaseMutex(handle)?;
                    CloseHandle(handle)?;
                }
                return Err(err);
            }
        }
    };

    debug!("windivert service does not exist");
    let driver_name = match win_divert_get_driver_file_name() {
        Some(value) => value,
        None => {
            unsafe {
                // CloseServiceHandle(service)?;
                CloseServiceHandle(manager)?;
                ReleaseMutex(handle)?;
                CloseHandle(handle)?;
            }

            return Ok(false)
        },
    };

    let binary_path = {
        let bytes = if win_divert_use_32_bit() {
            include_bytes!("../WinDivert32.sys")
        } else {
            include_bytes!("../WinDivert64.sys")
        };

        let exe_path = std::env::current_exe().ok().unwrap();
        let dir_path = exe_path.parent().unwrap();
        let driver_name_str = unsafe { driver_name.to_string().unwrap() };
        let driver_path = dir_path.join(&driver_name_str);
        let system_dir = std::env::var("SystemRoot")
            .unwrap_or_else(|_| "C:\\Windows".to_string());
        let driver_path = std::path::PathBuf::from(system_dir)
            .join("System32")
            .join(&driver_name_str);

        if driver_path.exists() {
            debug!("Found driver in {}", driver_path.display());
        } else {
            debug!("Creating driver in {}", driver_path.display());
            std::fs::write(&driver_path, bytes).unwrap();
        }

        let driver_path_wide: Vec<u16> = OsStr::new(&driver_path)
            .encode_wide()
            .chain(Some(0))
            .collect();
        
        PCWSTR::from_raw(driver_path_wide.as_ptr())
    };

    debug!("Creating windivert service");
    let mut service = unsafe {
        CreateServiceW(manager,
            WINDIVERT_DRIVER_NAME,
            WINDIVERT_DRIVER_NAME,
            SERVICE_ALL_ACCESS,
            SERVICE_KERNEL_DRIVER,
            SERVICE_DEMAND_START,
            SERVICE_ERROR_NORMAL,
            binary_path,
            None,
            None,
            None,
            None,
            None
        )?
    };

    let mut is_success = true;

    if service.is_invalid() {
        let error = unsafe { GetLastError() };
        
        if error == ERROR_SERVICE_EXISTS {
            debug!("Invalid handle, starting service");
            service = unsafe {
                OpenServiceW(
                    manager,
                    WINDIVERT_DRIVER_NAME,
                    SERVICE_ALL_ACCESS,
                )
            }?;
        }

        if !service.is_invalid() {
            is_success = unsafe { StartServiceW(service, None).is_ok() };

            if is_success {
                debug!("Started service, marking as delete on restart");
                unsafe { DeleteService(service)?; }
            }
            else {
                let error = unsafe { GetLastError() };
                is_success = error == ERROR_SERVICE_ALREADY_RUNNING;
            }
        }
    }

    debug!("Registering event soruce");
    win_divert_register_event_source(driver_name)?;

    if !service.is_invalid() {
        debug!("Starting service");
        is_success = unsafe { StartServiceW(service, None).is_ok() };

        if is_success {
            debug!("Started service, marking as delete on restart");
            unsafe { DeleteService(service)?; }
            // NtUnloadDriver(); // use ntapi::ntioapi::NtUnloadDriver;
        }
        else {
            let error = unsafe { GetLastError() };
            debug!("Could not start service, error: {}", error.0);
            is_success = error == ERROR_SERVICE_ALREADY_RUNNING;
        }
    }

    unsafe {
        CloseServiceHandle(service)?;
        CloseServiceHandle(manager)?;
        ReleaseMutex(handle)?;
        CloseHandle(handle)?;
    }

    Ok(is_success)
}

fn win_divert_register_event_source(windivert_sys: PCWSTR) -> windows::core::Result<()> {
    let len = match unsafe { win_divert_str_len(windivert_sys.as_wide(), MAX_PATH) } {
        Some(l) => l,
        None => return Err(Error::from(ERROR_INVALID_PARAMETER)),
    };

    let mut key: HKEY = HKEY(std::ptr::null_mut());
    const REG_OPTION_VOLATILE: REG_OPEN_CREATE_OPTIONS = REG_OPEN_CREATE_OPTIONS(0);
    
    let mut result = unsafe {
        RegCreateKeyExA(
            HKEY_LOCAL_MACHINE,
            EVENTLOG_REGISTRY_PATH,
            Some(0),
            None,
            REG_OPTION_VOLATILE,
            KEY_SET_VALUE,
            None,
            &mut key,
            None,
        )
    };

    if result != ERROR_SUCCESS {
        return Err(Error::from(result));
    }

    let types: u32 = 7;

    unsafe {
        result = RegSetValueExW(
            key,
            w!("EventMessageFile"),
            Some(0),
            REG_SZ,
            Some(std::slice::from_raw_parts(
                windivert_sys.as_ptr() as *const u8,
                (len + 1) * 2,
            )),
        );

        if result != ERROR_SUCCESS {
            RegCloseKey(key);
            return Err(Error::from(result));
        }

        result = RegSetValueExA(
            key,
            s!("TypesSupported"),
            Some(0),
            REG_DWORD,
            Some(std::slice::from_raw_parts(
                &types as *const u32 as *const u8,
                std::mem::size_of::<u32>(),
            )),
        );

        if result != ERROR_SUCCESS {
            RegCloseKey(key);
            return Err(Error::from(result));
        }

        result = RegCloseKey(key);

        if result != ERROR_SUCCESS {
            RegCloseKey(key);
            return Err(Error::from(result));
        }
    }

    Ok(())
}