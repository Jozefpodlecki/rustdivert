use crate::{constants::*, filter::{FilterField, WinDivertFilterRaw}};

#[derive(Debug)]
pub enum SerdeError {
    UnexpectedEof,
    Overflow,
    InvalidDigit(char),
    InvalidHeader,
    InvalidVersion(u32),
    InvalidLength(u32),
    InvalidLabel,
    InvalidJump,
}

pub struct FilterSerializer {
    buf: Vec<u8>,
}

impl FilterSerializer {
    fn new() -> Self {
        Self { buf: Vec::with_capacity(128) }
    }

    fn serialize_all(
        mut self,
        filters: &[WinDivertFilterRaw],
    ) -> Result<Vec<u8>, SerdeError> {
        self.put_str("@WinDiv_");
        self.write_number(0);
        self.write_number(filters.len() as u32);

        for f in filters {
            self.serialize_test(f)?;
        }

        self.put_nul();
        Ok(self.buf)
    }

     fn serialize_label(&mut self, label: u16) {
        match label as i16 {
            FILTER_RESULT_ACCEPT => self.put_char(b'A'),
            FILTER_RESULT_REJECT => self.put_char(b'X'),
            _ => {
                self.put_char(b'L');
                self.write_number(label as u32);
            }
        }
    }

    fn serialize_test(&mut self, filter: &WinDivertFilterRaw) -> Result<(), SerdeError> {
        self.put_char(b'_');

        self.write_number(filter.field().into());
        self.write_number(filter.test() as u32);
        self.write_number(filter.neg());
        self.write_number(filter.nth_arg(0));

        match filter.field() {
            FilterField::Ipv6SrcAddr
            | FilterField::Ipv6DstAddr
            | FilterField::LocalAddr
            | FilterField::RemoteAddr => {
                self.write_numbers(&filter.args());
            }

            FilterField::EndpointId
            | FilterField::ParentEndpointId
            | FilterField::Timestamp => {
                self.write_number(filter.nth_arg(1));
            }

            FilterField::Packet
            | FilterField::Packet16
            | FilterField::Packet32
            | FilterField::TcpPayload
            | FilterField::TcpPayload16
            | FilterField::TcpPayload32
            | FilterField::UdpPayload
            | FilterField::UdpPayload16
            | FilterField::UdpPayload32 => {
                let idx = (filter.nth_arg(1) as i32 + u16::MAX as i32) as u32;
                self.write_number(idx);
            }

            _ => {}
        }

        self.serialize_label(filter.success());
        self.serialize_label(filter.failure());

        Ok(())
    }

    #[inline]
    fn put_char(&mut self, c: u8) {
        self.buf.push(c);
    }

    fn put_str(&mut self, s: &str) {
        self.buf.extend_from_slice(s.as_bytes());
    }

    fn put_nul(&mut self) {
        self.buf.push(0);
    }

     fn encode_digit(dig: u8, final_: bool) -> u8 {
        const TABLE: &[u8; 64] =
            b"0123456789ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz+=";

        TABLE[(dig & 0x1F) as usize + if final_ { 32 } else { 0 }]
    }

    fn write_numbers(&mut self, values: &[u32]) {
        for value in values {
            self.write_number(*value);
        }   
    }

    fn write_number(&mut self, val: u32) {
        const FIRST_DIGIT_MASK: u32 = 0x3E00_0000;
        let mut mask: u32 = 0xC000_0000;
        let mut dig: i32 = 6;

        while (mask & val) == 0 && dig != 0 {
            mask = if dig == 6 { FIRST_DIGIT_MASK } else { mask >> 5 };
            dig -= 1;
        }

        loop {
            let final_ = dig == 0;
            let digit = ((mask & val) >> (5 * dig)) as u8;

            self.put_char(Self::encode_digit(digit, final_));

            if final_ {
                break;
            }

            mask = if dig == 6 { FIRST_DIGIT_MASK } else { mask >> 5 };
            dig -= 1;
        }
    }
}

pub struct FilterDeserializer<'a> {
    buf: &'a [u8],
    pos: usize,
}

impl<'a> FilterDeserializer<'a> {
    fn new(buf: &'a [u8]) -> Self {
        Self { buf, pos: 0 }
    }

    fn read_test(&mut self) -> Result<WinDivertFilterRaw, SerdeError> {
        if self.get_char()? != b'_' {
            return Err(SerdeError::InvalidHeader);
        }

        let field = FilterField::from(self.read_number(1)?);
        let test = self.read_number(1)? as u32;
        let neg = self.read_number(1)?;
        let arg0 = self.read_number(1)?;

        let mut args = [arg0, 0, 0, 0];

        match field {
            FilterField::Ipv6SrcAddr
            | FilterField::Ipv6DstAddr
            | FilterField::LocalAddr
            | FilterField::RemoteAddr => {
                for i in 1..4 {
                    args[i] = self.read_number(1)?;
                }
            }
            FilterField::EndpointId
            | FilterField::ParentEndpointId
            | FilterField::Timestamp => {
                args[1] = self.read_number(1)?;
            }
            FilterField::Packet
            | FilterField::Packet16
            | FilterField::Packet32
            | FilterField::TcpPayload
            | FilterField::TcpPayload16
            | FilterField::TcpPayload32
            | FilterField::UdpPayload
            | FilterField::UdpPayload16
            | FilterField::UdpPayload32 => {
                let idx = self.read_number(1)?;
                args[1] = (idx as i32 - u16::MAX as i32) as u32;
            }
            _ => {}
        }

        let success = self.read_label()?;
        let failure = self.read_label()?;

        let mut filter = WinDivertFilterRaw::default();
        filter.set_field(field);
        filter.set_test(test.into());
        filter.set_neg(neg);
        filter.set_args(&args);
        filter.set_success(success);
        filter.set_failure(failure);

        Ok(filter)
    }

