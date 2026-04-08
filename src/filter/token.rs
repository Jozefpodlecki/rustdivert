use std::{collections::HashMap, fmt, sync::LazyLock};

use crate::{filter::TokenKind, *};

pub static TOKEN_MAP: LazyLock<HashMap<&'static str, TokenKind>> = LazyLock::new(|| {

    let pairs = [
        ("ACCEPT", TokenKind::EventAccept),
        ("BIND", TokenKind::EventBind),
        ("CLOSE", TokenKind::EventClose),
        ("CONNECT", TokenKind::EventConnect),
        ("DELETED", TokenKind::EventDeleted),
        ("ESTABLISHED", TokenKind::EventEstablished),
        ("FALSE", TokenKind::MacroFalse),
        ("FLOW", TokenKind::Flow),
        ("ICMP", TokenKind::MacroIcmp),
        ("ICMPV6", TokenKind::MacroIcmpv6),
        ("LISTEN", TokenKind::EventListen),
        ("NETWORK", TokenKind::Network),
        ("NETWORK_FORWARD", TokenKind::NetworkForward),
        ("OPEN", TokenKind::EventOpen),
        ("PACKET", TokenKind::EventPacket),
        ("REFLECT", TokenKind::Reflect),
        ("SOCKET", TokenKind::Socket),
        ("TCP", TokenKind::MacroTcp),
        ("TRUE", TokenKind::MacroTrue),
        ("UDP", TokenKind::MacroUdp),
        ("and", TokenKind::And),
        ("endpointId", TokenKind::EndpointId),
        ("event", TokenKind::Event),
        ("false", TokenKind::False),
        ("fragment", TokenKind::Fragment),
        ("icmp", TokenKind::Icmp),
        ("icmp.Body", TokenKind::IcmpBody),
        ("icmp.Checksum", TokenKind::IcmpChecksum),
        ("icmp.Code", TokenKind::IcmpCode),
        ("icmp.Type", TokenKind::IcmpType),
        ("icmpv6", TokenKind::Icmpv6),
        ("icmpv6.Body", TokenKind::Icmpv6Body),
        ("icmpv6.Checksum", TokenKind::Icmpv6Checksum),
        ("icmpv6.Code", TokenKind::Icmpv6Code),
        ("icmpv6.Type", TokenKind::Icmpv6Type),
        ("ifIdx", TokenKind::IfIdx),
        ("impostor", TokenKind::Impostor),
        ("inbound", TokenKind::Inbound),
        ("ip", TokenKind::Ip),
        ("ip.Checksum", TokenKind::IpChecksum),
        ("ip.DF", TokenKind::IpDf),
        ("ip.DstAddr", TokenKind::IpDstAddr),
        ("ip.FragOff", TokenKind::IpFragOff),
        ("ip.HdrLength", TokenKind::IpHeaderLength),
        ("ip.Id", TokenKind::IpId),
        ("ip.Length", TokenKind::IpLength),
        ("ip.MF", TokenKind::IpMf),
        ("ip.Protocol", TokenKind::IpProtocol),
        ("ip.SrcAddr", TokenKind::IpSrcAddr),
        ("ip.TOS", TokenKind::IpTos),
        ("ip.TTL", TokenKind::IpTtl),
        ("ipv6", TokenKind::Ipv6),
        ("ipv6.DstAddr", TokenKind::Ipv6DstAddr),
        ("ipv6.FlowLabel", TokenKind::Ipv6FlowLabel),
        ("ipv6.HopLimit", TokenKind::Ipv6HopLimit),
        ("ipv6.Length", TokenKind::Ipv6Length),
        ("ipv6.NextHdr", TokenKind::Ipv6NextHdr),
        ("ipv6.SrcAddr", TokenKind::Ipv6SrcAddr),
        ("ipv6.TrafficClass", TokenKind::Ipv6TrafficClass),
        ("layer", TokenKind::Layer),
        ("length", TokenKind::Length),
        ("localAddr", TokenKind::LocalAddr),
        ("localPort", TokenKind::LocalPort),
        ("loopback", TokenKind::Loopback),
        ("not", TokenKind::Not),
        ("or", TokenKind::Or),
        ("outbound", TokenKind::Outbound),
        ("packet", TokenKind::Packet),
        ("packet16", TokenKind::Packet16),
        ("packet32", TokenKind::Packet32),
        ("parentEndpointId", TokenKind::ParentEndpointId),
        ("priority", TokenKind::Priority),
        ("processId", TokenKind::ProcessId),
        ("protocol", TokenKind::Protocol),
        ("random16", TokenKind::Random16),
        ("random32", TokenKind::Random32),
        ("random8", TokenKind::Random8),
        ("remoteAddr", TokenKind::RemoteAddr),
        ("remotePort", TokenKind::RemotePort),
        ("subIfIdx", TokenKind::SubIfIdx),
        ("tcp", TokenKind::Tcp),
        ("tcp.Ack", TokenKind::TcpAck),
        ("tcp.AckNum", TokenKind::TcpAckNum),
        ("tcp.Checksum", TokenKind::TcpChecksum),
        ("tcp.DstPort", TokenKind::TcpDstPort),
        ("tcp.Fin", TokenKind::TcpFin),
        ("tcp.HdrLength", TokenKind::TcpHeaderLength),
        ("tcp.Payload", TokenKind::TcpPayload),
        ("tcp.Payload16", TokenKind::TcpPayload16),
        ("tcp.Payload32", TokenKind::TcpPayload32),
        ("tcp.PayloadLength", TokenKind::TcpPayloadLength),
        ("tcp.Psh", TokenKind::TcpPsh),
        ("tcp.Rst", TokenKind::TcpRst),
        ("tcp.SeqNum", TokenKind::TcpSeqNum),
        ("tcp.SrcPort", TokenKind::TcpSrcPort),
        ("tcp.Syn", TokenKind::TcpSyn),
        ("tcp.Urg", TokenKind::TcpUrg),
        ("tcp.UrgPtr", TokenKind::TcpUrgPtr),
        ("tcp.Window", TokenKind::TcpWindow),
        ("timestamp", TokenKind::Timestamp),
        ("true", TokenKind::True),
        ("udp", TokenKind::Udp),
        ("udp.Checksum", TokenKind::UdpChecksum),
        ("udp.DstPort", TokenKind::UdpDstPort),
        ("udp.Length", TokenKind::UdpLength),
        ("udp.Payload", TokenKind::UdpPayload),
        ("udp.Payload16", TokenKind::UdpPayload16),
        ("udp.Payload32", TokenKind::UdpPayload32),
        ("udp.PayloadLength", TokenKind::UdpPayloadLength),
        ("udp.SrcPort", TokenKind::UdpSrcPort),
        ("zero", TokenKind::Zero),
    ];
    pairs.into_iter().collect()
});

