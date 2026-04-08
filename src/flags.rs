
/**
Flag type required by [`WinDivertOpen()`](fn@super::WinDivertOpen). It follows a builder like style.

Different flags affect how the opened handle behaves. The following flags are supported:
 * `sniff`: This flag opens the WinDivert handle in `packet sniffing` mode. In packet sniffing mode the original packet is not dropped-and-diverted (the default) but copied-and-diverted. This mode is useful for implementing packet sniffing tools similar to those applications that currently use Winpcap.
 * `drop`: This flag indicates that the user application does not intend to read matching packets with [`recv()`](fn@super::WinDivertRecv) (or any of it's variants), instead the packets should be silently dropped. This is useful for implementing simple packet filters using the WinDivert [filter language](https://reqrypt.org/windivert-doc.html#filter_language).
 * `recv_only`: This flags forces the handle into receive only mode which effectively disables [`send()`](fn@super::WinDivertSend) (and any of it's variants). This means that it is possible to block/capture packets or events but not inject them.
 * `send_only`: This flags forces the handle into send only mode which effectively disables [`recv()`](fn@super::WinDivertRecv) (and any of it's variants). This means that it is possible to inject packets or events, but not block/capture them.
 * `no_installs`: This flags causes [`WinDivertOpen`](fn@super::WinDivertOpen) to fail with ERROR_SERVICE_DOES_NOT_EXIST (1060) if the WinDivert driver is not already installed. This flag is useful for querying the WinDivert driver state using [`Reflect`](super::WinDivertLayer::Reflect) layer.
 * `fragments`: If set, the handle will capture inbound IP fragments, but not inbound reassembled IP packets. Otherwise, if not set (the default), the handle will capture inbound reassembled IP packets, but not inbound IP fragments. This flag only affects inbound packets at the [`Network`](super::WinDivertLayer::Network) layer, else the flag is ignored.
Note that any combination of (`snif` | `drop`) or (`recv_only` | `send_only`) are considered invalid.

Some layers have mandatory flags:
 * [`WinDivertLayer::Flow`](type@WinDivertLayer::Flow): (`sniff` | `recv_only`)
 * [`WinDivertLayer::Socket`](type@WinDivertLayer::Socket): `recv_only`
 * [`WinDivertLayer::Reflect`](type@WinDivertLayer::Reflect): (`sniff` | `recv_only`)
*/

#[derive(Debug, Default, Copy, Clone)]
#[repr(transparent)]
pub struct WinDivertFlags(u64);

/// WinDivertFlags builder methods.
impl WinDivertFlags {
    /// Creates a new flag field with all options unset.
    pub const fn new() -> Self {
        Self(0)
    }

    /// Sets `sniff` flag.
    pub const fn set_sniff(mut self) -> Self {
        self.0 |= 0x0001;
        self
    }

    /// Unsets `sniff` flag.
    pub const fn unset_sniff(mut self) -> Self {
        self.0 &= !0x001;
        self
    }

    /// Sets `sniff` flag to `value`.
    pub fn set_sniff_value(&mut self, value: bool) {
        self.0 = (self.0 & !0x0001) | (value as u64);
    }

    /// Sets `drop` flag.
    pub const fn set_drop(mut self) -> Self {
        self.0 |= 0x0002;
        self
    }

    /// Unsets `drop` flag.
    pub const fn unset_drop(mut self) -> Self {
        self.0 &= !0x0002;
        self
    }

    /// Sets `drop` flag to `value`.
    pub fn set_drop_value(&mut self, value: bool) {
        self.0 = (self.0 & !0x0002) | ((value as u64) << 1);
    }

    /// Sets `recv_only` flag
    pub const fn set_recv_only(mut self) -> Self {
        self.0 |= 0x0004;
        self
    }

    /// Unsets `recv_only` flag
    pub const fn unset_recv_only(mut self) -> Self {
        self.0 &= !0x0004;
        self
    }

    /// Sets `recv_only` flag to `value`.
    pub fn set_recv_only_value(&mut self, value: bool) {
        self.0 = (self.0 & !0x0004) | ((value as u64) << 2);
    }

    /// Sets `send_only` flag.
    pub const fn set_send_only(mut self) -> Self {
        self.0 |= 0x0008;
        self
    }

    /// Unsets `send_only` flag.
    pub const fn unset_send_only(mut self) -> Self {
        self.0 &= !0x0008;
        self
    }

    /// Sets `send_only` flag to `value`.
    pub fn set_send_only_value(&mut self, value: bool) {
        self.0 = (self.0 & !0x0008) | ((value as u64) << 3);
    }

    /// Sets `no_installs` flag.
    pub const fn set_no_installs(mut self) -> Self {
        self.0 |= 0x0010;
        self
    }

    /// Unsets `no_installs` flag.
    pub const fn unset_no_installs(mut self) -> Self {
        self.0 &= !0x0010;
        self
    }

    /// Sets `no_installs` flag to `value`.
    pub fn set_no_installs_value(&mut self, value: bool) {
        self.0 = (self.0 & !0x0010) | ((value as u64) << 4);
    }

    /// Sets `fragments` flag.
    pub const fn set_fragments(mut self) -> Self {
        self.0 |= 0x0020;
        self
    }

    /// Unsets `fragments` flag.
    pub const fn unset_fragments(mut self) -> Self {
        self.0 &= !0x0020;
        self
    }

    /// Sets `fragments` flag to `value`.
    pub fn set_fragments_value(&mut self, value: bool) {
        self.0 = (self.0 & !0x0020) | ((value as u64) << 5);
    }
}

impl From<WinDivertFlags> for u64 {
    fn from(flags: WinDivertFlags) -> Self {
        flags.0
    }
}
