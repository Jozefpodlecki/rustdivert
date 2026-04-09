use windows::Win32::{Foundation::{CloseHandle, GENERIC_READ, GENERIC_WRITE, GetLastError, HANDLE}, Storage::FileSystem::{CreateFileW, FILE_ATTRIBUTE_NORMAL, FILE_FLAG_OVERLAPPED, FILE_SHARE_MODE, OPEN_EXISTING}, System::{IO::DeviceIoControl, Threading::{CreateEventW, TLS_OUT_OF_INDEXES, TlsAlloc, TlsGetValue, TlsSetValue}}};
use log::*;

use crate::{WinDivertError, WinDivertFlags, WinDivertIoctl, WinDivertLayer, WinDivertVersion, constants::{IOCTL_WINDIVERT_INITIALIZE, IOCTL_WINDIVERT_STARTUP, WINDIVERT_PIPE_NAME}, filter::{WinDivertFilterProgram, WinDivertFilterRaw}, misc::{sanity_checks, try_install_driver}};

pub struct Windivert {
    handle: HANDLE,
    event: HANDLE,
}

impl Windivert {

    pub fn open(layer: WinDivertLayer, filter: &str, priority: u32, flags: WinDivertFlags) -> Result<Self, WinDivertError> {
        
        let event = unsafe { 
            CreateEventW(None, false, false, None)
            .map_err(|err| WinDivertError::Handle(err.code().0 as u32))?
        };

        unsafe {
            let tls_id = TlsAlloc();
            TlsSetValue(tls_id, Some(event.0));
        }

        // NULL, FALSE, FALSE, NULL
        // TlsGetValue();
        // TlsSetValue();
        //  TlsAlloc())
        //  TLS_OUT_OF_INDEXES

         debug!(
            "Opening WinDivert layer={:?}, filter={:?}, priority={}, flags={:?}",
            layer, filter, priority, flags
        );

        if !sanity_checks() {
            return Err(WinDivertError::InvalidParameter);
        }

        let filter = WinDivertFilterProgram::compile(filter, layer)?;

        debug!("Trying to open handle {}", unsafe { WINDIVERT_PIPE_NAME.display() });
        let handle = unsafe {
            CreateFileW(
                WINDIVERT_PIPE_NAME,
                (GENERIC_READ | GENERIC_WRITE).0,
                FILE_SHARE_MODE(0),
                None,
                OPEN_EXISTING,
                FILE_ATTRIBUTE_NORMAL | FILE_FLAG_OVERLAPPED,
                None,
            ).map_err(|err| WinDivertError::File(err.code().0 as u32))
        };

        if handle.is_err() {
            let error = windows::core::Error::from_thread();
            debug!("Got error {}, trying to install service", error.code().0);

            if !try_install_driver().map_err(|err| WinDivertError::CouldNotInstallService(err.code().0 as u32))? {
                return Err(WinDivertError::Handle(error.code().0 as u32));   
            }

            debug!("2nd attempt to open handle {}", unsafe { WINDIVERT_PIPE_NAME.display() });
            let handle = unsafe {
                CreateFileW(
                    WINDIVERT_PIPE_NAME,
                    (GENERIC_READ | GENERIC_WRITE).0,
                    FILE_SHARE_MODE(0),
                    None,
                    OPEN_EXISTING,
                    FILE_ATTRIBUTE_NORMAL | FILE_FLAG_OVERLAPPED,
                    None,
                ).map_err(|err| WinDivertError::File(err.code().0 as u32))?
            };
        }

        let handle = handle.unwrap();
        debug!("WinDivert handle created: {:?}", handle);
        let ioctl = WinDivertIoctl::initialize(layer as u32, priority, flags.into());
        let mut version = WinDivertVersion::new();

        let is_success = unsafe {
            DeviceIoControl(
                handle,
                IOCTL_WINDIVERT_INITIALIZE,
                Some(&ioctl as *const _ as *const std::ffi::c_void),
                WinDivertIoctl::size_of(),
                Some(&mut version as *mut _ as *mut std::ffi::c_void),
                WinDivertVersion::size_of(),
                None,
                None
            ).is_ok()
        };
        debug!(
            "DeviceIoControl IOCTL_WINDIVERT_INITIALIZE success={}",
            is_success
        );

        if !is_success {
            unsafe {
                CloseHandle(handle).map_err(|err| WinDivertError::Handle(err.code().0 as u32))?;
            }
            let error = unsafe { GetLastError() };
            return Err(WinDivertError::DeviceIoControl(error.0));
        }

        let filter_flags = filter.analyse();
        let ioctl = WinDivertIoctl::startup(filter_flags);
        let size_of = filter.size_of();
        let filter = filter.into_inner();

        let is_success = unsafe {
            DeviceIoControl(
                handle,
                IOCTL_WINDIVERT_STARTUP,
                Some(&ioctl as *const _ as *const std::ffi::c_void),
                WinDivertIoctl::size_of(),
                Some(filter.as_ptr() as *mut std::ffi::c_void), 
                filter.len() as u32 * std::mem::size_of::<WinDivertFilterRaw>() as u32,
                None,
                None
            ).is_ok()
        };
        info!("DeviceIoControl IOCTL_WINDIVERT_STARTUP success={:?}", is_success);

        if !is_success {
            let error = unsafe { GetLastError() };
            unsafe {
                CloseHandle(handle).map_err(|err| WinDivertError::Handle(err.code().0 as u32))?;
            }

            debug!("IOCTL_WINDIVERT_STARTUP {:?}", error);
            return Err(WinDivertError::DeviceIoControl(error.0));
        } 

        Ok(Self {
            handle,
            event
        })
    }

    fn close_inner(&mut self) {
        debug!("Closing handle");
        unsafe { CloseHandle(self.handle).ok(); }
        self.handle = HANDLE::default(); 
    }

    pub fn close(mut self) {
        self.close_inner();
    }
}

impl Drop for Windivert {
    fn drop(&mut self) {
        self.close_inner();
    }
}
