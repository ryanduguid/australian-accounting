//! `config.py`: configuration, and the rule that nothing leaves the machine by default.
//!
//! Three separate things have to be true before a request is sent:
//!
//! 1. remote access is explicitly enabled (`enabled`, or
//!    `LODGEIT_ADAPTER_ENABLED=1` when building from the environment);
//! 2. a base URL is configured;
//! 3. that base URL's host is on the allowlist.
//!
//! None of them has a default that reaches the network.
use num_bigint::BigInt;
use num_traits::{One, Signed, ToPrimitive};

use crate::errors::{CONFIGURATION, DISALLOWED_TARGET, NOT_ENABLED};
use crate::pyops::{Exc, R, pts};
use crate::{py, pyurl, sre};

/// Hosts this adapter is built to talk to. Configuration may narrow this set,
/// never widen it beyond what the caller passes as `allowed_hosts`.
pub const KNOWN_HOSTS: [&str; 3] = [
    "fbt-calculator-api-qkp3j5bjnq-ts.a.run.app",
    "fano-engine-79859053141.australia-southeast1.run.app",
    "fano-engine-afmurhqkaq-ts.a.run.app",
];
pub const LOOPBACK_HOSTS: [&str; 4] = ["127.0.0.1", "localhost", "::1", "[::1]"];

/// Route prefixes a caller may reach. A path outside these is refused before a
/// socket is opened, so a URL taken from a response body cannot become a request.
pub const ALLOWED_PATH_PREFIXES: [&str; 7] = [
    "/v1/calculators",
    "/v1/rates",
    "/openapi.json",
    "/healthz",
    "/livez",
    "/v1/info",
    "/ingest/trial_balance",
];

pub const USER_AGENT: &str =
    "lodgeit-calculator-adapter (https://github.com/ryanduguid/australian-accounting)";

/// `_segments(path)`: the path's segments as a server would see them, after
/// decoding. A gateway decodes before it routes, and a backslash is a path
/// delimiter to a browser and to some servers, so the decoded text is split
/// again on both.
pub fn segments(path: &str) -> Vec<String> {
    path.split('/')
        .flat_map(|segment| {
            pyurl::unquote(segment)
                .replace('\\', "/")
                .split('/')
                .map(str::to_owned)
                .collect::<Vec<_>>()
        })
        .collect()
}

/// Everything the transport is allowed to do.
#[derive(Clone, Debug)]
pub struct AdapterConfig {
    pub enabled: bool,
    pub base_url: Option<String>,
    pub allowed_hosts: Vec<String>,
    pub allow_loopback: bool,
    /// One timeout: it bounds the connection and each socket read.
    pub read_timeout: f64,
    pub max_response_bytes: BigInt,
    pub max_attempts: BigInt,
    pub retry_backoff_seconds: f64,
    pub user_agent: String,
    pub extra_headers: Vec<(String, String)>,
}

impl Default for AdapterConfig {
    /// The dataclass defaults, which `checked` accepts.
    fn default() -> AdapterConfig {
        AdapterConfig {
            enabled: false,
            base_url: None,
            allowed_hosts: KNOWN_HOSTS.map(str::to_owned).to_vec(),
            allow_loopback: false,
            read_timeout: 20.0,
            max_response_bytes: BigInt::from(2 * 1024 * 1024),
            max_attempts: BigInt::from(2),
            retry_backoff_seconds: 1.0,
            user_agent: USER_AGENT.to_owned(),
            extra_headers: Vec::new(),
        }
    }
}

fn refuse(class: &'static str, text: String) -> Exc {
    Exc::new(class, pts(&text))
}

impl AdapterConfig {
    /// `__post_init__`: the dataclass's validation, in its order.
    pub fn checked(self) -> R<AdapterConfig> {
        for (name, value) in [
            ("read_timeout", self.read_timeout),
            ("retry_backoff_seconds", self.retry_backoff_seconds),
        ] {
            // `value <= 0` is false for NaN, which Python lets through.
            if value <= 0.0 {
                return Err(refuse(
                    CONFIGURATION,
                    format!("{name} must be a positive number"),
                ));
            }
        }
        if !self.max_response_bytes.is_positive() {
            return Err(refuse(
                CONFIGURATION,
                "max_response_bytes must be positive".into(),
            ));
        }
        if self.max_attempts < BigInt::one() || self.max_attempts > BigInt::from(5) {
            return Err(refuse(
                CONFIGURATION,
                "max_attempts must be between 1 and 5".into(),
            ));
        }
        if let Some(base_url) = &self.base_url {
            // A base URL carries no route of its own; everything else still
            // applies, including the refusal of a query string.
            self.check_url(base_url, false)?;
        }
        Ok(self)
    }

