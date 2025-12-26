use alloy_primitives::{Address, U256};
use std::collections::HashSet;

/// Format a `U256` fixed-point integer into a decimal string, trimming trailing zeros.
///
/// Example (18 decimals):
/// - `1_000_000_000_000_000_000` -> `"1"`
/// - `1_500_000_000_000_000_000` -> `"1.5"`
pub fn format_units(value: U256, decimals: u32) -> String {
    let factor = U256::from(10).pow(U256::from(decimals));
    let whole = value / factor;
    let fractional = value % factor;

    if fractional.is_zero() {
        whole.to_string()
    } else {
        let fractional_str = format!("{:0>width$}", fractional, width = decimals as usize)
            .trim_end_matches('0')
            .to_string();
        format!("{}.{}", whole, fractional_str)
    }
}

/// Deduplicate addresses while preserving the first-seen order.
pub fn unique_preserve_order(addresses: impl IntoIterator<Item = Address>) -> Vec<Address> {
    let mut seen = HashSet::new();
    let mut out = Vec::new();
    for address in addresses {
        if seen.insert(address) {
            out.push(address);
        }
    }
    out
}

/// Redact sensitive parts of an RPC URL.
///
/// The intent is to avoid leaking API keys/userinfo/query params in logs, while still giving a
/// useful hint about which host is being used.
pub fn redact_rpc_url(url: &str) -> String {
    let base = url.split(['?', '#']).next().unwrap_or(url);

    // If we have a scheme, keep it; always strip userinfo + path.
    if let Some((scheme, rest)) = base.split_once("://") {
        let rest = rest
            .rsplit_once('@')
            .map(|(_, after)| after)
            .unwrap_or(rest);
        let host = rest.split('/').next().unwrap_or(rest);
        format!("{scheme}://{host}")
    } else {
        // No scheme: just strip userinfo + path.
        let rest = base
            .rsplit_once('@')
            .map(|(_, after)| after)
            .unwrap_or(base);
        rest.split('/').next().unwrap_or(rest).to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn format_units_zero() {
        assert_eq!(format_units(U256::ZERO, 18), "0");
    }

    #[test]
    fn format_units_zero_decimals_is_identity() {
        assert_eq!(format_units(U256::from(123), 0), "123");
    }

    #[test]
    fn format_units_whole_only() {
        let one = U256::from(10).pow(U256::from(18));
        assert_eq!(format_units(one, 18), "1");
        assert_eq!(format_units(one * U256::from(42), 18), "42");
    }

    #[test]
    fn format_units_trims_trailing_zeros() {
        let factor = U256::from(10).pow(U256::from(18));
        assert_eq!(format_units(factor + factor / U256::from(2), 18), "1.5");
        assert_eq!(format_units(factor + factor / U256::from(10), 18), "1.1");
    }

    #[test]
    fn format_units_keeps_fractional_precision() {
        let factor = U256::from(10).pow(U256::from(18));
        assert_eq!(
            format_units(factor + U256::from(1), 18),
            "1.000000000000000001"
        );
    }

    #[test]
    fn unique_preserve_order_keeps_first_seen_order() {
        let a = Address::from([0x11; 20]);
        let b = Address::from([0x22; 20]);
        let c = Address::from([0x33; 20]);

        let out = unique_preserve_order([a, b, a, c, b]);
        assert_eq!(out, vec![a, b, c]);
    }

    #[test]
    fn redact_rpc_url_hides_userinfo_and_query() {
        let input = "https://user:pass@rpc.example.com/path?api_key=secret#frag";
        assert_eq!(redact_rpc_url(input), "https://rpc.example.com");
    }

    #[test]
    fn redact_rpc_url_plain_host() {
        assert_eq!(
            redact_rpc_url("https://rpc.example.com"),
            "https://rpc.example.com"
        );
        assert_eq!(
            redact_rpc_url("rpc.example.com:8545/path"),
            "rpc.example.com:8545"
        );
    }
}
