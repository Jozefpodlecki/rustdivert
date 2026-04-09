use std::{borrow::Cow, ffi::OsStr, marker::PhantomData, mem::{MaybeUninit, offset_of}, os::windows::ffi::OsStrExt};
use etherparse::{NetSlice, SlicedPacket};
use log::*;
use windows::{Win32::{Foundation::{CloseHandle, ERROR_FATAL_APP_EXIT, ERROR_FILE_NOT_FOUND, ERROR_SERVICE_ALREADY_RUNNING, ERROR_SERVICE_DOES_NOT_EXIST, ERROR_SERVICE_EXISTS, GENERIC_READ, GENERIC_WRITE, GetLastError, HANDLE, STATUS_ACCESS_DENIED, WAIT_ABANDONED, WAIT_OBJECT_0}, Storage::FileSystem::{CreateFile2, CreateFileW, FILE_ATTRIBUTE_NORMAL, FILE_FLAG_OVERLAPPED, FILE_SHARE_MODE, OPEN_EXISTING}, System::{IO::{DeviceIoControl, OVERLAPPED}, Services::{CloseServiceHandle, CreateServiceW, DeleteService, OpenSCManagerW, OpenServiceW, SC_HANDLE, SC_MANAGER_ALL_ACCESS, SERVICE_ALL_ACCESS, SERVICE_DEMAND_START, SERVICE_ERROR_NORMAL, SERVICE_KERNEL_DRIVER, StartServiceW}, Threading::{CreateMutexW, GetCurrentProcess, INFINITE, IsWow64Process, ReleaseMutex, WaitForSingleObject}}}, core::{BOOL, PCWSTR, w}};
use std::marker::Send;

use crate::{constants::*, filter::{WinDivertFilterProgram, WinDivertFilterRaw}, ioctl::*, misc::{sanity_checks, try_install_driver}, *};
use windows::core::{Error, HRESULT};
use std::ops::Deref;

#[derive(Debug, Clone)]
pub struct WindivertHandle(HANDLE);

unsafe impl Send for WindivertHandle {}
unsafe impl Sync for WindivertHandle {}

#[derive(Debug, Clone, Copy)]
pub struct WindivertEvent(HANDLE);

unsafe impl Send for WindivertEvent {}
unsafe impl Sync for WindivertEvent {}