    /// The attempt budget, which `checked` bounds to 1-5.
    pub fn attempts(&self) -> u32 {
        self.max_attempts
            .to_u32()
            .expect("checked bounds max_attempts")
    }

    pub fn require_enabled(&self) -> R<()> {
        if !self.enabled {
            return Err(refuse(
                NOT_ENABLED,
                "Remote access is not enabled. This adapter never contacts a service unless it \
                 is switched on deliberately: pass enabled=True, or set LODGEIT_ADAPTER_ENABLED=1."
                    .into(),
            ));
        }
        if self.base_url.as_deref().is_none_or(str::is_empty) {
            return Err(refuse(
                CONFIGURATION,
                "No base URL is configured; there is nothing to call.".into(),
            ));
        }
        Ok(())
    }

    /// `check_url(url, require_route=...)`: refuse a URL the configuration
    /// does not allow, on the URL itself and before resolution. A host is
    /// compared literally against the allowlist. `urlsplit`'s own
    /// `ValueError` propagates, as Python let it.
    pub fn check_url(&self, url: &str, require_route: bool) -> R<()> {
        let refused = |text: String| Err(refuse(DISALLOWED_TARGET, format!("{url}: {text}")));
        let parts =
            pyurl::urlsplit(url).map_err(|message| Exc::new("ValueError", pts(&message)))?;
        if pyurl::has_credentials(&parts.netloc) {
            return refused("credentials in a URL are refused".into());
        }
        let host: String =
            sre::str_lower(&pts(&pyurl::hostname(&parts.netloc).unwrap_or_default()))
                .into_iter()
                .filter_map(char::from_u32)
                .collect();
        if host.is_empty() {
            return refused("no host".into());
        }
        let loopback = LOOPBACK_HOSTS.contains(&host.as_str());
        if loopback {
            if !self.allow_loopback {
                return refused(
                    "loopback is only reachable when allow_loopback is set, which is for tests \
                     against a local stub."
                        .into(),
                );
            }
        } else {
            if parts.scheme != "https" {
                return refused("only https is allowed off loopback".into());
            }
            let allowed = self.allowed_hosts.iter().any(|name| lower(name) == host);
            if !allowed {
                let mut names = self.allowed_hosts.join(", ");
                if names.is_empty() {
                    names = "empty".to_owned();
                }
                return refused(format!(
                    "host {host} is not on the configured allowlist ({names})"
                ));
            }
        }
        if parts.scheme != "https" && parts.scheme != "http" {
            return refused(format!("scheme {} is refused", py::repr_str(&parts.scheme)));
        }
        let Ok(port) = pyurl::port(&parts.netloc) else {
            return refused("the port is not a number".into());
        };
        if let Some(port) = port
            && !loopback
            && port != 443
            && port != 80
        {
            return refused(format!("port {port} is not one this adapter calls"));
        }
        if !parts.query.is_empty() || !parts.fragment.is_empty() {
            return refused(
                "a query string or fragment on a base URL would swallow the route built from it"
                    .into(),
            );
        }
        let path = if parts.path.is_empty() {
            "/"
        } else {
            parts.path.as_str()
        };
        if require_route {
            // Compare the decoded segments, whole segment by whole segment:
            // `startswith` on the raw path accepts `/v1/calculators/../../admin`,
            // `/v1/ratesheet-evil` and a `%2e%2e` dot segment.
            let decoded = segments(path);
            if decoded.iter().any(|s| s == ".." || s == ".") {
                return refused(
                    "the path contains a dot segment, so where it lands depends on who \
                     normalises it"
                        .into(),
                );
            }
            let normalised = decoded.join("/");
            if normalised != path {
                return refused(
                    "the path is not in its decoded form, so this check and the server would \
                     be reading different routes"
                        .into(),
                );
            }
            let routed = ALLOWED_PATH_PREFIXES.iter().any(|prefix| {
                normalised == *prefix
                    || normalised
                        .strip_prefix(prefix)
                        .is_some_and(|rest| rest.starts_with('/'))
            });
            if !routed {
                return refused(format!(
                    "path {path} is not one of the routes this adapter calls. A URL found in a \
                     response body is data, not a destination."
                ));
            }
        }
        Ok(())
    }
}

/// `str.lower()`.
fn lower(text: &str) -> String {
    sre::str_lower(&pts(text))
        .into_iter()
        .filter_map(char::from_u32)
        .collect()
}

