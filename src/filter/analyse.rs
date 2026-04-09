use crate::{filter::*, *};
use crate::constants::*;

pub struct Analyser;

impl Analyser {
    pub fn analyse(inner: &[WinDivertFilterRaw], layer: WinDivertLayer) -> u64 {
        let length = inner.len();
        
        if !Self::cond_exec(inner, length, FilterField::Zero, 0) {
            return 0;
        }

        let mut flags = 0u64;
        
        if matches!(layer, WinDivertLayer::Network | WinDivertLayer::NetworkForward) {
            flags |= Self::check_direction(inner, length);
        }
        
        if layer != WinDivertLayer::Reflect {
            flags |= Self::check_ip_version(inner, length);
        }
        
        flags
    }

    fn check_direction(inner: &[WinDivertFilterRaw], length: usize) -> u64 {
        let mut flags = 0u64;
        
        let inbound_ok = Self::check_condition(inner, length, FilterField::Inbound, 1, FilterField::Outbound, 0);
        if inbound_ok {
            flags |= WinDivertFilterFlag::Inbound as u64;
        }
        
        let outbound_ok = Self::check_condition(inner, length, FilterField::Outbound, 1, FilterField::Inbound, 0);
        if outbound_ok {
            flags |= WinDivertFilterFlag::Outbound as u64;
        }
        
        flags
    }

    fn check_ip_version(inner: &[WinDivertFilterRaw], length: usize) -> u64 {
        let mut flags = 0u64;
        
        let ipv4_ok = Self::check_condition(inner, length, FilterField::Ip, 1, FilterField::Ipv6, 0);
        if ipv4_ok {
            flags |= WinDivertFilterFlag::Ip as u64;
        }
        
        let ipv6_ok = Self::check_condition(inner, length, FilterField::Ipv6, 1, FilterField::Ip, 0);
        if ipv6_ok {
            flags |= WinDivertFilterFlag::Ipv6 as u64;
        }
        
        flags
    }

    fn check_condition(inner: &[WinDivertFilterRaw], length: usize, field1: FilterField, arg1: u32, field2: FilterField, arg2: u32) -> bool {
        let mut res = Self::cond_exec(inner, length, field1, arg1);
        if res {
            res = Self::cond_exec(inner, length, field2, arg2);
        }
        res
    }

    fn cond_exec(inner: &[WinDivertFilterRaw], length: usize, field: FilterField, arg: u32) -> bool {
        if length == 0 {
            return true;
        }

        let mut result = vec![false; length];

        for ip in (0..length).rev() {
            let filter = &inner[ip];
            
            let result_succ = Self::get_branch_result(filter.success(), ip, &result, length);
            let result_fail = Self::get_branch_result(filter.failure(), ip, &result, length);

            let node_result = Self::evaluate_node(filter, field, arg, result_succ, result_fail);
            result[ip] = node_result;
        }

        result[0]
    }

    fn get_branch_result(branch: u16, ip: usize, result: &[bool], length: usize) -> bool {
        match branch as i16 {
            FILTER_RESULT_ACCEPT => true,
            FILTER_RESULT_REJECT => false,
            _ => {
                if branch > ip as u16 && (branch as usize) < length {
                    result[branch as usize]
                } else {
                    true
                }
            }
        }
    }

    fn evaluate_node(filter: &WinDivertFilterRaw, field: FilterField, arg: u32, result_succ: bool, result_fail: bool) -> bool {
        if result_succ && result_fail {
            return true;
        }
        
        if !result_succ && !result_fail {
            return false;
        }
        
        if filter.field() != field {
            return true;
        }
        
        if !filter.is_simple_predicate() {
            return true;
        }
        
        let first_arg = filter.nth_arg(0);

        let test_result = match filter.test() {
            FilterTest::Eq => arg == first_arg,
            FilterTest::Neq => arg != first_arg,
            FilterTest::Lt => arg < first_arg,
            FilterTest::Leq => arg <= first_arg,
            FilterTest::Gt => arg > first_arg,
            FilterTest::Geq => arg >= first_arg,
        };
        
        if test_result { result_succ } else { result_fail }
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn should_evaluate_flags() {
        let filter = "ip && tcp && loopback && (tcp.SrcPort == 53124 || tcp.DstPort == 53124)";
        let mut tokenizer = Tokenizer::new(filter);
        let tokens = tokenizer.tokenize().unwrap();
        let mut parser = Parser::new(&tokens);
        let expr = parser.parse().unwrap();
        let flattener = Flattener::new();
        let (label, cfg) = flattener.flatten(expr);
        
        let emitter = Emitter::new();
        let filters = emitter.emit(label, cfg);
        let layer = WinDivertLayer::Network;
        let flags = Analyser::analyse(&filters, layer);

        assert_eq!(flags, 112);
    }
}