impl Deref for WindivertEvent {
    type Target = HANDLE;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl WindivertHandle {
    pub fn open() -> Result<Self, WinDivertError> {
        let handle = unsafe {
            CreateFileW(
                WINDIVERT_PIPE_NAME,
                (GENERIC_READ | GENERIC_WRITE).0,
                FILE_SHARE_MODE(0),
                None,
                OPEN_EXISTING,
                FILE_ATTRIBUTE_NORMAL | FILE_FLAG_OVERLAPPED,
                None,
            )
        };

        match handle {
            Ok(handle) => Ok(Self(handle)),
            Err(err) => {
                let h_result: HRESULT = err.code();
                let win32_code = h_result.0 & 0xFFFF;
                println!("Win32 error: {}", win32_code);

                if win32_code == 2 {
                    return Err(WinDivertError::FileNotFound)
                }

                if win32_code == 5 {
                    return Err(WinDivertError::AccessDenied)
                }

                return Err(WinDivertError::CouldNotInitialize(win32_code as u32))
            },
        }
    }

    pub fn initialize(&self, layer: u32, priority: u32, flags: u64) -> Result<(), WinDivertError> {
        let ioctl = WinDivertIoctl::initialize(layer as u32, priority, flags.into());
        let mut version = WinDivertVersion::new();

        unsafe {
            DeviceIoControl(
                self.0,
                IOCTL_WINDIVERT_INITIALIZE,
                Some(&ioctl as *const _ as *const std::ffi::c_void),
                WinDivertIoctl::size_of(),
                Some(&mut version as *mut _ as *mut std::ffi::c_void),
                WinDivertVersion::size_of(),
                None,
                None
            ).map_err(|err| WinDivertError::CouldNotInitialize(err.code().0 as u32))?;
        };

        Ok(())
    }

    pub fn startup(&self, filter: WinDivertFilterProgram, filter_flags: u64) -> Result<(), WinDivertError> {
        let ioctl = WinDivertIoctl::startup(filter_flags);
        let size_of = filter.size_of();
        let filter = filter.into_inner();

        unsafe {
            DeviceIoControl(
                self.0,
                IOCTL_WINDIVERT_STARTUP,
                Some(&ioctl as *const _ as *const std::ffi::c_void),
                WinDivertIoctl::size_of(),
                Some(filter.as_ptr() as *mut std::ffi::c_void), 
                filter.len() as u32 * std::mem::size_of::<WinDivertFilterRaw>() as u32,
                None,
                None
            ).map_err(|err| WinDivertError::CouldNotInitialize(err.code().0 as u32))?;
        };

        Ok(())
    }

    pub fn send(&self, packet: &WinDivertPacket) -> Result<u32, WinDivertError> {

        let ioctl = WinDivertIoctl::send(&packet.address);
        let mut write_len = 0;

        unsafe {
            DeviceIoControl(
                self.0,
                IOCTL_WINDIVERT_SEND,
                Some(&ioctl as *const _ as *const std::ffi::c_void),
                WinDivertIoctl::size_of(),
                    Some(packet.data.as_ptr() as *mut _),
                packet.data.len() as u32,
                Some(&mut write_len),
                None
            ).map_err(|err| WinDivertError::CouldNotSend(err.code().0 as u32))?;
        };

        Ok(write_len)
    }

    pub fn send_ex(&self, packets: &[WinDivertPacket]) -> Result<u32, WinDivertError> {
        let packet_count = packets.len();
        let mut buffer: Vec<u8> = Vec::new();
        let mut address_buffer: Vec<WinDivertAddress> = Vec::with_capacity(packet_count);
        let mut write_len = 0;

        for packet in packets {
            buffer.extend(&packet.data[..]);
            address_buffer.push(packet.address);
        }

        let ioctl = WinDivertIoctl::send_ex(address_buffer.as_ptr(), packet_count as u64);
        let is_success = unsafe {
            DeviceIoControl(
                self.0,
                IOCTL_WINDIVERT_SEND,
                Some(&ioctl as *const _ as *const std::ffi::c_void),
                WinDivertIoctl::size_of(),
                    Some(buffer.as_ptr() as *mut _),
                buffer.len() as u32,
                Some(&mut write_len),
                None
            ).is_ok()
        };

        Ok(write_len)
    }

    pub fn recv_no_data(&self) -> Result<WinDivertAddress, WinDivertError> {
        let addr: MaybeUninit<WinDivertAddress> = MaybeUninit::uninit();
        let ioctl = WinDivertIoctl::recv(addr.as_ptr());

        unsafe {
            DeviceIoControl(
                self.0,
                IOCTL_WINDIVERT_RECV,
                Some(&ioctl as *const _ as *const std::ffi::c_void),
                WinDivertIoctl::size_of(),
                None,
                0,
                None,
                None
            ).map_err(|error| WinDivertError::CouldNotReceive(error.code().0 as u32))?;
        };

        Ok(unsafe { addr.assume_init() })
    }

    pub fn recv(&self, buffer: &mut [u8]) -> Result<WinDivertPacket, WinDivertError> {
        let addr: MaybeUninit<WinDivertAddress> = MaybeUninit::uninit();
        let ioctl = WinDivertIoctl::recv(addr.as_ptr());
        let mut read_len = 0;

        unsafe {
            DeviceIoControl(
                self.0,
                IOCTL_WINDIVERT_RECV,
                Some(&ioctl as *const _ as *const std::ffi::c_void),
                WinDivertIoctl::size_of(),
                Some(buffer.as_ptr() as *mut _),
                buffer.len() as u32,
                Some(&mut read_len),
                None
            ).map_err(|error| WinDivertError::CouldNotReceive(error.code().0 as u32))?;
        };

        Ok(WinDivertPacket {
            received: read_len,
            address: unsafe { addr.assume_init() },
            data: buffer[..read_len as usize].into(),
        })
    }

    pub fn recv_ex(
        &self,
        buffer: &mut [u8],
        buffer_len: u32,
        addr_buffer: &mut [WinDivertAddress],
        addr_len: &mut u64
    ) -> Result<WinDivertPacketBatch, WinDivertError> {
        let ioctl = WinDivertIoctl::recv_ex(addr_buffer.as_ptr(), addr_len);
        let mut read_len = 0;

        unsafe {
            DeviceIoControl(
                self.0,
                IOCTL_WINDIVERT_RECV,
                Some(&ioctl as *const _ as *const std::ffi::c_void),
                WinDivertIoctl::size_of(),
                    Some(buffer.as_ptr() as *mut _),
                buffer_len,
                Some(&mut read_len),
                None
            ).map_err(|error| WinDivertError::CouldNotReceive(error.code().0 as u32))?;
        };

        let mut packets = vec![];
        let mut buffer = &buffer[..];
        let effective_len = (*addr_len / WinDivertAddress::size_of() as u64) as usize;
        for addr in &addr_buffer[..effective_len] {
            
            let headers = SlicedPacket::from_ip(buffer)
                .map_err(|error| WinDivertError::CouldNotReceive(0))?;

                let offset = match headers.net.unwrap() {
                    NetSlice::Ipv4(slice) => slice.header().total_len() as usize,
                    NetSlice::Ipv6(slice) => slice.header().payload_length() as usize + 40,
                    NetSlice::Arp(slice) => unreachable!(),
                };
                let (data, tail) = buffer.split_at(offset);
                buffer = tail;
            
            packets.push(WinDivertPacket {
                received: read_len,
                address: addr.to_owned(),
                data: data.into(),
            });
        }

        Ok(WinDivertPacketBatch {
            received: read_len,
            packets
        })
    }

    // pub fn recv_ex_overlapped(&self) -> Result<WinDivertPacket, WinDivertError> {
    //     let addr: MaybeUninit<WinDivertAddress> = MaybeUninit::uninit();
    //     let ioctl = WinDivertIoctl::recv(addr.as_ptr());

    //     let is_success = unsafe {
    //         DeviceIoControl(
    //             self.0,
    //             IOCTL_WINDIVERT_RECV,
    //             Some(&ioctl as *const _ as *const std::ffi::c_void),
    //             WinDivertIoctl::size_of(),
    //                 Some(p_packet),
    //             packet_len,
    //             Some(read_len),
    //             Some()
    //         ).is_ok()
    //     };

    //     is_success
    // }

    pub fn set_param(&self, param: WinDivertParam, value: u64) -> Result<(), WinDivertError> {
        let ioctl = WinDivertIoctl::set_param(param, value);
        
        unsafe {
            DeviceIoControl(
                self.0,
                IOCTL_WINDIVERT_SET_PARAM,
                Some(&ioctl as *const _ as *const std::ffi::c_void),
                WinDivertIoctl::size_of(),
                None,
                0,
                None,
                None
            ).map_err(|error| WinDivertError::CouldNotSetParam(error.code().0 as u32))?;
        };

        Ok(())
    }

    pub fn get_param(&self, param: WinDivertParam) -> Result<u64, WinDivertError> {
        let mut value: u64 = 0;
        let ioctl = WinDivertIoctl::get_param(param);

        unsafe {
            DeviceIoControl(
                self.0,
                IOCTL_WINDIVERT_GET_PARAM,
                Some(&ioctl as *const _ as *const std::ffi::c_void),
                WinDivertIoctl::size_of(),
                Some(&mut value as *mut _ as *mut std::ffi::c_void),
                std::mem::size_of::<u64>() as u32,
                None,
                None
            ).map_err(|error| {
                WinDivertError::CouldNotGetParam(error.code().0 as u32)
            })?;
        };

        Ok(value)
    }

    pub fn shutdown(&self, how: WinDivertShutdown) -> Result<(), WinDivertError> {
        let ioctl = WinDivertIoctl::shutdown(how);

        unsafe {
            DeviceIoControl(
                self.0,
                IOCTL_WINDIVERT_SHUTDOWN,
                Some(&ioctl as *const _ as *const std::ffi::c_void),
                WinDivertIoctl::size_of(),
                    None,
                0,
                None,
                None
            ).map_err(|err| WinDivertError::CouldNotSend(err.code().0 as u32))?;
        };

        Ok(())
    }
}

impl Drop for WindivertHandle {
    fn drop(&mut self) {
        unsafe {
            CloseHandle(self.0).ok();
        }
    }
}

impl Deref for WindivertHandle {
    type Target = HANDLE;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

pub struct MutexHandle(HANDLE);

impl MutexHandle {
    pub fn new() -> Result<Self, WinDivertError> {
        unsafe {
            let handle = CreateMutexW(None, false, WINDIVERT_MUTEX_NAME)
                .map_err(|err| WinDivertError::CouldNotLockForInstall)?;
            let result = WaitForSingleObject(handle, INFINITE);

            match result {
                WAIT_OBJECT_0 | WAIT_ABANDONED => Ok(Self(handle)),
                _ => {
                    ReleaseMutex(handle).ok();
                    CloseHandle(handle).ok();
                    return Err(WinDivertError::CouldNotLockForInstall)
                }
            }
        }
    }
}

impl Drop for MutexHandle {
    fn drop(&mut self) {
        unsafe {
            ReleaseMutex(self.0).ok();
            CloseHandle(self.0).ok();
        }
    }
}

pub struct CreateArgs {
    pub driver_sys_name: PCWSTR,
    pub binary_path: PCWSTR
}

impl CreateArgs {
    fn use_32_bit() -> bool {
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

    pub fn new() -> Self {
    
        let (driver_name_str, bytes) = if Self::use_32_bit() {
            (WINDIVERT_32_SYS, include_bytes!("../WinDivert32.sys"))
        } else {
            (WINDIVERT_64_SYS, include_bytes!("../WinDivert64.sys"))
        };

        // let exe_path = std::env::current_exe().ok().unwrap();
        // let dir_path = exe_path.parent().unwrap();
        // let driver_name_str = unsafe { driver_name.to_string().unwrap() };
        // let driver_path = dir_path.join(&driver_name_str);
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

        let driver_name_wide: Vec<u16> = OsStr::new(&driver_name_str)
            .encode_wide()
            .chain(Some(0))
            .collect();

        let driver_path_wide: Vec<u16> = OsStr::new(&driver_path)
            .encode_wide()
            .chain(Some(0))
            .collect();
        
        let driver_sys_name = PCWSTR::from_raw(driver_name_wide.as_ptr());
        let binary_path = PCWSTR::from_raw(driver_path_wide.as_ptr());

        Self {
            binary_path,
            driver_sys_name
        }
    }
}

pub struct ServiceManager(SC_HANDLE);

impl ServiceManager {
    pub fn new() -> Result<Self, WinDivertError> {
        unsafe {
            let handle = OpenSCManagerW(None, None, SC_MANAGER_ALL_ACCESS)
                .map_err(|err| WinDivertError::CouldNotInstallService(err.code().0 as u32))?;
            Ok(Self(handle))
        }
    }