#[derive(Debug, Clone)]
pub struct Token {
    pub kind: TokenKind,
    pub position: usize,
    pub val: [u32; 4],
}

impl Token {
    pub fn new(kind: TokenKind, position: usize) -> Self {
        Self {
            kind,
            position,
            val: [0; 4]
        }
    }
}

impl fmt::Display for Token {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self.kind)
    }
}

pub struct Tokenizer<'a> {
    input: &'a [u8],
    position: usize
}


impl<'a> Tokenizer<'a> {
    pub fn new(filter: &'a str) -> Self {
        Self {
            input: filter.as_bytes(),
            position: 0,
        }
    }

    pub fn tokenize(&mut self) -> Result<Vec<Token>, WinDivertError> {
        let mut tokens = Vec::new();

        while tokens.len() < 1024 {
            self.skip_whitespace();
            
            if self.position >= self.input.len() {
                tokens.push(Token::new(TokenKind::End, self.position));
                return Ok(tokens);
            }

            let start = self.position;
            let c = self.input[self.position];
            self.position += 1;

            match c {
                b'(' => tokens.push(Token::new(TokenKind::Open, start)),
                b')' => tokens.push(Token::new(TokenKind::Close, start)),
                b'[' => tokens.push(Token::new(TokenKind::SquareOpen, start)),
                b']' => tokens.push(Token::new(TokenKind::SquareClose, start)),
                b'-' => tokens.push(Token::new(TokenKind::Minus, start)),
                b'?' => tokens.push(Token::new(TokenKind::Question, start)),
                b':' => {
                    if self.peek() != Some(b':') {
                        tokens.push(Token::new(TokenKind::Colon, start));
                    } else {
                        self.position = start;
                        self.parse_identifier(&mut tokens, &TOKEN_MAP)?;
                    }
                }
                b'!' => {
                    if self.peek() == Some(b'=') {
                        self.position += 1;
                        tokens.push(Token::new(TokenKind::Neq, start));
                    } else {
                        tokens.push(Token::new(TokenKind::Not, start));
                    }
                }
                b'=' => {
                    if self.peek() == Some(b'=') {
                        self.position += 1;
                    }
                    tokens.push(Token::new(TokenKind::Eq, start));
                }
                b'<' => {
                    if self.peek() == Some(b'=') {
                        self.position += 1;
                        tokens.push(Token::new(TokenKind::Leq, start));
                    } else {
                        tokens.push(Token::new(TokenKind::Lt, start));
                    }
                }
                b'>' => {
                    if self.peek() == Some(b'=') {
                        self.position += 1;
                        tokens.push(Token::new(TokenKind::Geq, start));
                    } else {
                        tokens.push(Token::new(TokenKind::Gt, start));
                    }
                }
                b'&' => {
                    if self.peek() != Some(b'&') {
                        return Err(WinDivertError::BadToken(self.position - 1));
                    }
                    self.position += 1;
                    tokens.push(Token::new(TokenKind::And, start));
                }
                b'|' => {
                    if self.peek() != Some(b'|') {
                        return Err(WinDivertError::BadToken(self.position - 1));
                    }
                    self.position += 1;
                    tokens.push(Token::new(TokenKind::Or, start));
                }
                _ if self.is_alnum(c) || c == b'.' || c == b'_' => {
                    self.position = start;
                    self.parse_identifier(&mut tokens, &TOKEN_MAP)?;
                }
                _ => return Err(WinDivertError::BadToken(start)),
            }
        }
        
        Err(WinDivertError::TooLong)
    }

