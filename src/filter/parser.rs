use std::fmt::{self, Debug};

use crate::filter::{Expression, ExpressionData, Token, TokenKind};
use crate::*;

pub struct Parser<'a> {
    tokens: &'a [Token],
    position: usize,
}

impl<'a> Parser<'a> {
    pub fn new(tokens: &'a [Token]) -> Self {
        Self { tokens, position: 0 }
    }

    pub fn parse(&mut self) -> Result<Box<Expression>, WinDivertError> {
        let depth = 1024;
        self.parse_filter(depth, false)
    }

    fn parse_filter(&mut self, depth: i32, and: bool) -> Result<Box<Expression>, WinDivertError> {
        let depth = depth - 1;
        if depth < 0 {
            return Err(WinDivertError::TooDeep(self.current_position()));
        }

        let mut expr = if and {
            self.parse_and_or_arg(depth)?
        } else {
            self.parse_filter(depth, true)?
        };

        loop {
            match self.current_kind() {
                TokenKind::And => {
                    self.position += 1;
                    let arg = self.parse_and_or_arg(depth)?;
                    expr = Expression::new_bin_op(TokenKind::And, expr, arg);
                }
                TokenKind::Or => {
                    self.position += 1;
                    let arg = self.parse_filter(depth, true)?;
                    expr = Expression::new_bin_op(TokenKind::Or, expr, arg);
                }
                _ => return Ok(expr),
            }
        }
    }

    fn parse_and_or_arg(&mut self, depth: i32) -> Result<Box<Expression>, WinDivertError> {
        let depth = depth - 1;
        if depth < 0 {
            return Err(WinDivertError::TooDeep(self.current_position()));
        }

        match self.current_kind() {
            TokenKind::Open => {
                self.position += 1;
                let arg = self.parse_filter(depth, false)?;

                match self.current_kind() {
                    TokenKind::Close => {
                        self.position += 1;
                        Ok(arg)
                    }
                    TokenKind::Question => {
                        self.position += 1;
                        let then_expr = self.parse_filter(depth, false)?;
                        
                        if self.current_kind() != TokenKind::Colon {
                            return Err(WinDivertError::UnexpectedToken(self.current_position()));
                        }
                        self.position += 1;
                        
                        let else_expr = self.parse_filter(depth, false)?;
                        
                        if self.current_kind() != TokenKind::Close {
                            return Err(WinDivertError::UnexpectedToken(self.current_position()));
                        }
                        self.position += 1;
                        
                        let mut expr = Expression::new_ternary(arg, then_expr, else_expr);
                        Ok(expr)
                    }
                    _ => Err(WinDivertError::UnexpectedToken(self.current_position())),
                }
            }
            _ => self.parse_test(),
        }
    }