/// `int(value)` of a float: the OverflowError or ValueError Python raised
/// uncaught for infinity and NaN.
fn float_int(value: f64) -> R<BigInt> {
    if value.is_nan() {
        return Err(Exc::msg(
            "ValueError",
            "cannot convert float NaN to integer",
        ));
    }
    if value.is_infinite() {
        return Err(Exc::msg(
            "OverflowError",
            "cannot convert float infinity to integer",
        ));
    }
    Ok(crate::pyvalue::float_to_int(value.trunc()))
}

/// The variables [`from_environment`] reads.
const ENVIRONMENT: [&str; 7] = [
    "LODGEIT_ADAPTER_ALLOWED_HOSTS",
    "LODGEIT_ADAPTER_ENABLED",
    "LODGEIT_ADAPTER_BASE_URL",
    "LODGEIT_ADAPTER_ALLOW_LOOPBACK",
    "LODGEIT_ADAPTER_READ_TIMEOUT",
    "LODGEIT_ADAPTER_MAX_RESPONSE_BYTES",
    "LODGEIT_ADAPTER_MAX_ATTEMPTS",
];

/// `from_environment(os.environ)` over the values as the operating system
/// holds them. Python keeps undecodable bytes as surrogates; this port cannot
/// carry them, so it refuses such a value rather than read it with
/// replacement characters, which could name another host.
pub fn from_os_environment(env: impl Fn(&str) -> Option<std::ffi::OsString>) -> R<AdapterConfig> {
    if let Some(name) = ENVIRONMENT
        .into_iter()
        .find(|name| env(name).is_some_and(|value| value.to_str().is_none()))
    {
        return Err(refuse(
            CONFIGURATION,
            format!("{name} is not valid Unicode"),
        ));
    }
    from_environment(|name| env(name).and_then(|value| value.into_string().ok()))
}

/// `from_environment(environ)`: a config from the environment, still disabled
/// unless asked. `env` is `environ.get`.
pub fn from_environment(env: impl Fn(&str) -> Option<String>) -> R<AdapterConfig> {
    let mut allowed_hosts = KNOWN_HOSTS.map(str::to_owned).to_vec();
    if let Some(hosts) = env("LODGEIT_ADAPTER_ALLOWED_HOSTS").filter(|h| !h.is_empty()) {
        allowed_hosts = hosts
            .split(',')
            .map(py::strip)
            .filter(|part| !part.is_empty())
            .map(str::to_owned)
            .collect();
    }
    let number = |name: &str, fallback: f64| -> R<f64> {
        match env(name) {
            None => Ok(fallback),
            Some(raw) if raw.is_empty() => Ok(fallback),
            Some(raw) => py::parse_float(&raw).map_err(|_| {
                refuse(
                    CONFIGURATION,
                    format!("{name}={} is not a number", py::repr_str(&raw)),
                )
            }),
        }
    };
    let enabled = env("LODGEIT_ADAPTER_ENABLED").as_deref() == Some("1");
    let base_url = env("LODGEIT_ADAPTER_BASE_URL").filter(|u| !u.is_empty());
    let allow_loopback = env("LODGEIT_ADAPTER_ALLOW_LOOPBACK").as_deref() == Some("1");
    let read_timeout = number("LODGEIT_ADAPTER_READ_TIMEOUT", 20.0)?;
    let max_response_bytes = float_int(number(
        "LODGEIT_ADAPTER_MAX_RESPONSE_BYTES",
        f64::from(2 * 1024 * 1024),
    )?)?;
    let max_attempts = float_int(number("LODGEIT_ADAPTER_MAX_ATTEMPTS", 2.0)?)?;
    AdapterConfig {
        enabled,
        base_url,
        allowed_hosts,
        allow_loopback,
        read_timeout,
        max_response_bytes,
        max_attempts,
        ..AdapterConfig::default()
    }
    .checked()
}

