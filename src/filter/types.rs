use std::{fmt::{self, Debug, Display}, mem::ManuallyDrop, ptr::{addr_of, addr_of_mut}};

use crate::constants::*;

#[derive(Debug, Default, Clone, Copy)]
#[repr(C, packed)]
pub struct WinDivertFilterRaw {
    word1: u32,
    word2: u32,
    arg: [u32; 4],
}

impl WinDivertFilterRaw {

    pub fn args(&self) -> [u32; 4] {
        unsafe {
            [
                std::ptr::read_unaligned(addr_of!(self.arg[0])),
                std::ptr::read_unaligned(addr_of!(self.arg[1])),
                std::ptr::read_unaligned(addr_of!(self.arg[2])),
                std::ptr::read_unaligned(addr_of!(self.arg[3])),
            ]
        }
    }

    pub fn is_simple_predicate(&self) -> bool {
        self.neg() == 0 &&
        self.arg[1] == 0 &&
        self.arg[2] == 0 &&
        self.arg[3] == 0
    }

    pub fn nth_arg(&self, index: usize) -> u32 {
        unsafe { addr_of!(self.arg[index]).read_unaligned() }
    }

    pub fn set_nth_arg(&mut self, index: usize, value: u32) {
        unsafe { addr_of_mut!(self.arg[index]).write_unaligned(value) }
    }

    pub fn set_args(&mut self, value: &[u32; 4]) {
        for i in 0..4 {
            unsafe {
                std::ptr::write_unaligned(addr_of!(self.arg[i]) as *mut u32, value[i]);
            }
        }
    }

    pub fn reset_args(&mut self) {
        self.arg = [0; 4];
    }

    pub fn set_field(&mut self, v: FilterField) {
        self.word1 = (self.word1 & !0x7FF) | (v as u32 & 0x7FF);
    }

    pub fn set_test(&mut self, value: FilterTest) {
        self.word1 = (self.word1 & !(0x1F << 11)) | ((value as u32 & 0x1F) << 11);
    }

    pub fn set_success(&mut self, v: u16) {
        self.word1 = (self.word1 & !(0xFFFF << 16)) | ((v as u32) << 16);
    }

    pub fn set_failure(&mut self, v: u16) {
        self.word2 = (self.word2 & !0xFFFF) | (v as u32);
    }

    pub fn set_neg(&mut self, v: u32) {
        self.word2 = (self.word2 & !(1 << 16)) | ((v & 1) << 16);
    }

    pub fn field(&self) -> FilterField {
        let filter = self.word1 & 0x7FF;
        filter.into()
    }

    pub fn test(&self) -> FilterTest {
        let test = (self.word1 >> 11) & 0x1F;
        test.into()
    }

    pub fn success(&self) -> u16 {
        ((self.word1 >> 16) & 0xFFFF) as u16
    }

    pub fn failure(&self) -> u16 {
        (self.word2 & 0xFFFF) as u16
    }

    pub fn neg(&self) -> u32 {
        (self.word2 >> 16) & 1
    }
}

#[derive(Clone)]
pub struct Expression {
    pub data: ExpressionData,
    pub kind: TokenKind,
    pub count: u8,
    pub neg: bool,
    pub succ: u16,
    pub fail: u16,
}

impl Default for Expression {
    fn default() -> Self {
        Self {
            data: ExpressionData::Number { values: [0; 4], neg: false },
            kind: Default::default(),
            count: 0,
            neg: false,
            succ: 0,
            fail: 0,
        }
    }
}

