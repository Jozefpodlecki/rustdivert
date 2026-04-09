use crate::{constants::*, filter::*};

#[derive(Debug, Default)]
pub struct Flattener {
    pub stack: Vec<Expression>,
    pub label: i16,
}

impl Flattener {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn flatten(mut self, root: Box<Expression>) -> (i16, Vec<Expression>) {
        let mut label = self.label;
        let result = self.flatten_expr(*root, &mut label, FILTER_RESULT_ACCEPT, FILTER_RESULT_REJECT);
        (result, self.stack)
    }

    fn flatten_expr(&mut self, expr: Expression, label: &mut i16, succ: i16, fail: i16) -> i16 {
        if succ < 0 || fail < 0 {
            return -1;
        }

        match expr.kind {
            TokenKind::And => {
                let (left, right) = expr.take_binary();
                let succ = self.flatten_expr(right, label, succ, fail);
                let succ = self.flatten_expr(left, label, succ, fail);
                succ
            }
            TokenKind::Or => {
                let (left, right) = expr.take_binary();
                let fail = self.flatten_expr(right, label, succ, fail);
                let fail = self.flatten_expr(left, label, succ, fail);
                fail
            }
            TokenKind::Question => {
                let (cond, then_expr, else_expr) = expr.take_ternary();
                let fail1 = self.flatten_expr(else_expr, label, succ, fail);
                let succ1 = self.flatten_expr(then_expr, label, succ, fail);
                let succ = self.flatten_expr(cond, label, succ1, fail1);
                succ
            }
            _ => {
                let mut leaf = expr;
                leaf.simplify_test();

                if let Some(result) = leaf.shortcut_branch(succ, fail) {
                    return result
                }
                
                if *label >= 256 {
                    return -1;
                }
                
                leaf.succ = succ as u16;
                leaf.fail = fail as u16;
                self.stack.push(leaf);
                let result = *label;
                *label += 1;
                result
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::{WinDivertLayer, filter::Tokenizer};
    use super::*;

    fn verify_leaf_eq(expr: &Expression, expected_var: TokenKind, expected_val: u32, expected_succ: u16, expected_fail: u16) {
        assert_eq!(expr.kind, TokenKind::Eq);
        assert_eq!(expr.succ, expected_succ);
        assert_eq!(expr.fail, expected_fail);
        if let ExpressionData::Binary { left, right } = &expr.data {
            if let ExpressionData::Var(kind) = &left.data {
                assert_eq!(*kind, expected_var);
            } else {
                panic!("Expected Var, got {:?}", left.data);
            }
            if let ExpressionData::Number { values, .. } = &right.data {
                assert_eq!(values[0], expected_val);
            } else {
                panic!("Expected Number, got {:?}", right.data);
            }
        } else {
            panic!("Expected Binary, got {:?}", expr.data);
        }
    }

    fn verify_leaf_neq(expr: &Expression, expected_var: TokenKind, expected_succ: u16, expected_fail: u16) {
        assert_eq!(expr.kind, TokenKind::Neq);
        assert_eq!(expr.succ, expected_succ);
        assert_eq!(expr.fail, expected_fail);
        if let ExpressionData::Binary { left, right } = &expr.data {
            if let ExpressionData::Var(kind) = &left.data {
                assert_eq!(*kind, expected_var);
            } else {
                panic!("Expected Var, got {:?}", left.data);
            }
        } else {
            panic!("Expected Binary, got {:?}", expr.data);
        }
    }

    #[test]
    fn should_flatten_complex() {
        let filter = "ip && tcp && loopback && (tcp.SrcPort == 53124 || tcp.DstPort == 53124)";
        let mut tokenizer = Tokenizer::new(filter);
        let tokens = tokenizer.tokenize().unwrap();
        let mut parser = Parser::new(&tokens);
        let expr = parser.parse().unwrap();
        let flattener = Flattener::new();

        let (label, cfg) = flattener.flatten(expr);
        
        assert_eq!(label, 4);
        assert_eq!(cfg.len(), 5);
       
        verify_leaf_eq(&cfg[0], TokenKind::TcpDstPort, 53124, 
                       FILTER_RESULT_ACCEPT as u16, FILTER_RESULT_REJECT as u16);
        verify_leaf_eq(&cfg[1], TokenKind::TcpSrcPort, 53124, 
                       FILTER_RESULT_ACCEPT as u16, 0);

        verify_leaf_neq(&cfg[2], TokenKind::Loopback, 1, FILTER_RESULT_REJECT as u16);
        verify_leaf_neq(&cfg[3], TokenKind::Tcp, 2, FILTER_RESULT_REJECT as u16);
        verify_leaf_neq(&cfg[4], TokenKind::Ip, 3, FILTER_RESULT_REJECT as u16);
    }

    #[test]
    fn should_flatten_simple_and() {
        let filter = "ip && tcp";
        let mut tokenizer = Tokenizer::new(filter);
        let tokens = tokenizer.tokenize().unwrap();
        let mut parser = Parser::new(&tokens);
        let expr = parser.parse().unwrap();
        let flattener = Flattener::new();

        let (label, cfg) = flattener.flatten(expr);
        
        assert_eq!(label, 1);
        assert_eq!(cfg.len(), 2);
        
        verify_leaf_neq(&cfg[0], TokenKind::Tcp, FILTER_RESULT_ACCEPT as u16, FILTER_RESULT_REJECT as u16);
        verify_leaf_neq(&cfg[1], TokenKind::Ip, 0, FILTER_RESULT_REJECT as u16);
    }

    #[test]
    fn should_flatten_simple_or() {
        let filter = "tcp.SrcPort == 80 || tcp.DstPort == 80";
        let mut tokenizer = Tokenizer::new(filter);
        let tokens = tokenizer.tokenize().unwrap();
        let mut parser = Parser::new(&tokens);
        let expr = parser.parse().unwrap();
        let flattener = Flattener::new();

        let (label, cfg) = flattener.flatten(expr);
        
        assert_eq!(label, 1);
        assert_eq!(cfg.len(), 2);
        
        verify_leaf_eq(&cfg[0], TokenKind::TcpDstPort, 80, 
                       FILTER_RESULT_ACCEPT as u16, FILTER_RESULT_REJECT as u16);
        verify_leaf_eq(&cfg[1], TokenKind::TcpSrcPort, 80, 
                       FILTER_RESULT_ACCEPT as u16, 0);
    }
}