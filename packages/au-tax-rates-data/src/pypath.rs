//! `pathlib.Path` and `os.path` as CPython 3.15 parses, prints, joins and resolves paths.
//!
//! The radar prints its output paths and names its inputs in messages, so a
//! path is carried in pathlib's parsed form: drive, root and the non-empty,
//! non-`.` parts. Resolution follows `ntpath.realpath` on Windows (lexical
//! `..`, then the final path name of the longest existing prefix) and
//! `posixpath.realpath` elsewhere (symbolic links in order).
use std::path::PathBuf;

#[cfg(windows)]
pub const SEP: &str = "\\";
#[cfg(not(windows))]
pub const SEP: &str = "/";

/// The class `repr(path)` names.
#[cfg(windows)]
const CLASS: &str = "WindowsPath";
#[cfg(not(windows))]
const CLASS: &str = "PosixPath";

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

#[cfg(windows)]
fn is_sep(c: char) -> bool {
    c == '/' || c == '\\'
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

/// `os.path.split(p)`.
#[cfg(windows)]
pub fn split(p: &str) -> (String, String) {
    let (d, r, rest) = splitroot(p);
    let i = rest.rfind(is_sep).map_or(0, |i| i + 1);
    let head = rest[..i].trim_end_matches(is_sep);
    (format!("{d}{r}{head}"), rest[i..].to_string())
}

/// `os.path.isabs(p)`.
#[cfg(windows)]
pub fn isabs(p: &str) -> bool {
    let start: String = p.chars().take(3).collect::<String>().replace('/', "\\");
    start.get(1..).is_some_and(|s| s.starts_with(":\\")) || start.starts_with("\\\\")
}

/// `os.path.normpath(p)` (Windows).
#[cfg(windows)]
pub fn normpath(p: &str) -> String {
    let p = p.replace('/', "\\");
    let (drive, root, rest) = splitroot(&p);
    let mut comps: Vec<&str> = Vec::new();
    for comp in rest.split('\\') {
        if comp.is_empty() || comp == "." {
            continue;
        }
        if comp == ".." {
            if comps.last().is_some_and(|c| *c != "..") {
                comps.pop();
                continue;
            }
            if comps.is_empty() && !root.is_empty() {
                continue;
            }
        }
        comps.push(comp);
    }
    if drive.is_empty() && root.is_empty() && comps.is_empty() {
        return ".".into();
    }
    format!("{drive}{root}{}", comps.join("\\"))
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
        // A UNC drive without its root gets one; a POSIX drive is always empty.
        #[cfg(windows)]
        let mut root = root;
        #[cfg(windows)]
        if root.is_empty() && drv.starts_with(sep) && !drv.ends_with(sep) {
            let drv_parts: Vec<&str> = drv.split(sep).collect();
            if drv_parts.len() == 4 && !"?.".contains(drv_parts[2]) || drv_parts.len() == 6 {
                root = SEP;
            }
        }
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

    pub fn name(&self) -> &str {
        self.tail.last().map_or("", String::as_str)
    }

    pub fn parent(&self) -> Self {
        let mut out = self.clone();
        out.tail.pop();
        out
    }

    /// `path.with_suffix(suffix)` for a suffix that starts with a dot.
    pub fn with_suffix(&self, suffix: &str) -> Result<Self, String> {
        let name = self.name();
        let stem = match name.rfind('.') {
            Some(i) if !name[..i].trim_start_matches('.').is_empty() => &name[..i],
            _ => name,
        };
        if stem.is_empty() {
            return Err(format!("{} has an empty name", self.repr()));
        }
        let mut out = self.clone();
        if let Some(last) = out.tail.last_mut() {
            *last = format!("{stem}{suffix}");
        }
        Ok(out)
    }

    /// `repr(path)`.
    pub fn repr(&self) -> String {
        format!("{CLASS}({})", crate::py::repr_str(&self.display()))
    }

    /// `self == other`: case-blind on Windows.
    pub fn same(&self, other: &PyPath) -> bool {
        #[cfg(windows)]
        let key = |path: &PyPath| path.display().to_lowercase();
        #[cfg(not(windows))]
        let key = PyPath::display;
        key(self) == key(other)
    }

    /// `self.is_relative_to(other)`: `other` is this path or one of its parents.
    pub fn is_relative_to(&self, other: &PyPath) -> bool {
        let mut path = self.clone();
        loop {
            if path.same(other) {
                return true;
            }
            if path.tail.is_empty() {
                return false;
            }
            path.tail.pop();
        }
    }

    /// `self.relative_to(other)`, or the `ValueError` message.
    pub fn relative_to(&self, other: &PyPath) -> Result<Self, String> {
        if !self.is_relative_to(other) {
            return Err(format!(
                "{} is not in the subpath of {}",
                crate::py::repr_str(&self.display()),
                crate::py::repr_str(&other.display())
            ));
        }
        Ok(PyPath {
            drv: String::new(),
            root: String::new(),
            tail: self.tail[other.tail.len()..].to_vec(),
        })
    }

    /// `path.stem`: the name less its last suffix, unless that leaves only dots.
    pub fn stem(&self) -> &str {
        let name = self.name();
        match name.rfind('.') {
            Some(i) if !name[..i].trim_start_matches('.').is_empty() => &name[..i],
            _ => name,
        }
    }

    /// `path.resolve()`.
    pub fn resolve(&self) -> Result<Self, crate::pyos::OsError> {
        realpath(&self.display()).map(|text| PyPath::new(&text))
    }
}

#[cfg(windows)]
fn final_path_name(path: &str) -> Result<String, std::io::Error> {
    std::fs::canonicalize(path).map(|p| p.to_string_lossy().into_owned())
}

#[cfg(windows)]
fn winerror(error: &std::io::Error) -> i32 {
    error.raw_os_error().unwrap_or(0)
}

/// Restore the verbatim prefix CPython's `_nt_readlink` keeps on drive-rooted
/// and UNC reparse targets after Rust's `std::fs::read_link` strips `\??\`.
#[cfg(windows)]
fn restore_verbatim_prefix(path: String) -> String {
    if path.len() >= 3
        && path.as_bytes()[0].is_ascii_alphabetic()
        && path.as_bytes()[1] == b':'
        && path.as_bytes()[2] == b'\\'
    {
        format!("\\\\?\\{path}")
    } else if let Some(unc) = path.strip_prefix("\\\\")
        && !unc.is_empty()
        && !unc.starts_with(['?', '.', '\\'])
    {
        format!("\\\\?\\UNC\\{unc}")
    } else {
        path
    }
}

/// `ntpath._readlink_deep`, enough for broken symbolic links and junctions.
#[cfg(windows)]
fn readlink_deep(path: &str) -> String {
    readlink_deep_with(
        path,
        |p| std::fs::read_link(p).ok(),
        |p| std::fs::symlink_metadata(p).is_ok_and(|m| m.file_type().is_symlink()),
    )
}

#[cfg(windows)]
fn readlink_deep_with(
    path: &str,
    mut read_link: impl FnMut(&str) -> Option<PathBuf>,
    mut is_symlink: impl FnMut(&str) -> bool,
) -> String {
    let mut path = path.to_string();
    let mut seen = std::collections::HashSet::new();
    while seen.insert(path.to_lowercase()) {
        let old = path.clone();
        let Some(target) = read_link(&path) else {
            break;
        };
        path = restore_verbatim_prefix(target.to_string_lossy().into_owned());
        if !isabs(&path) {
            if !is_symlink(&old) {
                path = old;
                break;
            }
            path = normpath(&join(&split(&old).0, &path));
        }
    }
    path
}

/// `ntpath._getfinalpathname_nonstrict`.
///
/// shortcut: CPython asks `FindFirstFileW` for the stored name of a component
/// it may not open (access denied, sharing violation); this keeps the name as
/// given there. Only the letter case of such a component can differ.
/// Ask for the stored name too if a case mismatch is reported for such a path.
#[cfg(windows)]
fn final_path_name_nonstrict(path: &str) -> Result<String, crate::pyos::OsError> {
    const ALLOWED: &[i32] = &[
        1, 2, 3, 5, 21, 32, 50, 53, 65, 67, 87, 123, 161, 1005, 1920, 1921,
    ];
    let mut path = path.to_string();
    let mut tail = String::new();
    while !path.is_empty() {
        match final_path_name(&path) {
            Ok(found) => {
                return Ok(if tail.is_empty() {
                    found
                } else {
                    join(&found, &tail)
                });
            }
            Err(error) => {
                if !ALLOWED.contains(&winerror(&error)) {
                    return Err(crate::pyos::OsError::new(
                        &error,
                        Some(&path),
                        crate::pyos::Style::Native,
                    ));
                }
                let new_path = readlink_deep(&path);
                if new_path != path {
                    return Ok(if tail.is_empty() {
                        new_path
                    } else {
                        join(&new_path, &tail)
                    });
                }
                let (head, name) = split(&path);
                path = head;
                if !path.is_empty() && name.is_empty() {
                    return Ok(format!("{path}{tail}"));
                }
                tail = if tail.is_empty() {
                    name
                } else {
                    join(&name, &tail)
                };
            }
        }
    }
    Ok(tail)
}

/// `os.path.realpath(path)`.
#[cfg(windows)]
pub fn realpath(path: &str) -> Result<String, crate::pyos::OsError> {
    const PREFIX: &str = "\\\\?\\";
    const UNC_PREFIX: &str = "\\\\?\\UNC\\";
    let mut path = normpath(path);
    if path.to_lowercase() == "nul" {
        return Ok("\\\\.\\NUL".into());
    }
    let had_prefix = path.starts_with(PREFIX);
    if !had_prefix && !isabs(&path) {
        let cwd = std::env::current_dir()
            .map_err(|error| crate::pyos::OsError::new(&error, None, crate::pyos::Style::Native))?;
        path = join(&cwd.to_string_lossy(), &path);
    }
    let initial_winerror;
    match final_path_name(&path) {
        Ok(found) => {
            path = found;
            initial_winerror = 0;
        }
        Err(error) => {
            initial_winerror = winerror(&error);
            path = final_path_name_nonstrict(&path)?;
        }
    }
    if !had_prefix && path.starts_with(PREFIX) {
        let spath = match path.strip_prefix(UNC_PREFIX) {
            Some(rest) => format!("\\\\{rest}"),
            None => path[PREFIX.len()..].to_string(),
        };
        match final_path_name(&spath) {
            Ok(found) if found == path => path = spath,
            Ok(_) => {}
            Err(error) if winerror(&error) == initial_winerror => path = spath,
            Err(_) => {}
        }
    }
    Ok(path)
}

/// `os.path.realpath(path)`.
#[cfg(not(windows))]
pub fn realpath(filename: &str) -> Result<String, crate::pyos::OsError> {
    realpath_from(filename, std::env::current_dir)
}

/// `realpath` with `os.getcwd` passed in, so a failing one can be tested.
#[cfg(not(windows))]
fn realpath_from(
    filename: &str,
    getcwd: fn() -> std::io::Result<PathBuf>,
) -> Result<String, crate::pyos::OsError> {
    let mut rest: Vec<Option<String>> = Vec::new();
    let mut part_count = push_parts(&mut rest, filename.trim_end_matches('/'));
    let mut path = realpath_start(filename, getcwd)?;
    let mut seen: std::collections::HashMap<String, Option<String>> = Default::default();
    while part_count > 0 {
        let Some(entry) = rest.pop() else { break };
        let Some(name) = entry else {
            if let Some(Some(link)) = rest.pop() {
                seen.insert(link, Some(path.clone()));
            }
            continue;
        };
        part_count -= 1;
        if name.is_empty() || name == "." {
            continue;
        }
        if name == ".." {
            path = match path.rfind('/') {
                Some(0) | None => "/".into(),
                Some(i) => path[..i].to_string(),
            };
            continue;
        }
        let newpath = if path == "/" {
            format!("/{name}")
        } else {
            format!("{path}/{name}")
        };
        let target = match link_step(&newpath, &seen) {
            Ok(target) => target,
            Err(next) => {
                path = next;
                continue;
            }
        };
        if target.starts_with('/') {
            path = "/".into();
        }
        seen.insert(newpath.clone(), None);
        rest.push(Some(newpath));
        rest.push(None);
        part_count += push_parts(&mut rest, &target);
    }
    Ok(path)
}

/// Pushes `path`'s `/`-separated names so the first pops first; returns
/// how many.
#[cfg(not(windows))]
fn push_parts(rest: &mut Vec<Option<String>>, path: &str) -> usize {
    let before = rest.len();
    rest.extend(path.split('/').rev().map(|s| Some(s.to_string())));
    rest.len() - before
}

/// Where `realpath` starts: `/` for an absolute name, else `os.getcwd()`.
#[cfg(not(windows))]
fn realpath_start(
    filename: &str,
    getcwd: fn() -> std::io::Result<PathBuf>,
) -> Result<String, crate::pyos::OsError> {
    if filename.starts_with('/') {
        return Ok("/".to_string());
    }
    let cwd = getcwd()
        .map_err(|error| crate::pyos::OsError::new(&error, None, crate::pyos::Style::Native))?;
    Ok(cwd.to_string_lossy().into_owned())
}

/// What `posixpath.realpath` does at `newpath`, in its order: lstat, the
/// cache of links already met, then readlink. `Ok` is the link's target to
/// follow; `Err` is the path to go on from: `newpath` itself where lstat or
/// readlink fails (one `except OSError`) or the name is not a link, else the
/// cached resolution (`newpath` for a link still being resolved, a loop).
#[cfg(not(windows))]
fn link_step(
    newpath: &str,
    seen: &std::collections::HashMap<String, Option<String>>,
) -> Result<String, String> {
    let stay = newpath.to_string();
    if !std::fs::symlink_metadata(newpath).is_ok_and(|meta| meta.file_type().is_symlink()) {
        return Err(stay);
    }
    if let Some(cached) = seen.get(newpath) {
        return Err(cached.clone().unwrap_or(stay));
    }
    let target = std::fs::read_link(newpath).ok();
    target
        .map(|target| target.to_string_lossy().into_owned())
        .ok_or(stay)
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
        assert_eq!(
            PyPath::new("a/b.json")
                .with_suffix(".md")
                .unwrap()
                .display(),
            format!("a{SEP}b.md")
        );
        assert_eq!(
            PyPath::new(".json").with_suffix(".md").unwrap().display(),
            ".json.md"
        );
        assert_eq!(
            PyPath::new("a.").with_suffix(".md").unwrap().display(),
            "a.md"
        );
        assert!(PyPath::new("/").with_suffix(".md").is_err());
        assert!(PyPath::new("a/b/c").is_relative_to(&PyPath::new("a")));
        assert!(!PyPath::new("ab/c").is_relative_to(&PyPath::new("a")));
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
        assert_eq!(normpath("C:a\\..\\..\\b"), "C:..\\b");
        assert_eq!(normpath("\\\\server\\share\\.."), "\\\\server\\share\\");
        assert!(PyPath::new("A\\B").same(&PyPath::new("a/b")));
        assert_eq!(realpath("nul").unwrap(), "\\\\.\\NUL");
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

    /// What CPython 3.15's `posixpath` and `pathlib` give for each.
    #[cfg(not(windows))]
    #[test]
    fn posix_paths_follow_posixpath() {
        assert_eq!(splitroot("//a"), ("", "//", "a"));
        assert_eq!(splitroot("///a"), ("", "/", "//a"));
        assert_eq!(join("", "b"), "b");
        assert_eq!(join("a/", "b"), "a/b");
        assert_eq!(join("a", "/b"), "/b");
        assert_eq!(PyPath::new("a/b.tar.gz").stem(), "b.tar");
        assert_eq!(PyPath::new(".bashrc").stem(), ".bashrc");
        assert_eq!(PyPath::new("a.").stem(), "a");
        assert_eq!(PyPath::new("a/b").repr(), "PosixPath('a/b')");
        assert!(!PyPath::new("A").same(&PyPath::new("a")));
        assert_eq!(
            PyPath::new("a/b")
                .relative_to(&PyPath::new("c"))
                .unwrap_err(),
            "'a/b' is not in the subpath of 'c'"
        );
        assert_eq!(realpath("/..").unwrap(), "/");
        let cwd = std::fs::canonicalize(".").unwrap();
        assert_eq!(
            realpath("nope-x").unwrap(),
            format!("{}/nope-x", cwd.display())
        );
    }

    /// `os.path.realpath` over a tree of links, each answer CPython 3.15 gave
    /// for the same tree (`d` a folder, `f` a file, links as named).
    #[cfg(unix)]
    #[test]
    fn realpath_follows_links_as_posixpath_does() {
        use std::os::unix::fs::symlink;
        let base = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("target")
            .join(format!("pypath-realpath-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&base);
        std::fs::create_dir_all(base.join("d")).unwrap();
        let t = std::fs::canonicalize(&base)
            .unwrap()
            .to_string_lossy()
            .into_owned();
        std::fs::write(base.join("f"), "").unwrap();
        for (link, target) in [
            ("l1", "d".to_string()),
            ("l2", format!("{t}/d")),
            ("loop1", "loop2".into()),
            ("loop2", "loop1".into()),
            ("dang", "nope".into()),
            ("top", "/".into()),
            ("chain", "l1/../f".into()),
        ] {
            symlink(target, base.join(link)).unwrap();
        }
        for (input, want) in [
            ("/d/./x/../f", "/d/f"),
            ("/l1", "/d"),
            ("/l1/", "/d"),
            ("/l2/f2", "/d/f2"),
            ("/loop1", "/loop1"),
            ("/loop1/x", "/loop1/x"),
            ("/l1/../f", "/f"),
            ("/l1/../l1", "/d"),
            ("/dang", "/nope"),
            ("/chain", "/f"),
            ("//d", "/d"),
            ("/f/../d", "/d"),
        ] {
            assert_eq!(
                realpath(&format!("{t}{input}")).unwrap(),
                format!("{t}{want}"),
                "{input}"
            );
        }
        assert_eq!(realpath(&format!("{t}/top")).unwrap(), "/");
        std::fs::remove_dir_all(&base).unwrap();
    }

    /// A deleted working folder: `os.getcwd()` raises, and so does realpath.
    #[cfg(unix)]
    #[test]
    fn realpath_raises_when_the_working_folder_is_gone() {
        let gone = realpath_from("x", || Err(std::io::Error::from_raw_os_error(2))).unwrap_err();
        assert_eq!(
            (gone.class(), gone.text.as_str()),
            ("FileNotFoundError", "[Errno 2] No such file or directory")
        );
    }
}

#[cfg(all(test, windows))]
#[path = "cov_a7_pypath.rs"]
mod cov_a7_tests;