    pub fn create(&self, args: CreateArgs) -> Result<WindivertService, WinDivertError> {

        let service = unsafe {
            CreateServiceW(self.0,
                WINDIVERT_DRIVER_NAME,
                WINDIVERT_DRIVER_NAME,
                SERVICE_ALL_ACCESS,
                SERVICE_KERNEL_DRIVER,
                SERVICE_DEMAND_START,
                SERVICE_ERROR_NORMAL,
                args.binary_path,
                None,
                None,
                None,
                None,
                None
            )
        };
        
        match service {
            Ok(handle) => Ok(WindivertService {
                handle,
                _marker: PhantomData,
            }),
            Err(err) => {
                if err.code().0 == ERROR_SERVICE_EXISTS.0 as i32 {
                    return Err(WinDivertError::ServiceExists)
                }

                return Err(WinDivertError::CouldNotInstallService(err.code().0 as u32))
            }
        }
    }

    pub fn open(&self) -> Result<Option<WindivertService>, WinDivertError> {
        unsafe {
            let handle = OpenServiceW(self.0, WINDIVERT_DRIVER_NAME, SERVICE_ALL_ACCESS);
            
            match handle {
                Ok(handle) => Ok(Some(WindivertService {
                    handle,
                    _marker: PhantomData,
                })),
                Err(err) => {
                    if err == ERROR_SERVICE_DOES_NOT_EXIST.into() {
                        return Ok(None)
                    }

                    return Err(WinDivertError::CouldNotInstallService(err.code().0 as u32))
                },
            }
        }
    }
}

impl Drop for ServiceManager {
    fn drop(&mut self) {
        unsafe {
            CloseServiceHandle(self.0).ok();
        }
    }
}

pub struct WindivertService<'a> {
    handle: SC_HANDLE,
    _marker: PhantomData<&'a ServiceManager>,
}

impl<'a> WindivertService<'a> {
    pub fn start(&self) -> Result<(), WinDivertError> {
        let result = unsafe { StartServiceW(self.handle, None) };

        match result {
            Ok(_) => Ok(()),
            Err(err) => {
                if err == ERROR_SERVICE_ALREADY_RUNNING.into() {
                    return Err(WinDivertError::ServiceAlreadyRunning)
                }

                return Err(WinDivertError::CouldNotInstallService(err.code().0 as u32))
            },
        }
    }

    pub fn delete(&self) -> Result<(), WinDivertError> {
        unsafe {
            DeleteService(self.handle)
                .map_err(|err| WinDivertError::CouldNotMarkServiceForDeletion(err.code().0 as u32))?;
        }

        Ok(())
    }
}

impl<'a> Drop for WindivertService<'a> {
    fn drop(&mut self) {
        unsafe {
            CloseServiceHandle(self.handle).ok();
        }
    }
}