    fn parse_identifier(&mut self, tokens: &mut Vec<Token>, token_info: &std::collections::HashMap<&'static str, TokenKind>) -> Result<(), WinDivertError> {
        let start = self.position;
        let mut ident = Vec::new();
        
        while self.position < self.input.len() && (self.is_alnum(self.input[self.position]) || 
              self.input[self.position] == b'.' || self.input[self.position] == b'_') {
            ident.push(self.input[self.position]);
            self.position += 1;
        }
        
        if ident.is_empty() {
            return Err(WinDivertError::BadToken(start));
        }
        
        let ident_str = String::from_utf8_lossy(&ident);
        
        if let Some(&kind) = token_info.get(ident_str.as_ref()) {
            tokens.push(Token {
                kind,
                position: start,
                val: [0; 4],
            });
            return Ok(());
        }
        
        if ident_str == "b" {
            tokens.push(Token::new(TokenKind::Bytes, start));
            return Ok(());
        }
        
        let num_str = std::str::from_utf8(&ident).unwrap();
        
        if let Some(num) = self.parse_number(num_str) {
            let mut val = [0; 4];
            val[0] = num;
            tokens.push(Token {
                kind: TokenKind::Number,
                position: start,
                val,
            });
            return Ok(());
        }
        
        if let Some(addr) = self.parse_ipv4(num_str) {
            let mut val = [0; 4];
            val[0] = addr;
            val[1] = 0x0000FFFF;
            tokens.push(Token {
                kind: TokenKind::Number,
                position: start,
                val,
            });
            return Ok(());
        }
        
        if let Some(addr) = self.parse_ipv6(num_str) {
            let mut val = [0; 4];
            val[0] = addr[0];
            val[1] = addr[1];
            val[2] = addr[2];
            val[3] = addr[3];
            tokens.push(Token {
                kind: TokenKind::Number,
                position: start,
                val,
            });
            return Ok(());
        }
        
        Err(WinDivertError::BadToken(start))
    }

