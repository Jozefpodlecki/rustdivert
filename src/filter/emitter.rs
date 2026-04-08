use crate::filter::{Expression, ExpressionData, TokenKind, WinDivertFilterRaw, FilterField, FilterTest};
use crate::constants::*;

pub struct Emitter {
    kind_to_field: fn(TokenKind) -> FilterField,
}

impl Emitter {
    pub fn new() -> Self {
        Self {
            kind_to_field: TokenKind::into,
        }
    }

    pub fn emit(&self, label: i16, cfg: Vec<Expression>) -> Vec<WinDivertFilterRaw> {
        if label == FILTER_RESULT_ACCEPT || label == FILTER_RESULT_REJECT {
            let mut object = WinDivertFilterRaw::default();
            object.set_field(FilterField::Zero);
            object.set_test(FilterTest::Eq);
            object.set_neg(0);
            object.arg = [0, 0, 0, 0];
            object.set_success(label as u16);
            object.set_failure(label as u16);
            return vec![object];
        }

        let offset = label as u16; 
        let mut result = Vec::with_capacity(cfg.len());
        
        for expr in cfg.into_iter().rev() {
            result.push(self.emit_test(&expr, offset));
        }
        
        result
    }

    fn emit_test(&self, test: &Expression, offset: u16) -> WinDivertFilterRaw {
        let mut object = WinDivertFilterRaw::default();
        
        let (var, val) = match &test.data {
            ExpressionData::Binary { left, right } => (left.as_ref(), right.as_ref()),
            _ => panic!("Expected binary expression"),
        };
        
        if let Some(filter_test) = test.kind.to_filter_test() {
            object.set_test(filter_test);
        } else {
            return object;
        }
        
        let field_kind = match &var.data {
            ExpressionData::Var(kind) => *kind,
            _ => panic!("Expected Var"),
        };
        object.set_field((self.kind_to_field)(field_kind));
        
        match &val.data {
            ExpressionData::Number { val, neg } => {
                object.set_neg(if *neg { 1 } else { 0 });
                object.arg = *val;
            }
            _ => panic!("Expected Number"),
        }
        
        match var.kind {
            TokenKind::Packet | TokenKind::Packet16 | TokenKind::Packet32 |
            TokenKind::TcpPayload | TokenKind::TcpPayload16 | TokenKind::TcpPayload32 |
            TokenKind::UdpPayload | TokenKind::UdpPayload16 | TokenKind::UdpPayload32 => {
                if let ExpressionData::Number { val, .. } = &var.data {
                    object.arg[1] = val[0];
                }
            }
            _ => {}
        }
        
        match test.succ as i16 {
            FILTER_RESULT_ACCEPT | FILTER_RESULT_REJECT => {
                object.set_success(test.succ);
            }
            _ => {
                object.set_success(offset - test.succ);
            }
        }
        
        match test.fail as i16 {
            FILTER_RESULT_ACCEPT | FILTER_RESULT_REJECT => {
                object.set_failure(test.fail);
            }
            _ => {
                object.set_failure(offset - test.fail);
            }
        }
        
        object
    }
}

#[cfg(test)]
mod tests {
    use crate::{WinDivertLayer, filter::{Parser, Tokenizer, flattener::Flattener}};
    use super::*;

    #[test]
    fn should_emit_simple_and() {
        let filter = "ip && tcp";
        let mut tokenizer = Tokenizer::new(filter);
        let tokens = tokenizer.tokenize().unwrap();
        let mut parser = Parser::new(&tokens);
        let expr = parser.parse().unwrap();
        let flattener = Flattener::new();
        let (label, cfg) = flattener.flatten(expr);
        
        let emitter = Emitter::new();
        let result = emitter.emit(label, cfg);
        
        assert_eq!(result.len(), 2);
        
        assert_eq!(result[0].test(), FilterTest::Neq);
        assert_eq!(result[0].success(), 1);
        assert_eq!(result[0].failure(), FILTER_RESULT_REJECT as u16);
        
        assert_eq!(result[1].test(), FilterTest::Neq);
        assert_eq!(result[1].success(), FILTER_RESULT_ACCEPT as u16);
        assert_eq!(result[1].failure(), FILTER_RESULT_REJECT as u16);
    }
    
    #[test]
    fn should_emit_complex() {
        let filter = "ip && tcp && loopback && (tcp.SrcPort == 53124 || tcp.DstPort == 53124)";
        let mut tokenizer = Tokenizer::new(filter);
        let tokens = tokenizer.tokenize().unwrap();
        let mut parser = Parser::new(&tokens);
        let expr = parser.parse().unwrap();
        let flattener = Flattener::new();
        let (label, cfg) = flattener.flatten(expr);
        
        let emitter = Emitter::new();
        let result = emitter.emit(label, cfg);

        assert_eq!(result.len(), 5);
        
        assert_eq!(result[0].test(), FilterTest::Neq);
        assert_eq!(result[0].success(), 1);
        assert_eq!(result[0].failure(), FILTER_RESULT_REJECT as u16);
        
        assert_eq!(result[1].test(), FilterTest::Neq);
        assert_eq!(result[1].success(), 2);
        assert_eq!(result[1].failure(), FILTER_RESULT_REJECT as u16);
        
        assert_eq!(result[2].test(), FilterTest::Neq);
        assert_eq!(result[2].success(), 3);
        assert_eq!(result[2].failure(), FILTER_RESULT_REJECT as u16);
        
        assert_eq!(result[3].test(), FilterTest::Eq);
        assert_eq!(result[3].success(), FILTER_RESULT_ACCEPT as u16);
        assert_eq!(result[3].failure(), 4);
        
        assert_eq!(result[4].test(), FilterTest::Eq);
        assert_eq!(result[4].success(), FILTER_RESULT_ACCEPT as u16);
        assert_eq!(result[4].failure(), FILTER_RESULT_REJECT as u16);
    }
}