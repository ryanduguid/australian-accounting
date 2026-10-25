//! The Worker shares the checked offline renderer with the native prototype.

#![cfg(any(target_arch = "wasm32", test))]

const TTL_MS: u64 = 300_000;
#[cfg(not(feature = "synthetic-trial"))]
const ALLOWED: &str = include_str!("../../glama-badge-server/allowed.txt");

#[cfg(not(feature = "synthetic-trial"))]
fn identity(path: &str, has_query: bool) -> Option<&str> {
    let id = path.strip_prefix("/glama/")?.strip_suffix(".svg")?;
    (!has_query && ALLOWED.lines().any(|allowed| allowed == id)).then_some(id)
}

fn cache_age(fetched: u64, now: u64) -> Option<u64> {
    let elapsed = now.checked_sub(fetched)?;
    (elapsed < TTL_MS).then(|| ((TTL_MS - elapsed) / 1_000).min(60))
}

#[cfg(target_arch = "wasm32")]
mod http;
#[cfg(all(target_arch = "wasm32", not(feature = "synthetic-trial")))]
mod runtime;
#[cfg(all(target_arch = "wasm32", feature = "synthetic-trial"))]
mod synthetic;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[cfg(not(feature = "synthetic-trial"))]
    fn exact_public_routes_only() {
        for id in ALLOWED.lines() {
            let path = format!("/glama/{id}.svg");
            assert_eq!(identity(&path, false), Some(id));
            assert_eq!(identity(&path, true), None);
        }
        for path in [
            "/glama/https://evil.example.svg",
            "/glama/servers/ryanduguid%2faustralian-accounting.svg",
            "/glama/servers/ryanduguid/australian-accounting.svg/",
            "/glama/servers/ryanduguid/australian-accounting.svg?x=1",
            "/glama/../servers/ryanduguid/australian-accounting.svg",
            "/",
        ] {
            assert_eq!(identity(path, false), None, "{path}");
        }
        assert_eq!(ALLOWED.lines().count(), 2_947);
    }

    #[test]
    fn cache_never_extends_source_freshness() {
        assert_eq!(cache_age(10, 10), Some(60));
        assert_eq!(cache_age(10, 250_010), Some(50));
        assert_eq!(cache_age(10, 299_010), Some(1));
        assert_eq!(cache_age(10, 300_009), Some(0));
        assert_eq!(cache_age(10, 300_010), None);
        assert_eq!(cache_age(10, 9), None);
    }
}