    fn read_header(&mut self) -> Result<usize, SerdeError> {
        const MAGIC: &[u8] = b"@WinDiv_";

        for &b in MAGIC {
            if self.get_char()? != b {
                return Err(SerdeError::InvalidHeader);
            }
        }

        let version = self.read_number(4)?;
        if version != 0 {
            return Err(SerdeError::InvalidVersion(version));
        }

        let length = self.read_number(2)?;
        if length == 0 || length > FILTER_MAXLEN as u32 {
            return Err(SerdeError::InvalidLength(length));
        }

        Ok(length as usize)
    }

    fn read_label(&mut self) -> Result<u16, SerdeError> {
        match self.get_char()? {
            b'A' => Ok(FILTER_RESULT_ACCEPT as u16),
            b'X' => Ok(FILTER_RESULT_REJECT as u16),
            b'L' => {
                let val = self.read_number(2)?;
                if val > FILTER_MAXLEN as u32 {
                    return Err(SerdeError::InvalidLabel);
                }
                Ok(val as u16)
            }
            c => Err(SerdeError::InvalidDigit(c as char)),
        }
    }

    fn deserialize_all(&mut self) -> Result<Box<[WinDivertFilterRaw]>, SerdeError> {
        let len = self.read_header()?;

        let mut filters = Vec::with_capacity(len);

        for i in 0..len {
            let filter = self.read_test()?;

            match filter.success() as i16 {
                FILTER_RESULT_ACCEPT
                | FILTER_RESULT_REJECT => {}
                s => {
                    if s as usize <= i || s as usize >= len {
                        return Err(SerdeError::InvalidJump);
                    }
                }
            }

            match filter.failure() as i16 {
                FILTER_RESULT_ACCEPT
                | FILTER_RESULT_REJECT => {}
                s => {
                    if s as usize <= i || s as usize >= len {
                        return Err(SerdeError::InvalidJump);
                    }
                }
            }

            filters.push(filter);
        }

        if self.get_char()? != 0 {
            return Err(SerdeError::InvalidHeader);
        }

        Ok(filters.into())
    }

    fn get_char(&mut self) -> Result<u8, SerdeError> {
        if self.pos >= self.buf.len() {
            return Err(SerdeError::UnexpectedEof);
        }
        let c = self.buf[self.pos];
        self.pos += 1;
        Ok(c)
    }

    fn decode_digit(c: u8) -> Result<(u8, bool), SerdeError> {
        match c {
            b'0'..=b'9' => Ok((c - b'0', false)),
            b'A'..=b'V' => Ok((c - b'A' + 10, false)),
            b'W'..=b'Z' => Ok((c - b'W', true)),
            b'a'..=b'z' => Ok((c - b'a' + 4, true)),
            b'+' => Ok((30, true)),
            b'=' => Ok((31, true)),
            _ => Err(SerdeError::InvalidDigit(c as char)),
        }
    }

    fn read_number(&mut self, max_len: usize) -> Result<u32, SerdeError> {
        let mut val: u32 = 0;

        for _ in 0..max_len {
            if (val & 0xF800_0000) != 0 {
                return Err(SerdeError::Overflow);
            }

            val <<= 5;

            let c = self.get_char()?;
            let (digit, final_) = Self::decode_digit(c)?;

            val += digit as u32;

            if final_ {
                return Ok(val);
            }
        }

        Err(SerdeError::Overflow)
    }  
}

#[cfg(test)]
mod tests {
    use crate::{WinDivertLayer, filter::*};
    use super::*;

    #[test]
    fn should_serialize_and_deserialize() {
        let filter = "ip && tcp";
        let mut tokenizer = Tokenizer::new(filter);
        let tokens = tokenizer.tokenize().unwrap();
        let mut parser = Parser::new(&tokens);
        let expr = parser.parse().unwrap();
        let flattener = Flattener::new();
        let (label, cfg) = flattener.flatten(expr);
        
        let emitter = Emitter::new();
        let filters = emitter.emit(label, cfg);
        
        let serializer = FilterSerializer::new();
        let stream = serializer.serialize_all(&filters).unwrap();

        let mut deserializer = FilterDeserializer::new(&stream);
        let filters = deserializer.deserialize_all().unwrap();
    }
}