use std::sync::atomic::{AtomicU64, Ordering};

/// Generate a unique request ID
pub fn generate_request_id() -> u64 {
    static COUNTER: AtomicU64 = AtomicU64::new(1);
    COUNTER.fetch_add(1, Ordering::Relaxed)
}

/// Convert domain and method into a full method name
pub fn make_method_name(domain: &str, method: &str) -> String {
    format!("{}.{}", domain, method)
} 