    fn parse_test(&mut self) -> Result<Box<Expression>, WinDivertError> {
        let mut not = false;
        while self.current_kind() == TokenKind::Not {
            not = !not;
            self.position += 1;
        }

        let var = match self.current_kind() {
            TokenKind::Timestamp | TokenKind::Priority | TokenKind::Zero
            | TokenKind::Event | TokenKind::Random8 | TokenKind::Random16
            | TokenKind::Random32 | TokenKind::True | TokenKind::False
            | TokenKind::Outbound | TokenKind::Inbound | TokenKind::Fragment
            | TokenKind::IfIdx | TokenKind::SubIfIdx | TokenKind::Loopback
            | TokenKind::Impostor | TokenKind::Ip | TokenKind::Ipv6
            | TokenKind::Icmp | TokenKind::Icmpv6 | TokenKind::Tcp | TokenKind::Udp
            | TokenKind::ProcessId | TokenKind::LocalAddr | TokenKind::RemoteAddr
            | TokenKind::LocalPort | TokenKind::RemotePort | TokenKind::Protocol
            | TokenKind::EndpointId | TokenKind::ParentEndpointId | TokenKind::Length
            | TokenKind::Layer | TokenKind::IpHeaderLength | TokenKind::IpTos
            | TokenKind::IpLength | TokenKind::IpId | TokenKind::IpDf
            | TokenKind::IpMf | TokenKind::IpFragOff | TokenKind::IpTtl
            | TokenKind::IpProtocol | TokenKind::IpChecksum | TokenKind::IpSrcAddr
            | TokenKind::IpDstAddr | TokenKind::Ipv6TrafficClass | TokenKind::Ipv6FlowLabel
            | TokenKind::Ipv6Length | TokenKind::Ipv6NextHdr | TokenKind::Ipv6HopLimit
            | TokenKind::Ipv6SrcAddr | TokenKind::Ipv6DstAddr | TokenKind::IcmpType
            | TokenKind::IcmpCode | TokenKind::IcmpChecksum | TokenKind::IcmpBody
            | TokenKind::Icmpv6Type | TokenKind::Icmpv6Code | TokenKind::Icmpv6Checksum
            | TokenKind::Icmpv6Body | TokenKind::TcpSrcPort | TokenKind::TcpDstPort
            | TokenKind::TcpSeqNum | TokenKind::TcpAckNum | TokenKind::TcpHeaderLength
            | TokenKind::TcpUrg | TokenKind::TcpAck | TokenKind::TcpPsh | TokenKind::TcpRst
            | TokenKind::TcpSyn | TokenKind::TcpFin | TokenKind::TcpWindow
            | TokenKind::TcpChecksum | TokenKind::TcpUrgPtr | TokenKind::TcpPayloadLength
            | TokenKind::UdpSrcPort | TokenKind::UdpDstPort | TokenKind::UdpLength
            | TokenKind::UdpChecksum | TokenKind::UdpPayloadLength => {
                let kind = self.current_kind();
                self.position += 1;
                Expression::new_var(kind)
            }
            _ => return Err(WinDivertError::UnexpectedToken(self.current_position())),
        };

        let kind = match self.current_kind() {
            TokenKind::Eq | TokenKind::Neq | TokenKind::Lt | TokenKind::Leq 
            | TokenKind::Gt | TokenKind::Geq => {
                let k = self.current_kind();
                self.position += 1;
                k
            }
            _ => {
                let kind = if not { TokenKind::Eq } else { TokenKind::Neq };
                let right = Expression::new_number([0, 0, 0, 0], false);
                let expr = Expression::new_bin_op(
                    kind,
                    var,
                    right,
                );
                return Ok(expr);
            }
        };

        let kind = if not {
            match kind {
                TokenKind::Eq => TokenKind::Neq,
                TokenKind::Neq => TokenKind::Eq,
                TokenKind::Lt => TokenKind::Geq,
                TokenKind::Leq => TokenKind::Gt,
                TokenKind::Gt => TokenKind::Leq,
                TokenKind::Geq => TokenKind::Lt,
                _ => kind,
            }
        } else {
            kind
        };

        let mut neg = false;
        if self.current_kind() == TokenKind::Minus {
            neg = true;
            self.position += 1;
        }

        if self.current_kind() != TokenKind::Number {
            return Err(WinDivertError::UnexpectedToken(self.current_position()));
        }

        let val = self.current_token().val;
        self.position += 1;

        let expr = Expression::new_bin_op(kind, var, Expression::new_number(val, neg));
        Ok(expr)
    }

    fn current_kind(&self) -> TokenKind {
        self.tokens.get(self.position).map(|t| t.kind).unwrap_or(TokenKind::End)
    }

    fn current_token(&self) -> &Token {
        &self.tokens[self.position]
    }

    fn current_position(&self) -> usize {
        self.tokens.get(self.position).map(|t| t.position).unwrap_or(0)
    }
}

#[cfg(test)]
mod tests {
    use crate::filter::Tokenizer;

    use super::*;

    #[test]
    fn should_parse_complex() {
        let filter = "ip && tcp && loopback && (tcp.SrcPort == 53124 || tcp.DstPort == 53124)";
        let mut tokenizer = Tokenizer::new(filter);
        let tokens = tokenizer.tokenize().unwrap();
        let mut parser = Parser::new(&tokens);
        let expr = parser.parse().unwrap();
        
        assert_eq!(expr.kind, TokenKind::And);
    }

    #[test]
    fn should_parse_simple_and() {
        let filter = "ip && tcp";
        let mut tokenizer = Tokenizer::new(filter);
        let tokens = tokenizer.tokenize().unwrap();
        let mut parser = Parser::new(&tokens);
        let expr = parser.parse().unwrap();
        
        assert_eq!(expr.kind, TokenKind::And);
    }

    #[test]
    fn should_parse_simple_test() {
        let filter = "tcp.DstPort == 80";
        let mut tokenizer = Tokenizer::new(filter);
        let tokens = tokenizer.tokenize().unwrap();
        let mut parser = Parser::new(&tokens);
        let expr = parser.parse().unwrap();
        
        assert_eq!(expr.kind, TokenKind::Eq);
    }
}