/// `from_environment()` over the process environment.
pub fn from_process_environment() -> R<AdapterConfig> {
    from_os_environment(|name| std::env::var_os(name))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn config(loopback: bool) -> AdapterConfig {
        AdapterConfig {
            allow_loopback: loopback,
            ..AdapterConfig::default()
        }
    }

    fn refusal(url: &str, route: bool) -> String {
        let error = config(false).check_url(url, route).unwrap_err();
        error.text.into_iter().filter_map(char::from_u32).collect()
    }

    #[test]
    fn routes_are_matched_on_decoded_whole_segments() {
        let host = "https://fano-engine-afmurhqkaq-ts.a.run.app";
        assert!(
            config(false)
                .check_url(&format!("{host}/v1/rates/x"), true)
                .is_ok()
        );
        assert!(
            config(false)
                .check_url(&format!("{host}:443/v1/rates"), true)
                .is_ok()
        );
        for (path, tail) in [
            ("/v1/ratesheet-evil", "is not one of the routes"),
            ("/v1/calculators/%2e%2e/admin", "contains a dot segment"),
            ("/v1/calculators/..%2f..%2fadmin", "contains a dot segment"),
            ("/v1/calculators/a%2Fb", "is not in its decoded form"),
            ("/v1/calculators/a\\b", "is not in its decoded form"),
        ] {
            let text = refusal(&format!("{host}{path}"), true);
            assert!(text.contains(tail), "{path}: {text}");
        }
        assert_eq!(
            refusal("https://u:@fano-engine-afmurhqkaq-ts.a.run.app/", false),
            "https://u:@fano-engine-afmurhqkaq-ts.a.run.app/: credentials in a URL are refused"
        );
        assert!(
            config(false)
                .check_url("https://:@fano-engine-afmurhqkaq-ts.a.run.app/", false)
                .is_ok()
        );
        assert_eq!(
            refusal("http://localhost:8080/v1/rates", true),
            "http://localhost:8080/v1/rates: loopback is only reachable when allow_loopback is \
             set, which is for tests against a local stub."
        );
        assert!(
            config(true)
                .check_url("http://[::1]:8080/v1/rates", true)
                .is_ok()
        );
        assert_eq!(
            config(true)
                .check_url("http://[::1/v1", true)
                .unwrap_err()
                .class,
            "ValueError"
        );
    }

    #[test]
    fn a_value_that_is_not_unicode_is_refused_not_replaced() {
        use std::ffi::OsString;
        #[cfg(windows)]
        let undecodable =
            <OsString as std::os::windows::ffi::OsStringExt>::from_wide(&[0x68, 0xD800]);
        #[cfg(unix)]
        let undecodable = <OsString as std::os::unix::ffi::OsStringExt>::from_vec(vec![b'h', 0xff]);
        for name in ENVIRONMENT {
            let env = |asked: &str| (asked == name).then(|| undecodable.clone());
            let error = from_os_environment(env).unwrap_err();
            assert_eq!(error.class, CONFIGURATION);
            assert_eq!(
                crate::pyvalue::points_lossy(&error.text),
                format!("{name} is not valid Unicode")
            );
        }
        let plain = |name: &str| (name == "LODGEIT_ADAPTER_ENABLED").then(|| OsString::from("1"));
        assert!(from_os_environment(plain).unwrap().enabled);
    }

    #[test]
    fn the_environment_is_read_through_float_and_int() {
        let env = |pairs: &'static [(&'static str, &'static str)]| {
            move |name: &str| {
                pairs
                    .iter()
                    .find(|(k, _)| *k == name)
                    .map(|(_, v)| (*v).to_owned())
            }
        };
        let c = from_environment(env(&[
            ("LODGEIT_ADAPTER_MAX_ATTEMPTS", " 5.9 "),
            ("LODGEIT_ADAPTER_READ_TIMEOUT", "nan"),
            ("LODGEIT_ADAPTER_ALLOWED_HOSTS", " a , ,b"),
        ]))
        .unwrap();
        assert_eq!(
            (c.attempts(), c.allowed_hosts.clone()),
            (5, vec!["a".into(), "b".into()])
        );
        assert!(c.read_timeout.is_nan());
        let text = |e: Exc| {
            (
                e.class,
                e.text
                    .into_iter()
                    .filter_map(char::from_u32)
                    .collect::<String>(),
            )
        };
        assert_eq!(
            text(from_environment(env(&[("LODGEIT_ADAPTER_MAX_ATTEMPTS", "x")])).unwrap_err()),
            (
                CONFIGURATION,
                "LODGEIT_ADAPTER_MAX_ATTEMPTS='x' is not a number".into()
            )
        );
        assert_eq!(
            text(
                from_environment(env(&[("LODGEIT_ADAPTER_MAX_RESPONSE_BYTES", "-inf")]))
                    .unwrap_err()
            ),
            (
                "OverflowError",
                "cannot convert float infinity to integer".into()
            )
        );
        assert_eq!(
            text(from_environment(env(&[("LODGEIT_ADAPTER_MAX_ATTEMPTS", "0.5")])).unwrap_err()),
            (CONFIGURATION, "max_attempts must be between 1 and 5".into())
        );
    }
}