impl Display for Expression {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.data {
            ExpressionData::Binary { left, right } => match self.kind {
                TokenKind::And => write!(f, "({} AND {}) [succ={}, fail={}]", left, right, self.succ, self.fail),
                TokenKind::Or => write!(f, "({} OR {}) [succ={}, fail={}]", left, right, self.succ, self.fail),
                TokenKind::Eq => write!(f, "{} == {} [succ={}, fail={}]", left, right, self.succ, self.fail),
                TokenKind::Neq => write!(f, "{} != {} [succ={}, fail={}]", left, right, self.succ, self.fail),
                _ => write!(f, "{:?} [succ={}, fail={}]", self.kind, self.succ, self.fail),
            },
            ExpressionData::Ternary { cond, then_expr, else_expr } => {
                write!(f, "({} ? {} : {}) [succ={}, fail={}]", cond, then_expr, else_expr, self.succ, self.fail)
            }
            ExpressionData::Number { values, .. } => write!(f, "Number({:?}) [succ={}, fail={}]", values, self.succ, self.fail),
            ExpressionData::Var(kind) => write!(f, "Var({:?}) [succ={}, fail={}]", kind, self.succ, self.fail),
        }
    }
}

impl Debug for Expression {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut debug = f.debug_struct("Expression");
        debug.field("kind", &self.kind);
        debug.field("count", &self.count);
        debug.field("neg", &self.neg);
        debug.field("succ", &self.succ);
        debug.field("fail", &self.fail);

        match &self.data {
            ExpressionData::Var(kind) => { debug.field("var_kind", kind); },
            ExpressionData::Number { values, neg } => { debug.field("val", values); debug.field("val_neg", neg); },
            ExpressionData::Binary { left, right } => { debug.field("left", left); debug.field("right", right); },
            ExpressionData::Ternary { cond, then_expr, else_expr } => {
                debug.field("cond", cond);
                debug.field("then_expr", then_expr);
                debug.field("else_expr", else_expr);
            }
        }

        debug.finish()
    }
}


impl Expression {
    pub fn take_binary(self) -> (Expression, Expression) {
        match self.data {
            ExpressionData::Binary { left, right } => (*left, *right),
            _ => unreachable!(),
        }
    }

    pub fn take_ternary(self) -> (Expression, Expression, Expression) {
        match self.data {
            ExpressionData::Ternary { cond, then_expr, else_expr } => {
                (*cond, *then_expr, *else_expr)
            }
            _ => unreachable!(),
        }
    }

    pub fn boxed_default() -> Box<Self> {
        Box::new(Expression::default())
    }

    pub fn new_var(kind: TokenKind) -> Box<Self> {
        Box::new(Self {
            data: ExpressionData::Var(kind),
            kind,
            count: 0,
            neg: false,
            succ: 0,
            fail: 0,
        })
    }

    pub fn new_number(values: [u32; 4], neg: bool) -> Box<Self> {
        Box::new(Self {
            data: ExpressionData::Number { values, neg },
            kind: TokenKind::Number,
            count: 0,
            neg,
            succ: 0,
            fail: 0,
        })
    }

    pub fn new_one() -> Box<Self> {
        Self::new_number([1, 0, 0, 0], false)
    }

    pub fn new_zero() -> Box<Self> {
        Self::new_number([0, 0, 0, 0], false)
    }

    pub fn new_bin_op(kind: TokenKind, left: Box<Expression>, right: Box<Expression>) -> Box<Self> {
        Box::new(Self {
            data: ExpressionData::Binary { left, right },
            kind,
            count: 0,
            neg: false,
            succ: 0,
            fail: 0,
        })
    }

    pub fn new_ternary(cond: Box<Expression>, then_expr: Box<Expression>, else_expr: Box<Expression>) -> Box<Self> {
        Box::new(Self {
            data: ExpressionData::Ternary { cond, then_expr, else_expr },
            kind: TokenKind::Question,
            count: 0,
            neg: false,
            succ: 0,
            fail: 0,
        })
    }

    pub fn eq(var: Box<Expression>, val: Box<Expression>, count: u8, neg: bool, succ: u16, fail: u16) -> Box<Self> {
        Box::new(Self {
            data: ExpressionData::Binary { left: var, right: val },
            kind: TokenKind::Eq,
            count,
            neg,
            succ,
            fail,
        })
    }

