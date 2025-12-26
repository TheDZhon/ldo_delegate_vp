use alloy_primitives::{Address, U256};
use std::collections::HashSet;
use url::Url;

/// Format a `U256` fixed-point integer into a decimal string, trimming trailing zeros.
///
/// # Arguments
///
/// * `value` - The raw integer value (e.g., in wei for 18 decimals)
/// * `decimals` - Number of decimal places (e.g., 18 for LDO/ETH tokens)
///
/// # Examples
///
/// ```
/// use alloy_primitives::U256;
/// use ldo_delegate_vp::format_units;
///
/// // 1 LDO (18 decimals)
/// let one_ldo = U256::from(1_000_000_000_000_000_000u64);
/// assert_eq!(format_units(one_ldo, 18), "1");
///
/// // 1.5 LDO
/// let one_and_half = U256::from(1_500_000_000_000_000_000u64);
/// assert_eq!(format_units(one_and_half, 18), "1.5");
///
/// // Zero
/// assert_eq!(format_units(U256::ZERO, 18), "0");
/// ```
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

/// Format a `U256` fixed-point integer into a human-readable string with thousand separators.
///
/// Large values show fewer decimals for readability; small values preserve precision.
///
/// # Examples
///
/// ```
/// use alloy_primitives::U256;
/// use ldo_delegate_vp::format_units_human;
///
/// // Large amount: 1,234,567 LDO (no decimals shown)
/// let large = U256::from(1_234_567u64) * U256::from(10).pow(U256::from(18));
/// assert_eq!(format_units_human(large, 18), "1,234,567");
///
/// // Medium amount: shows 2 decimals
/// let medium = U256::from(1_234_560_000_000_000_000_000u128);
/// assert_eq!(format_units_human(medium, 18), "1,234.56");
/// ```
pub fn format_units_human(value: U256, decimals: u32) -> String {
    let factor = U256::from(10).pow(U256::from(decimals));
    let whole = value / factor;
    let fractional = value % factor;

    // Determine display precision based on whole part magnitude
    let display_decimals = if whole >= U256::from(10_000) {
        0 // Large amounts: no decimals
    } else if whole >= U256::from(100) {
        2 // Medium amounts: 2 decimals
    } else if whole >= U256::from(1) {
        4 // Small amounts: 4 decimals
    } else {
        decimals // Tiny amounts: full precision
    };

    // Format whole part with thousand separators
    let whole_str = add_thousand_separators(&whole.to_string());

    if fractional.is_zero() || display_decimals == 0 {
        whole_str
    } else {
        let fractional_str = format!("{:0>width$}", fractional, width = decimals as usize);
        let truncated = &fractional_str[..display_decimals.min(decimals) as usize];
        let trimmed = truncated.trim_end_matches('0');
        if trimmed.is_empty() {
            whole_str
        } else {
            format!("{}.{}", whole_str, trimmed)
        }
    }
}

/// Add thousand separators (commas) to a numeric string.
fn add_thousand_separators(s: &str) -> String {
    let chars: Vec<char> = s.chars().collect();
    let mut result = String::with_capacity(s.len() + s.len() / 3);
    for (i, c) in chars.iter().enumerate() {
        if i > 0 && (chars.len() - i).is_multiple_of(3) {
            result.push(',');
        }
        result.push(*c);
    }
    result
}

/// Deduplicate addresses while preserving the first-seen order.
///
/// # Examples
///
/// ```
/// use alloy_primitives::Address;
/// use ldo_delegate_vp::unique_preserve_order;
///
/// let a = Address::from([0x11; 20]);
/// let b = Address::from([0x22; 20]);
///
/// let result = unique_preserve_order([a, b, a, b]);
/// assert_eq!(result, vec![a, b]);
/// ```
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
///
/// # Examples
///
/// ```
/// use ldo_delegate_vp::redact_rpc_url;
///
/// // API key in path is removed
/// assert_eq!(
///     redact_rpc_url("https://mainnet.infura.io/v3/secret-key"),
///     "https://mainnet.infura.io"
/// );
///
/// // Username and password are removed
/// assert_eq!(
///     redact_rpc_url("https://user:pass@rpc.example.com"),
///     "https://rpc.example.com"
/// );
/// ```
pub fn redact_rpc_url(url: &str) -> String {
    // Try to parse as a URL first
    if let Ok(mut parsed) = Url::parse(url)
        && parsed.has_host()
    {
        let _ = parsed.set_username("");
        let _ = parsed.set_password(None);
        parsed.set_path("");
        parsed.set_query(None);
        parsed.set_fragment(None);
        let s = parsed.to_string();
        // remove trailing slash which Url::to_string() adds for empty path
        return s.trim_end_matches('/').to_string();
    }

    // Fallback: manual parsing for scheme-less or invalid URLs
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

    #[test]
    fn redact_rpc_url_complex() {
        let input = "https://user:pass@sub.domain.com:8545/path/to/resource?query=123";
        assert_eq!(redact_rpc_url(input), "https://sub.domain.com:8545");
    }

    // ── Additional edge case tests ──────────────────────────────────────────

    #[test]
    fn unique_preserve_order_empty_input() {
        let out = unique_preserve_order(std::iter::empty::<Address>());
        assert!(out.is_empty());
    }

    #[test]
    fn unique_preserve_order_single_element() {
        let a = Address::from([0xAA; 20]);
        let out = unique_preserve_order([a]);
        assert_eq!(out, vec![a]);
    }

    #[test]
    fn unique_preserve_order_all_duplicates() {
        let a = Address::from([0x99; 20]);
        let out = unique_preserve_order([a, a, a, a]);
        assert_eq!(out, vec![a]);
    }

    #[test]
    fn format_units_fractional_only() {
        // Value smaller than 1 unit (e.g., 0.5 LDO)
        let half = U256::from(500_000_000_000_000_000u64);
        assert_eq!(format_units(half, 18), "0.5");

        // Very small value
        let tiny = U256::from(1u64);
        assert_eq!(format_units(tiny, 18), "0.000000000000000001");
    }

    #[test]
    fn format_units_large_value() {
        // 1 million LDO
        let one_million = U256::from(1_000_000u64) * U256::from(10).pow(U256::from(18));
        assert_eq!(format_units(one_million, 18), "1000000");
    }

    #[test]
    fn format_units_different_decimals() {
        // 6 decimals (like USDC)
        let one_usdc = U256::from(1_000_000u64);
        assert_eq!(format_units(one_usdc, 6), "1");

        let half_usdc = U256::from(500_000u64);
        assert_eq!(format_units(half_usdc, 6), "0.5");
    }

    #[test]
    fn redact_rpc_url_websocket() {
        assert_eq!(
            redact_rpc_url("wss://user:key@mainnet.infura.io/ws/v3/secret"),
            "wss://mainnet.infura.io"
        );
    }

    #[test]
    fn redact_rpc_url_localhost() {
        assert_eq!(
            redact_rpc_url("http://localhost:8545/"),
            "http://localhost:8545"
        );
        assert_eq!(
            redact_rpc_url("http://127.0.0.1:8545"),
            "http://127.0.0.1:8545"
        );
    }

    #[test]
    fn redact_rpc_url_empty_string() {
        // Edge case: empty string should return empty
        assert_eq!(redact_rpc_url(""), "");
    }
}
