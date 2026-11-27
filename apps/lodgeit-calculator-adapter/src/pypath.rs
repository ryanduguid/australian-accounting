//! `pathlib.Path` and `os.path` as CPython 3.15 parses, prints and joins paths.
//!
//! The radar prints its output paths and names its inputs in messages, so a
//! path is carried in pathlib's parsed form: drive, root and the non-empty,
//! non-`.` parts.
use std::path::PathBuf;

#[cfg(windows)]
pub const SEP: &str = "\\";
#[cfg(not(windows))]
pub const SEP: &str = "/";

#[derive(Clone, Debug)]
pub struct PyPath {
    drv: String,
    root: String,
    tail: Vec<String>,
}

/// `os.path.splitroot`.
#[cfg(windows)]
pub fn splitroot(p: &str) -> (&str, &str, &str) {
    let normp = p.replace('/', "\\");
    let n = normp.as_bytes();
    if n.first() == Some(&b'\\') {
        if n.get(1) == Some(&b'\\') {
            // Bytes, not `normp[..8]`: byte 8 can fall inside a character.
            let start = if n.len() >= 8 && n[..8].eq_ignore_ascii_case(b"\\\\?\\UNC\\") {
                8
            } else {
                2
            };
            let Some(index) = normp[start..].find('\\').map(|i| i + start) else {
                return (p, "", "");
            };
            let Some(index2) = normp[index + 1..].find('\\').map(|i| i + index + 1) else {
                return (p, "", "");
            };
            return (&p[..index2], &p[index2..=index2], &p[index2 + 1..]);
        }
        return ("", &p[..1], &p[1..]);
    }
    if n.get(1) == Some(&b':') && p.is_char_boundary(2) {
        if n.get(2) == Some(&b'\\') {
            return (&p[..2], &p[2..3], &p[3..]);
        }
        return (&p[..2], "", &p[2..]);
    }
    ("", "", p)
}

/// `os.path.splitroot`.
#[cfg(not(windows))]
pub fn splitroot(p: &str) -> (&str, &str, &str) {
    if !p.starts_with('/') {
        ("", "", p)
    } else if !p[1..].starts_with('/') || p[2..].starts_with('/') {
        ("", "/", &p[1..])
    } else {
        ("", &p[..2], &p[2..])
    }
}

/// `os.path.join(a, b)`.
#[cfg(windows)]
pub fn join(a: &str, b: &str) -> String {
    let (mut drive, mut root, path) = splitroot(a);
    let mut path = path.to_string();
    let (p_drive, p_root, p_path) = splitroot(b);
    if !p_root.is_empty() {
        if !p_drive.is_empty() || drive.is_empty() {
            drive = p_drive;
        }
        root = p_root;
        path = p_path.to_string();
    } else {
        let mut relative = true;
        if !p_drive.is_empty() && p_drive != drive {
            if p_drive.to_lowercase() != drive.to_lowercase() {
                drive = p_drive;
                root = p_root;
                path = p_path.to_string();
                relative = false;
            } else {
                drive = p_drive;
            }
        }
        if relative {
            if path.ends_with(|c: char| c != '\\' && c != '/') {
                path.push('\\');
            }
            path.push_str(p_path);
        }
    }
    if !path.is_empty() && root.is_empty() && drive.ends_with(|c: char| !":\\/".contains(c)) {
        return format!("{drive}\\{path}");
    }
    format!("{drive}{root}{path}")
}

/// `os.path.join(a, b)`.
#[cfg(not(windows))]
pub fn join(a: &str, b: &str) -> String {
    if b.starts_with('/') || a.is_empty() {
        b.to_string()
    } else if a.ends_with('/') {
        format!("{a}{b}")
    } else {
        format!("{a}/{b}")
    }
}

impl PyPath {
    /// `Path(text)`.
    pub fn new(text: &str) -> Self {
        if text.is_empty() {
            return PyPath {
                drv: String::new(),
                root: String::new(),
                tail: Vec::new(),
            };
        }
        #[cfg(windows)]
        let text = &text.replace('/', "\\");
        let (drv, root, rel) = splitroot(text);
        let sep = SEP.chars().next().unwrap_or('/');
        #[cfg(windows)]
        let root = if root.is_empty() && drv.starts_with(sep) && !drv.ends_with(sep) && {
            let drv_parts: Vec<&str> = drv.split(sep).collect();
            drv_parts.len() == 4 && !"?.".contains(drv_parts[2]) || drv_parts.len() == 6
        } {
            SEP
        } else {
            root
        };
        PyPath {
            drv: drv.to_string(),
            root: root.to_string(),
            tail: rel
                .split(sep)
                .filter(|part| !part.is_empty() && *part != ".")
                .map(str::to_string)
                .collect(),
        }
    }