    fn skip_whitespace(&mut self) {
        while self.position < self.input.len() && self.input[self.position].is_ascii_whitespace() {
            self.position += 1;
        }
    }

    fn peek(&self) -> Option<u8> {
        if self.position < self.input.len() {
            Some(self.input[self.position])
        } else {
            None
        }
    }

    fn is_alnum(&self, c: u8) -> bool {
        c.is_ascii_alphanumeric()
    }

    fn parse_number(&self, s: &str) -> Option<u32> {
        s.parse().ok()
    }

    fn parse_ipv4(&self, s: &str) -> Option<u32> {
        let parts: Vec<&str> = s.split('.').collect();
        if parts.len() != 4 {
            return None;
        }
        let mut ip = 0u32;
        for part in parts {
            let octet: u32 = part.parse().ok()?;
            if octet > 255 {
                return None;
            }
            ip = (ip << 8) | octet;
        }
        Some(ip)
    }

    fn parse_ipv6(&self, s: &str) -> Option<[u32; 4]> {
        if s.parse::<std::net::Ipv6Addr>().is_ok() {
            Some([0, 0, 0, 0])
        } else {
            None
        }
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn should_tokenize_simple_filter() {
        let filter = "!loopback && ip && tcp && tcp.DstPort == 1234";
        let mut tokenizer = Tokenizer::new(filter);
        let tokens = tokenizer.tokenize().unwrap();
        
        assert_eq!(tokens[0].kind, TokenKind::Not);
        assert_eq!(tokens[1].kind, TokenKind::Loopback);
        assert_eq!(tokens[2].kind, TokenKind::And);
        assert_eq!(tokens[3].kind, TokenKind::Ip);
        assert_eq!(tokens[4].kind, TokenKind::And);
        assert_eq!(tokens[5].kind, TokenKind::Tcp);
        assert_eq!(tokens[6].kind, TokenKind::And);
        assert_eq!(tokens[7].kind, TokenKind::TcpDstPort);
        assert_eq!(tokens[8].kind, TokenKind::Eq);
        assert_eq!(tokens[9].kind, TokenKind::Number);
        assert_eq!(tokens[9].val[0], 1234);
        assert_eq!(tokens[10].kind, TokenKind::End);
    }

    #[test]
    fn should_tokenize_with_parentheses() {
        let filter = "(tcp.SrcPort == 80 || tcp.DstPort == 80) && ip";
        let mut tokenizer = Tokenizer::new(filter);
        let tokens = tokenizer.tokenize().unwrap();
        
        assert_eq!(tokens[0].kind, TokenKind::Open);
        assert_eq!(tokens[1].kind, TokenKind::TcpSrcPort);
        assert_eq!(tokens[2].kind, TokenKind::Eq);
        assert_eq!(tokens[3].kind, TokenKind::Number);
        assert_eq!(tokens[4].kind, TokenKind::Or);
        assert_eq!(tokens[5].kind, TokenKind::TcpDstPort);
        assert_eq!(tokens[6].kind, TokenKind::Eq);
        assert_eq!(tokens[7].kind, TokenKind::Number);
        assert_eq!(tokens[8].kind, TokenKind::Close);
        assert_eq!(tokens[9].kind, TokenKind::And);
        assert_eq!(tokens[10].kind, TokenKind::Ip);
    }
}