    pub fn simplify_test(&mut self) {
        if let (Some(var), Some(val)) = (self.data.first(), self.data.second()) {
            if let Some(info) = VarInfo::from_kind(var.kind) {
                match info.compare(self.kind, val) {
                    Some(true) => {
                        self.kind = TokenKind::Eq;
                        self.data = ExpressionData::Binary {
                            left: Expression::new_var(info.var_type),
                            right: Expression::new_one(),
                        };
                    }
                    Some(false) => {
                        self.kind = TokenKind::Eq;
                        self.data = ExpressionData::Binary {
                            left: Expression::new_var(info.var_type),
                            right: Expression::new_zero(),
                        };
                    }
                    None => return,
                }
            }
        }
    }

    pub fn shortcut_branch(&self, succ: i16, fail: i16) -> Option<i16> {
        if self.kind != TokenKind::Eq { return None; }
        let var = self.data.first()?;
        if var.kind != TokenKind::True { return None; }
        let val = self.data.second()?;
        if let Some(val_arr) = val.data.values() {
            let value = val_arr[0];
            return Some(if value != 0 { succ } else { fail });
        }
        None
    }

    pub fn array_offset(&self) -> Option<u32> {
        match self.kind {
            TokenKind::Packet | TokenKind::Packet16 | TokenKind::Packet32
            | TokenKind::TcpPayload | TokenKind::TcpPayload16 | TokenKind::TcpPayload32
            | TokenKind::UdpPayload | TokenKind::UdpPayload16 | TokenKind::UdpPayload32 => {
                self.data.values().map(|v| v[0])
            }
            _ => None,
        }
    }
}

#[derive(Clone)]
pub enum ExpressionData {
    Var(TokenKind),

    Number {
        values: [u32; 4],
        neg: bool,
    },

    Binary {
        left: Box<Expression>,
        right: Box<Expression>,
    },

    Ternary {
        cond: Box<Expression>,
        then_expr: Box<Expression>,
        else_expr: Box<Expression>,
    },
}

impl std::fmt::Debug for ExpressionData {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ExpressionData::Var(kind) => write!(f, "Var({:?})", kind),
            ExpressionData::Number { values, neg } => write!(f, "Num({:?}{})", values, if *neg { " neg" } else { "" }),
            ExpressionData::Binary { left, right } => write!(f, "({:?} ?? {:?})", left, right),
            ExpressionData::Ternary { cond, then_expr, else_expr } => write!(f, "({:?} ? {:?} : {:?})", cond, then_expr, else_expr),
        }
    }
}

impl ExpressionData {
    pub fn first(&self) -> Option<&Box<Expression>> {
        match self {
            ExpressionData::Binary { left, .. } => Some(left),
            ExpressionData::Ternary { cond, .. } => Some(cond),
            _ => None,
        }
    }

    pub fn second(&self) -> Option<&Box<Expression>> {
        match self {
            ExpressionData::Binary { right, .. } => Some(right),
            ExpressionData::Ternary { then_expr, .. } => Some(then_expr),
            _ => None,
        }
    }

    pub fn third(&self) -> Option<&Box<Expression>> {
        match self {
            ExpressionData::Ternary { else_expr, .. } => Some(else_expr),
            _ => None,
        }
    }

