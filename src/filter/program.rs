use crate::constants::*;
use crate::filter::*;
use crate::*;

pub struct WinDivertFilterProgram {
    inner: Box<[WinDivertFilterRaw]>,
    layer: WinDivertLayer
}

impl WinDivertFilterProgram {
    pub fn into_inner(self) -> Box<[WinDivertFilterRaw]> {
        self.inner
    }

    pub fn compile(filter: &str, layer: WinDivertLayer) -> Result<Self, WinDivertError> {
        if filter.starts_with('@') {
            return Self::from_precompiled(filter);
        }
        
        let mut tokenizer = Tokenizer::new(filter);
        let tokens = tokenizer.tokenize()?;
        
        let mut parser = Parser::new(&tokens);
        let expr = parser.parse()?;

        let flattener = Flattener::new();
        let (label, stack) = flattener.flatten(expr);

        let emitter = Emitter::new();
        let inner = emitter.emit(label, stack).into();

        Ok(Self {
            inner,
            layer
        })
    }

    pub const fn size_of(&self) -> u32 {
        (self.inner.len() * std::mem::size_of::<WinDivertFilterRaw>()) as u32
    }

    fn from_precompiled(filter: &str) -> Result<Self, WinDivertError> {
        // let stream = WinDivertStream::new(filter);
        // Deserializer::deserialize(stream)?;
        unimplemented!()
    }

    pub fn analyse(&self) -> u64 {
        Analyser::analyse(&self.inner, self.layer)
    }

}

#[cfg(test)]
mod tests {

    use super::*;

    fn dump_as_hex(filters: &[WinDivertFilterRaw]) {
        for (idx, filter) in filters.iter().enumerate() {
            let ptr = filter as *const _ as *const u8;
            for i in 0..std::mem::size_of::<WinDivertFilterRaw>() {
                print!("{:02X} ", unsafe { *ptr.add(i) });
            }
            println!();
        }
    }

    #[test]
    fn should_match_c_filter_output_2() {
        let filter = WinDivertFilterProgram::compile("ip && tcp && loopback && (tcp.SrcPort == 53124 || tcp.DstPort == 53124)", WinDivertLayer::Network).unwrap();

        let filter_flags = filter.analyse();
        let filters: Box<[WinDivertFilterRaw]> = filter.into_inner();

        dump_as_hex(&filters);
        // EXPECTED
        // 05 08 01 00 FF 7F 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00
        // 08 08 02 00 FF 7F 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00
        // 3A 08 03 00 FF 7F 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00
        // 26 00 FE 7F 04 00 00 00 84 CF 00 00 00 00 00 00 00 00 00 00 00 00 00 00
        // 27 00 FE 7F FF 7F 00 00 84 CF 00 00 00 00 00 00 00 00 00 00 00 00 00 00
    }

    #[test]
    fn should_match_c_filter_output_1() {
        let filter = WinDivertFilterProgram::compile("ip && tcp", WinDivertLayer::Network).unwrap();

        let filter_flags = filter.analyse();
        let filters = filter.into_inner();

        dump_as_hex(&filters);
        // EXPECTED
        // 05 08 01 00 FF 7F 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 
        // 08 08 FE 7F FF 7F 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 
    }
}