    /// `str(path)`.
    pub fn display(&self) -> String {
        if !self.drv.is_empty() || !self.root.is_empty() {
            return format!("{}{}{}", self.drv, self.root, self.tail.join(SEP));
        }
        #[cfg(windows)]
        if let Some(first) = self.tail.first()
            && !splitroot(first).0.is_empty()
        {
            return format!(".{SEP}{}", self.tail.join(SEP));
        }
        if self.tail.is_empty() {
            return ".".into();
        }
        self.tail.join(SEP)
    }

    /// The path to hand to the operating system.
    pub fn fs(&self) -> PathBuf {
        PathBuf::from(self.display())
    }

    /// `self / other`.
    pub fn join(&self, other: &str) -> Self {
        PyPath::new(&join(&self.display(), other))
    }

    pub fn parent(&self) -> Self {
        let mut out = self.clone();
        out.tail.pop();
        out
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn paths_print_like_pathlib() {
        assert_eq!(PyPath::new("").display(), ".");
        assert_eq!(PyPath::new("./a//b/.").display(), format!("a{SEP}b"));
        assert_eq!(PyPath::new("a/b/").display(), format!("a{SEP}b"));
        assert_eq!(PyPath::new("a").join("b").display(), format!("a{SEP}b"));
        // CPython 3.15's str(PurePosixPath(...)) and posixpath.join on each.
        #[cfg(not(windows))]
        {
            assert_eq!(PyPath::new("//a/b").display(), "//a/b");
            assert_eq!(PyPath::new("///a/b").display(), "/a/b");
            assert_eq!(PyPath::new("/").join("b").display(), "/b");
            assert_eq!(PyPath::new("a").join("/b").display(), "/b");
            assert_eq!(PyPath::new("/a/b").parent().display(), "/a");
            assert_eq!(join("", "b"), "b");
            assert_eq!(join("a/", "b"), "a/b");
        }
    }

    #[cfg(windows)]
    #[test]
    fn windows_paths_follow_ntpath() {
        assert_eq!(
            PyPath::new("//server/share/x").display(),
            "\\\\server\\share\\x"
        );
        assert_eq!(
            PyPath::new("//server/share").display(),
            "\\\\server\\share\\"
        );
        assert_eq!(PyPath::new("C:a/b").display(), "C:a\\b");
        assert_eq!(PyPath::new("./c:x").display(), ".\\c:x");
        assert_eq!(join("C:\\x", "\\y"), "C:\\y");
        assert_eq!(join("C:\\x", "D:y"), "D:y");
        assert_eq!(join("C:\\x", "c:y"), "c:\\x\\y");
    }

    #[cfg(windows)]
    #[test]
    fn unc_roots_with_a_character_across_byte_8() {
        // CPython 3.15's ntpath.splitroot on each.
        for (path, drive, root, tail) in [
            (
                "\\\\abcd\u{e9}\\share\\file",
                "\\\\abcd\u{e9}\\share",
                "\\",
                "file",
            ),
            (
                "\\\\abcde\u{e9}\\share\\file",
                "\\\\abcde\u{e9}\\share",
                "\\",
                "file",
            ),
            (
                "\\\\abcdef\u{e9}\\share\\file",
                "\\\\abcdef\u{e9}\\share",
                "\\",
                "file",
            ),
            (
                "\\\\?\\UNC\\srv\\share\\x",
                "\\\\?\\UNC\\srv\\share",
                "\\",
                "x",
            ),
            (
                "\\\\?\\unc\\\u{e9}srv\\share",
                "\\\\?\\unc\\\u{e9}srv\\share",
                "",
                "",
            ),
            ("//?/UNC/srv/share/x", "//?/UNC/srv/share", "/", "x"),
        ] {
            assert_eq!(splitroot(path), (drive, root, tail), "{path}");
        }
    }
}