    pub fn values(&self) -> Option<&[u32; 4]> {
        match self {
            ExpressionData::Number { values, .. } => Some(values),
            _ => None,
        }
    }
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum TokenKind {
    #[default]
    Icmp = 0,
    IcmpBody = 1,
    IcmpChecksum = 2,
    IcmpCode = 3,
    IcmpType = 4,
    Icmpv6 = 5,
    Icmpv6Body = 6,
    Icmpv6Checksum = 7,
    Icmpv6Code = 8,
    Icmpv6Type = 9,
    Ip = 10,
    IpChecksum = 11,
    IpDf = 12,
    IpDstAddr = 13,
    IpFragOff = 14,
    IpHeaderLength = 15,
    IpId = 16,
    IpLength = 17,
    IpMf = 18,
    IpProtocol = 19,
    IpSrcAddr = 20,
    IpTos = 21,
    IpTtl = 22,
    Ipv6 = 23,
    Ipv6DstAddr = 24,
    Ipv6FlowLabel = 25,
    Ipv6HopLimit = 26,
    Ipv6Length = 27,
    Ipv6NextHdr = 28,
    Ipv6SrcAddr = 29,
    Ipv6TrafficClass = 30,
    Tcp = 31,
    TcpAck = 32,
    TcpAckNum = 33,
    TcpChecksum = 34,
    TcpDstPort = 35,
    TcpFin = 36,
    TcpHeaderLength = 37,
    TcpPayload = 38,
    TcpPayload16 = 39,
    TcpPayload32 = 40,
    TcpPayloadLength = 41,
    TcpPsh = 42,
    TcpRst = 43,
    TcpSeqNum = 44,
    TcpSrcPort = 45,
    TcpSyn = 46,
    TcpUrg = 47,
    TcpUrgPtr = 48,
    TcpWindow = 49,
    Udp = 50,
    UdpChecksum = 51,
    UdpDstPort = 52,
    UdpLength = 53,
    UdpPayload = 54,
    UdpPayload16 = 55,
    UdpPayload32 = 56,
    UdpPayloadLength = 57,
    UdpSrcPort = 58,
    Zero = 59,
    Event = 60,
    Random8 = 61,
    Random16 = 62,
    Random32 = 63,
    Packet = 64,
    Packet16 = 65,
    Packet32 = 66,
    Length = 67,
    Timestamp = 68,
    True = 69,
    False = 70,
    Inbound = 71,
    Outbound = 72,
    Fragment = 73,
    IfIdx = 74,
    SubIfIdx = 75,
    Loopback = 76,
    Impostor = 77,
    ProcessId = 78,
    LocalAddr = 79,
    RemoteAddr = 80,
    LocalPort = 81,
    RemotePort = 82,
    Protocol = 83,
    EndpointId = 84,
    ParentEndpointId = 85,
    Layer = 86,
    Priority = 87,
    Flow = 88,
    Socket = 89,
    Network = 90,
    NetworkForward = 91,
    Reflect = 92,
    EventPacket = 93,
    EventEstablished = 94,
    EventDeleted = 95,
    EventBind = 96,
    EventConnect = 97,
    EventListen = 98,
    EventAccept = 99,
    EventOpen = 100,
    EventClose = 101,
    MacroTrue = 102,
    MacroFalse = 103,
    MacroTcp = 104,
    MacroUdp = 105,
    MacroIcmp = 106,
    MacroIcmpv6 = 107,
    Open = 108,
    Close = 109,
    SquareOpen = 110,
    SquareClose = 111,
    Minus = 112,
    Bytes = 113,
    Eq = 114,
    Neq = 115,
    Lt = 116,
    Leq = 117,
    Gt = 118,
    Geq = 119,
    Not = 120,
    And = 121,
    Or = 122,
    Colon = 123,
    Question = 124,
    Number = 125,
    End = 126,
}

impl TokenKind {
    pub fn to_filter_test(&self) -> Option<FilterTest> {
        use TokenKind::*;
        match self {
            Eq => Some(FilterTest::Eq),
            Neq => Some(FilterTest::Neq),
            Lt => Some(FilterTest::Lt),
            Leq => Some(FilterTest::Leq),
            Gt => Some(FilterTest::Gt),
            Geq => Some(FilterTest::Geq),
            _ => None,
        }
    }
}

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FilterField {
    Zero = 0,
    Inbound = 1,
    Outbound = 2,
    IfIdx = 3,
    SubIfIdx = 4,
    Ip = 5,
    Ipv6 = 6,
    Icmp = 7,
    Tcp = 8,
    Udp = 9,
    Icmpv6 = 10,
    IpHeaderLength = 11,
    IpTos = 12,
    IpLength = 13,
    IpId = 14,
    IpDf = 15,
    IpMf = 16,
    IpFragOff = 17,
    IpTTL = 18,
    IpProtocol = 19,
    IpChecksum = 20,
    IpSrcAddr = 21,
    IpDstAddr = 22,
    Ipv6TrafficClass = 23,
    Ipv6FlowLabel = 24,
    Ipv6Length = 25,
    Ipv6NextHdr = 26,
    Ipv6HopLimit = 27,
    Ipv6SrcAddr = 28,
    Ipv6DstAddr = 29,
    IcmpType = 30,
    IcmpCode = 31,
    IcmpChecksum = 32,
    IcmpBody = 33,
    Icmpv6Type = 34,
    Icmpv6Code = 35,
    Icmpv6Checksum = 36,
    Icmpv6Body = 37,
    TcpSrcPort = 38,
    TcpDstPort = 39,
    TcpSeqNum = 40,
    TcpAckNum = 41,
    TcpHeaderLength = 42,
    TcpUrg = 43,
    TcpAck = 44,
    TcpPsh = 45,
    TcpRst = 46,
    TcpSyn = 47,
    TcpFin = 48,
    TcpWindow = 49,
    TcpChecksum = 50,
    TcpUrgPtr = 51,
    TcpPayloadLength = 52,
    UdpSrcPort = 53,
    UdpDstPort = 54,
    UdpLength = 55,
    UdpChecksum = 56,
    UdpPayloadLength = 57,
    Loopback = 58,
    Impostor = 59,
    ProcessId = 60,
    LocalAddr = 61,
    RemoteAddr = 62,
    LocalPort = 63,
    RemotePort = 64,
    Protocol = 65,
    EndpointId = 66,
    ParentEndpointId = 67,
    Layer = 68,
    Priority = 69,
    Event = 70,
    Packet = 71,
    Packet16 = 72,
    Packet32 = 73,
    TcpPayload = 74,
    TcpPayload16 = 75,
    TcpPayload32 = 76,
    UdpPayload = 77,
    UdpPayload16 = 78,
    UdpPayload32 = 79,
    Length = 80,
    Timestamp = 81,
    Random8 = 82,
    Random16 = 83,
    Random32 = 84,
    Fragment = 85
}

impl From<u8> for FilterField {
    fn from(value: u8) -> Self {
        unsafe { std::mem::transmute(value) }
    }
}

impl From<u32> for FilterField {
    fn from(value: u32) -> Self {
        unsafe { std::mem::transmute(value as u8) }
    }
}

impl From<FilterField> for u8 {
    fn from(value: FilterField) -> Self {
        value as u8
    }
}

impl From<FilterField> for u32 {
    fn from(value: FilterField) -> Self {
        value as u32
    }
}

impl From<TokenKind> for FilterField {
    fn from(kind: TokenKind) -> Self {
        match kind {
            TokenKind::Zero => FilterField::Zero,
            TokenKind::Event => FilterField::Event,
            TokenKind::Random8 => FilterField::Random8,
            TokenKind::Random16 => FilterField::Random16,
            TokenKind::Random32 => FilterField::Random32,
            TokenKind::Packet => FilterField::Packet,
            TokenKind::Packet16 => FilterField::Packet16,
            TokenKind::Packet32 => FilterField::Packet32,
            TokenKind::Length => FilterField::Length,
            TokenKind::Timestamp => FilterField::Timestamp,
            TokenKind::TcpPayload => FilterField::TcpPayload,
            TokenKind::TcpPayload16 => FilterField::TcpPayload16,
            TokenKind::TcpPayload32 => FilterField::TcpPayload32,
            TokenKind::UdpPayload => FilterField::UdpPayload,
            TokenKind::UdpPayload16 => FilterField::UdpPayload16,
            TokenKind::UdpPayload32 => FilterField::UdpPayload32,
            TokenKind::Outbound => FilterField::Outbound,
            TokenKind::Inbound => FilterField::Inbound,
            TokenKind::Fragment => FilterField::Fragment,
            TokenKind::IfIdx => FilterField::IfIdx,
            TokenKind::SubIfIdx => FilterField::SubIfIdx,
            TokenKind::Loopback => FilterField::Loopback,
            TokenKind::Impostor => FilterField::Impostor,
            TokenKind::ProcessId => FilterField::ProcessId,
            TokenKind::LocalAddr => FilterField::LocalAddr,
            TokenKind::RemoteAddr => FilterField::RemoteAddr,
            TokenKind::LocalPort => FilterField::LocalPort,
            TokenKind::RemotePort => FilterField::RemotePort,
            TokenKind::Protocol => FilterField::Protocol,
            TokenKind::EndpointId => FilterField::EndpointId,
            TokenKind::ParentEndpointId => FilterField::ParentEndpointId,
            TokenKind::Layer => FilterField::Layer,
            TokenKind::Priority => FilterField::Priority,
            TokenKind::Ip => FilterField::Ip,
            TokenKind::Ipv6 => FilterField::Ipv6,
            TokenKind::Icmp => FilterField::Icmp,
            TokenKind::Icmpv6 => FilterField::Icmpv6,
            TokenKind::Tcp => FilterField::Tcp,
            TokenKind::Udp => FilterField::Udp,
            TokenKind::IpHeaderLength => FilterField::IpHeaderLength,
            TokenKind::IpTos => FilterField::IpTos,
            TokenKind::IpLength => FilterField::IpLength,
            TokenKind::IpId => FilterField::IpId,
            TokenKind::IpDf => FilterField::IpDf,
            TokenKind::IpMf => FilterField::IpMf,
            TokenKind::IpFragOff => FilterField::IpFragOff,
            TokenKind::IpTtl => FilterField::IpTTL,
            TokenKind::IpProtocol => FilterField::IpProtocol,
            TokenKind::IpChecksum => FilterField::IpChecksum,
            TokenKind::IpSrcAddr => FilterField::IpSrcAddr,
            TokenKind::IpDstAddr => FilterField::IpDstAddr,
            TokenKind::Ipv6TrafficClass => FilterField::Ipv6TrafficClass,
            TokenKind::Ipv6FlowLabel => FilterField::Ipv6FlowLabel,
            TokenKind::Ipv6Length => FilterField::Ipv6Length,
            TokenKind::Ipv6NextHdr => FilterField::Ipv6NextHdr,
            TokenKind::Ipv6HopLimit => FilterField::Ipv6HopLimit,
            TokenKind::Ipv6SrcAddr => FilterField::Ipv6SrcAddr,
            TokenKind::Ipv6DstAddr => FilterField::Ipv6DstAddr,
            TokenKind::IcmpType => FilterField::IcmpType,
            TokenKind::IcmpCode => FilterField::IcmpCode,
            TokenKind::IcmpChecksum => FilterField::IcmpChecksum,
            TokenKind::IcmpBody => FilterField::IcmpBody,
            TokenKind::Icmpv6Type => FilterField::Icmpv6Type,
            TokenKind::Icmpv6Code => FilterField::Icmpv6Code,
            TokenKind::Icmpv6Checksum => FilterField::Icmpv6Checksum,
            TokenKind::Icmpv6Body => FilterField::Icmpv6Body,
            TokenKind::TcpSrcPort => FilterField::TcpSrcPort,
            TokenKind::TcpDstPort => FilterField::TcpDstPort,
            TokenKind::TcpSeqNum => FilterField::TcpSeqNum,
            TokenKind::TcpAckNum => FilterField::TcpAckNum,
            TokenKind::TcpHeaderLength => FilterField::TcpHeaderLength,
            TokenKind::TcpUrg => FilterField::TcpUrg,
            TokenKind::TcpAck => FilterField::TcpAck,
            TokenKind::TcpPsh => FilterField::TcpPsh,
            TokenKind::TcpRst => FilterField::TcpRst,
            TokenKind::TcpSyn => FilterField::TcpSyn,
            TokenKind::TcpFin => FilterField::TcpFin,
            TokenKind::TcpWindow => FilterField::TcpWindow,
            TokenKind::TcpChecksum => FilterField::TcpChecksum,
            TokenKind::TcpUrgPtr => FilterField::TcpUrgPtr,
            TokenKind::TcpPayloadLength => FilterField::TcpPayloadLength,
            TokenKind::UdpSrcPort => FilterField::UdpSrcPort,
            TokenKind::UdpDstPort => FilterField::UdpDstPort,
            TokenKind::UdpLength => FilterField::UdpLength,
            TokenKind::UdpChecksum => FilterField::UdpChecksum,
            TokenKind::UdpPayloadLength => FilterField::UdpPayloadLength,
            _ => FilterField::Zero,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum FilterTest {
    Eq = 0,
    Neq = 1,
    Lt = 2,
    Leq = 3,
    Gt = 4,
    Geq = 5,
}

impl From<FilterTest> for u8 {
    fn from(value: FilterTest) -> Self {
        value as u8
    }
}

impl From<FilterTest> for u32 {
    fn from(value: FilterTest) -> Self {
        value as u32
    }
}

impl From<u32> for FilterTest {
    fn from(value: u32) -> Self {
        unsafe { std::mem::transmute(value as u8) }
    }
}


pub struct VarInfo {
    pub var_type: TokenKind,
    lb: [u32; 4],
    ub: [u32; 4],
    neg_lb: bool,
    neg_ub: bool,
    eq: bool,
}

impl VarInfo {
    pub fn from_kind(kind: TokenKind) -> Option<Self> {
        use TokenKind::*;

        let mut lb = [0u32; 4];
        let mut ub = [0u32; 4];
        let mut neg_lb = false;
        let mut neg_ub = false;
        let mut var_type = TokenKind::True;
        let mut eq = false;

        match kind {
            Zero | False => {
                eq = true;
                lb = [0;4];
                ub = [0;4];
                var_type = True;
            }
            True => {
                eq = true;
                lb = [1,0,0,0];
                ub = [1,0,0,0];
                var_type = True;
            }
            Layer => ub = [4,0,0,0],
            Priority => {
                lb = [15,0,0,0];
                ub = [15,0,0,0];
                neg_lb = true;
            }
            Event => ub = [5,0,0,0],
            IpDf | IpMf => {
                var_type = Ip;
                ub = [1,0,0,0];
            }
            TcpUrg | TcpAck | TcpPsh | TcpRst | TcpSyn | TcpFin => {
                var_type = Tcp;
                ub = [1,0,0,0];
            }
            Inbound | Outbound | Fragment | Ip | Ipv6 | Icmp | Icmpv6 | Tcp | Udp => ub = [1,0,0,0],
            IpHeaderLength => {
                var_type = Ip;
                ub = [0x0F,0,0,0];
            }
            TcpHeaderLength => {
                var_type = Tcp;
                ub = [0x0F,0,0,0];
            }
            IpTtl | IpProtocol => {
                var_type = Ip;
                ub = [0xFF,0,0,0];
            }
            Ipv6TrafficClass | Ipv6NextHdr | Ipv6HopLimit => {
                var_type = Ipv6;
                ub = [0xFF,0,0,0];
            }
            IcmpType | IcmpCode => {
                var_type = Icmp;
                ub = [0xFF,0,0,0];
            }
            Icmpv6Type | Icmpv6Code => {
                var_type = Icmpv6;
                ub = [0xFF,0,0,0];
            }
            TcpPayload => {
                var_type = Tcp;
                ub = [0xFF,0,0,0];
            }
            UdpPayload => {
                var_type = Udp;
                ub = [0xFF,0,0,0];
            }
            Protocol | Packet | Random8 => ub = [0xFF,0,0,0],
            IpFragOff => {
                var_type = Ip;
                ub = [0x1FFF,0,0,0];
            }
            IpTos | IpLength | IpId | IpChecksum => {
                var_type = Ip;
                ub = [0xFFFF,0,0,0];
            }
            Ipv6Length => {
                var_type = Ipv6;
                ub = [0xFFFF,0,0,0];
            }
            IcmpChecksum => {
                var_type = Icmp;
                ub = [0xFFFF,0,0,0];
            }
            Icmpv6Checksum => {
                var_type = Icmpv6;
                ub = [0xFFFF,0,0,0];
            }
            TcpSrcPort | TcpDstPort | TcpWindow | TcpChecksum | TcpUrgPtr | TcpPayloadLength | TcpPayload16 => {
                var_type = Tcp;
                ub = [0xFFFF,0,0,0];
            }
            UdpSrcPort | UdpDstPort | UdpLength | UdpChecksum | UdpPayloadLength | UdpPayload16 => {
                var_type = Udp;
                ub = [0xFFFF,0,0,0];
            }
            LocalPort | RemotePort | Packet16 | Random16 => ub = [0xFFFF,0,0,0],
            Length => {
                ub = [65535,0,0,0];
                lb = [40,0,0,0];
            }
            Ipv6FlowLabel => {
                var_type = Ipv6;
                ub = [0x000FFFFF,0,0,0];
            }
            IpSrcAddr | IpDstAddr => {
                var_type = Ip;
                lb = [0,0xFFFF,0,0];
                ub = [0xFFFFFFFF,0xFFFF,0,0];
            }
            Ipv6SrcAddr | Ipv6DstAddr | LocalAddr | RemoteAddr => {
                ub = [0xFFFFFFFF,0xFFFFFFFF,0xFFFFFFFF,0xFFFFFFFF];
            }
            Timestamp => {
                lb = [0,0x80000000,0,0];
                ub = [0xFFFFFFFF,0x7FFFFFFF,0,0];
                neg_lb = true;
            }
            TcpPayload32 => {
                var_type = Tcp;
                ub = [0xFFFFFFFF,0,0,0];
            }
            UdpPayload32 => {
                var_type = Udp;
                ub = [0xFFFFFFFF,0,0,0];
            }
            IfIdx | SubIfIdx | Random32 | ProcessId => ub = [0xFFFFFFFF,0,0,0],
            EndpointId | ParentEndpointId => ub = [0xFFFFFFFF,0xFFFFFFFF,0,0],
            _ => return None,
        }

        Some(Self { var_type, lb, ub, neg_lb, neg_ub, eq })
    }

    pub fn compare(&self, kind: TokenKind, val: &Expression) -> Option<bool> {
        let val_arr = match &val.data {
            ExpressionData::Number { values: arr, neg } => (*arr, *neg),
            _ => return None,
        };

        let result_lb = Self::compare128(val_arr.1, val_arr.0, self.neg_lb, self.lb);
        let result_ub = Self::compare128(val_arr.1, val_arr.0, self.neg_ub, self.ub);


         match kind {
            TokenKind::Eq => {
                if result_lb < 0 || result_ub > 0 {
                    Some(false)
                } else if self.eq && result_lb == 0 {
                    Some(true)
                } else {
                    None
                }
            }
            TokenKind::Neq => {
                if result_lb < 0 || result_ub > 0 {
                    Some(true)
                } else if self.eq && result_lb == 0 {
                    Some(false)
                } else {
                    None
                }
            }
            TokenKind::Lt => {
                if result_ub > 0 {
                    Some(true)
                } else if result_lb <= 0 {
                    Some(false)
                } else {
                    None
                }
            }
            TokenKind::Leq => {
                if result_ub >= 0 {
                    Some(true)
                } else if result_lb < 0 {
                    Some(false)
                } else {
                    None
                }
            }
            TokenKind::Gt => {
                if result_ub >= 0 {
                    Some(false)
                } else if result_lb < 0 {
                    Some(true)
                } else {
                    None
                }
            }
            TokenKind::Geq => {
                if result_ub > 0 {
                    Some(false)
                } else if result_lb <= 0 {
                    Some(true)
                } else {
                    None
                }
            }
            _ => None,
        }
    }

    fn compare128(neg_a: bool, a: [u32; 4], neg_b: bool, b: [u32; 4]) -> i32 {
        let mut a_val = a;
        let mut b_val = b;

        if neg_a {
            for v in &mut a_val {
                *v = !*v;
            }
        }
        if neg_b {
            for v in &mut b_val {
                *v = !*v;
            }
        }

        for i in 0..4 {
            if a_val[i] != b_val[i] {
                return if a_val[i] < b_val[i] { -1 } else { 1 };
            }
        }
        0
    }
}