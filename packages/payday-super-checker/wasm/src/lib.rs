//! The Payday Super due date derivation for wasm32-unknown-unknown.
//!
//! Exports, all over UTF-8 JSON in linear memory:
//!
//! - `alloc(len) -> ptr` and `dealloc(ptr, len)` for the host's buffers;
//! - `derive(ptr, len)` and `verify(ptr, len)`, each returning
//!   `(result_ptr << 32) | result_len` for `{"ok": ...}` or `{"error": "..."}`,
//!   which the host frees with `dealloc`.
//!
//! `derive` takes the `derive` command's input and `verify` a saved
//! derivation, exactly as the command line does.

use serde_json::{Value, json};

/// One call: JSON in, `{"ok": ...}` or `{"error": "..."}` out.
pub fn call(f: fn(&Value) -> Result<Value, String>, input: &[u8]) -> Vec<u8> {
    let result = if input.len() > 8 << 20 {
        Err("JSON input exceeds 8 MiB".to_string())
    } else {
        serde_json::from_slice(input)
            .map_err(|e| format!("invalid JSON: {e}"))
            .and_then(|v| f(&v))
    };
    let out = match result {
        Ok(v) => json!({"ok": v}),
        Err(m) => json!({"error": m}),
    };
    out.to_string().into_bytes()
}

#[cfg(target_arch = "wasm32")]
mod exports {
    use super::*;
    use payday_super_checker::derivation;

    #[unsafe(no_mangle)]
    pub extern "C" fn alloc(len: usize) -> *mut u8 {
        Box::into_raw(vec![0u8; len].into_boxed_slice()).cast()
    }

    /// # Safety
    /// `ptr` and `len` must come from `alloc` or a returned result, once.
    #[unsafe(no_mangle)]
    pub unsafe extern "C" fn dealloc(ptr: *mut u8, len: usize) {
        drop(unsafe { Box::from_raw(std::ptr::slice_from_raw_parts_mut(ptr, len)) });
    }

    unsafe fn run(f: fn(&Value) -> Result<Value, String>, ptr: *const u8, len: usize) -> u64 {
        let input = unsafe { std::slice::from_raw_parts(ptr, len) };
        let out = Box::into_raw(call(f, input).into_boxed_slice());
        ((out.cast::<u8>() as u64) << 32) | out.len() as u64
    }

    /// # Safety
    /// `ptr` and `len` must describe a live buffer from `alloc`.
    #[unsafe(no_mangle)]
    pub unsafe extern "C" fn derive(ptr: *const u8, len: usize) -> u64 {
        unsafe { run(derivation::derive, ptr, len) }
    }

    /// # Safety
    /// `ptr` and `len` must describe a live buffer from `alloc`.
    #[unsafe(no_mangle)]
    pub unsafe extern "C" fn verify(ptr: *const u8, len: usize) -> u64 {
        unsafe { run(derivation::verify, ptr, len) }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use payday_super_checker::derivation;

    #[test]
    fn calls_wrap_results_and_errors() {
        let out: Value = serde_json::from_slice(&call(derivation::verify, b"{}")).unwrap();
        assert_eq!(
            out,
            json!({"error": "not a payday-super-checker/derivation/1 document"})
        );
        let out: Value = serde_json::from_slice(&call(derivation::derive, b"[")).unwrap();
        assert!(out["error"].as_str().unwrap().starts_with("invalid JSON: "));
        let input = br#"{"qe_day": "2026-07-09", "first_to_fund": true}"#;
        let out: Value = serde_json::from_slice(&call(derivation::derive, input)).unwrap();
        assert_eq!(out["ok"]["result"]["due"], "2026-08-07");
    }
}

#[cfg(test)]
#[test]
fn json_input_limit_matches_the_native_cli() {
    fn accept(_: &Value) -> Result<Value, String> {
        Ok(json!(true))
    }
    let mut bytes = vec![b' '; 8 << 20];
    bytes[0] = b'0';
    assert_eq!(
        serde_json::from_slice::<Value>(&call(accept, &bytes)).unwrap(),
        json!({"ok": true})
    );
    bytes.push(b' ');
    assert_eq!(
        serde_json::from_slice::<Value>(&call(accept, &bytes)).unwrap(),
        json!({"error": "JSON input exceeds 8 MiB"})
    );
}
