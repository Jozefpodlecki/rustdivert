use windows::{Win32::{Foundation::{CloseHandle, ERROR_INVALID_PARAMETER, ERROR_SERVICE_ALREADY_RUNNING, ERROR_SERVICE_DOES_NOT_EXIST, ERROR_SERVICE_EXISTS, ERROR_SUCCESS, GetLastError, WAIT_ABANDONED, WAIT_OBJECT_0, WIN32_ERROR}, System::{Registry::{HKEY, HKEY_LOCAL_MACHINE, KEY_SET_VALUE, REG_DWORD, REG_OPEN_CREATE_OPTIONS, REG_SZ, RegCloseKey, RegCreateKeyExA, RegSetValueExA, RegSetValueExW}, Services::{CloseServiceHandle, CreateServiceW, DeleteService, OpenSCManagerW, OpenServiceW, SC_MANAGER_ALL_ACCESS, SERVICE_ALL_ACCESS, SERVICE_DEMAND_START, SERVICE_ERROR_NORMAL, SERVICE_KERNEL_DRIVER, StartServiceW}, Threading::{CreateMutexW, INFINITE, ReleaseMutex, WaitForSingleObject}}}, core::{BOOL, Error, PCWSTR, s, w}};
use windows::Win32::System::Threading::{GetCurrentProcess, IsWow64Process};
use std::{ffi::OsStr, mem::{offset_of, size_of}, os::windows::ffi::OsStrExt};
use log::*;
use crate::{constants::*, filter::WinDivertFilterRaw, newtypes::{CreateArgs, MutexHandle, ServiceManager}, *};

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

pub fn try_install_driver() -> Result<(), WinDivertError> {
    let _ = MutexHandle::new()?;

    debug!("Opening service manager");
    let manager = ServiceManager::new()?;

    debug!("Opening windivert service");
    match manager.open() {
        Ok(Some(service)) => {
            
            debug!("Starting service");
            match service.start() {
                Ok(_) => {
                    service.delete()?;
                    Ok(())
                },
                Err(WinDivertError::ServiceAlreadyRunning) => Ok(()),
                Err(err) => Err(err),
            }
        },
        Ok(None) => {
            let args = CreateArgs::new();
            let driver_sys_name = args.driver_sys_name;

            debug!("Creating windivert service");
            match manager.create(args) {
                Ok(service) => {
                    if let Err(err) = win_divert_register_event_source(driver_sys_name) {
                        error!("Could not register event source");
                    }

                    debug!("Starting service");
                    match service.start() {
                        Ok(_) => {
                            service.delete()?;
                            Ok(())
                        },
                        Err(WinDivertError::ServiceAlreadyRunning) => Ok(()),
                        Err(err) => Err(err),
                    }
                },
                Err(WinDivertError::ServiceExists) => {
                    match manager.open() {
                        Ok(service) => {
                            let service = service.ok_or_else(|| WinDivertError::CorruptedService)?;
                            service.delete()?;
                            Ok(())
                        },
                        Err(err) => Err(err),
                    }
                },
                Err(err) => Err(err),
            }
        },
        Err(err) => Err(err),
    }
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