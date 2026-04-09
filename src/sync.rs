use std::{borrow::Cow, mem::{MaybeUninit, offset_of}};
use log::*;
use windows::{Win32::{Foundation::{CloseHandle, ERROR_FATAL_APP_EXIT, ERROR_FILE_NOT_FOUND, GENERIC_READ, GENERIC_WRITE, GetLastError, HANDLE, STATUS_ACCESS_DENIED}, Storage::FileSystem::{CreateFile2, CreateFileW, FILE_ATTRIBUTE_NORMAL, FILE_FLAG_OVERLAPPED, FILE_SHARE_MODE, OPEN_EXISTING}, System::IO::{DeviceIoControl, OVERLAPPED}}, core::w};
use std::marker::Send;

use crate::{constants::*, filter::{WinDivertFilterProgram, WinDivertFilterRaw}, ioctl::*, misc::{sanity_checks, try_install_driver}, newtypes::WindivertHandle, *};

unsafe impl Send for Windivert {}
unsafe impl Sync for Windivert {}


pub struct Windivert {
    handle: WindivertHandle,
}

impl Windivert {

    pub fn open(options: WindivertOptions, layer: WinDivertLayer, filter: &str, priority: i16, flags: WinDivertFlags) -> Result<Self, WinDivertError> {
        assert!(priority > -30000 && priority < 30000);

        debug!(
            "Opening WinDivert layer={:?}, filter={:?}, priority={}, flags={:?}",
            layer, filter, priority, flags
        );

        if !sanity_checks() {
            return Err(WinDivertError::InvalidParameter);
        }

        let filter = WinDivertFilterProgram::compile(filter, layer)?;

        debug!("Trying to open handle");
        let result = WindivertHandle::open();

        let handle = match result {
            Ok(handle) => handle,
            Err(WinDivertError::FileNotFound) => {
                if options.install_service_on_file_not_found {
                    try_install_driver()?;
                    WindivertHandle::open()?
                }
                else {
                    return Err(WinDivertError::FileNotFound)
                }
            },
            Err(err) => return Err(err)
        };

        debug!("WinDivert handle created: {:?}", handle);

        let priority = (priority + WINDIVERT_PRIORITY_HIGHEST) as u32;
        handle.initialize(layer as u32, priority, flags.into())?;
        let filter_flags = filter.analyse();
        handle.startup(filter, filter_flags)?;

        Ok(Self{
            handle
        })
    }

    pub fn send(&self, packet: &WinDivertPacket) -> Result<u32, WinDivertError> {
        Ok(self.handle.send(packet)?)
    }

    pub fn send_ex(&self, packets: &[WinDivertPacket]) -> Result<u32, WinDivertError> {
        Ok(self.handle.send_ex(packets)?)
    }

    pub fn set_param(&self, param: WinDivertParam, value: u64) -> Result<(), WinDivertError> {
        Ok(self.handle.set_param(param, value)?)
    }

    pub fn get_param(&self, param: WinDivertParam) -> Result<u64, WinDivertError> {
        Ok(self.handle.get_param(param)?)
    }

    pub fn recv(&self) -> Result<WinDivertPacket, WinDivertError> {
        let mut buffer = vec![0u8; 65536]; 
        Ok(self.handle.recv(&mut buffer)?)
    }

    pub fn recv_ex(&self, packet_count: usize) -> Result<WinDivertPacketBatch, WinDivertError> {
        let mut buffer = vec![0u8; 65536]; 
        let buffer_len = buffer.len() as u32;
        let mut addr_len = (WinDivertAddress::size_of() * packet_count as u32) as u64;
        let mut addr_buffer: Vec<WinDivertAddress> = vec![WinDivertAddress::default(); packet_count];

        Ok(self.handle.recv_ex(&mut buffer, buffer_len, &mut addr_buffer, &mut addr_len)?)
    }

    pub fn shutdown(&self, how: WinDivertShutdown) -> bool {
        self.handle.shutdown(how).is_ok()
    }

    pub fn close(self) {}
}