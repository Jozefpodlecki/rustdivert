use std::{borrow::Cow, mem::{MaybeUninit, offset_of}};
use log::*;
use windows::{Win32::{Foundation::{CloseHandle, ERROR_FATAL_APP_EXIT, GENERIC_READ, GENERIC_WRITE, GetLastError, HANDLE}, Storage::FileSystem::{CreateFile2, CreateFileW, FILE_ATTRIBUTE_NORMAL, FILE_FLAG_OVERLAPPED, FILE_SHARE_MODE, OPEN_EXISTING}, System::IO::{DeviceIoControl, OVERLAPPED}}, core::w};
use std::marker::Send;

use crate::{constants::*, filter::{WinDivertFilterProgram, WinDivertFilterRaw}, ioctl::*, misc::{sanity_checks, try_install_driver}, *};

unsafe impl Send for Windivert {}
unsafe impl Sync for Windivert {}

// #[derive(Debug, Clone)]
pub struct WinDivertPacket<'a> {
    pub address: WinDivertAddress,
    pub data: Cow<'a, [u8]>,
}

pub struct Windivert {
    handle: HANDLE,
}

impl Windivert {

    pub fn open(layer: WinDivertLayer, filter: &str, priority: u32, flags: WinDivertFlags) -> Result<Self, WinDivertError> {

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

        Ok(Self{
            handle
        })
    }

    pub fn send(&self, packet: &WinDivertPacket) -> bool {
        let ioctl = WinDivertIoctl::send(&packet.address);
        let mut write_len = 0;

        let is_success = unsafe {
            DeviceIoControl(
                self.handle,
                IOCTL_WINDIVERT_SEND,
                Some(&ioctl as *const _ as *const std::ffi::c_void),
                WinDivertIoctl::size_of(),
                    Some(packet.data.as_ptr() as *mut _),
                packet.data.len() as u32,
                Some(&mut write_len),
                None
            ).is_ok()
        };

        is_success
    }

    pub fn send_ex(&self, packets: &[WinDivertPacket]) -> bool {
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
                self.handle,
                IOCTL_WINDIVERT_SEND,
                Some(&ioctl as *const _ as *const std::ffi::c_void),
                WinDivertIoctl::size_of(),
                    Some(buffer.as_ptr() as *mut _),
                buffer.len() as u32,
                Some(&mut write_len),
                None
            ).is_ok()
        };

        is_success
    }

    pub fn set_param(&self, param: WinDivertParam, value: u64) -> Result<(), WinDivertError> {
        let ioctl = WinDivertIoctl::set_param(param, value);
        
        unsafe {
            DeviceIoControl(
                self.handle,
                IOCTL_WINDIVERT_SET_PARAM,
                Some(&ioctl as *const _ as *const std::ffi::c_void),
                WinDivertIoctl::size_of(),
                None,
                0,
                None,
                None
            ).map_err(|error| WinDivertError::DeviceIoControl(error.code().0 as u32))?;
        };

        Ok(())
    }

    pub fn get_param(&self, param: WinDivertParam) -> Result<u64, WinDivertError> {
        let mut value: u64 = 0;
        let ioctl = WinDivertIoctl::get_param(param);

        unsafe {
            DeviceIoControl(
                self.handle,
                IOCTL_WINDIVERT_SET_PARAM,
                Some(&ioctl as *const _ as *const std::ffi::c_void),
                WinDivertIoctl::size_of(),
                Some(&mut value as *mut _ as *mut std::ffi::c_void),
                std::mem::size_of::<u64>() as u32,
                None,
                None
            ).map_err(|error| WinDivertError::DeviceIoControl(error.code().0 as u32))?;
        };

        Ok(value)
    }

    pub fn recv<'a>(&self, buffer: &'a mut [u8]) -> Result<WinDivertPacket<'a>, WinDivertError> {
        let addr: MaybeUninit<WinDivertAddress> = MaybeUninit::uninit();
        let ioctl = WinDivertIoctl::recv(addr.as_ptr());
        let mut read_len = 0;

        unsafe {
            DeviceIoControl(
                self.handle,
                IOCTL_WINDIVERT_RECV,
                Some(&ioctl as *const _ as *const std::ffi::c_void),
                WinDivertIoctl::size_of(),
                Some(buffer.as_ptr() as *mut _),
                buffer.len() as u32,
                Some(&mut read_len),
                None
            ).map_err(|error| WinDivertError::DeviceIoControl(error.code().0 as u32))?;
        };

        Ok(WinDivertPacket {
            address: unsafe { addr.assume_init() },
            data: Cow::Borrowed(&buffer[..read_len as usize]),
        })
    }

    pub fn recv_ex(
        &self,
        p_packet: *mut std::ffi::c_void,
        packet_len: u32,
        read_len: *mut u32,
        overlapped: *mut OVERLAPPED,
    ) -> bool {
        let addr: MaybeUninit<WinDivertAddress> = MaybeUninit::uninit();
        let ioctl = WinDivertIoctl::recv(addr.as_ptr());

        let is_success = unsafe {
            DeviceIoControl(
                self.handle,
                IOCTL_WINDIVERT_SHUTDOWN,
                Some(&ioctl as *const _ as *const std::ffi::c_void),
                WinDivertIoctl::size_of(),
                    Some(p_packet),
                packet_len,
                Some(read_len),
                Some(overlapped)
            ).is_ok()
        };

        is_success
    }

    pub fn shutdown(&self, how: WinDivertShutdown) -> bool {
        let ioctl = WinDivertIoctl::shutdown(how);

        let is_success = unsafe {
            DeviceIoControl(
                self.handle,
                IOCTL_WINDIVERT_SHUTDOWN,
                Some(&ioctl as *const _ as *const std::ffi::c_void),
                WinDivertIoctl::size_of(),
                    None,
                0,
                None,
                None
            ).is_ok()
        };

        is_success
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