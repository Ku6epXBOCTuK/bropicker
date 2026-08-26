use std::cell::RefCell;
use std::rc::Rc;

use crate::config::{self, BrowserEntry, Config};

pub fn split_flags(flags: &str) -> Vec<String> {
    let mut out = Vec::new();
    let mut cur = String::new();
    let mut in_quotes = false;
    for c in flags.chars() {
        match c {
            '"' => in_quotes = !in_quotes,
            ' ' if !in_quotes => {
                if !cur.is_empty() {
                    out.push(std::mem::take(&mut cur));
                }
            }
            _ => cur.push(c),
        }
    }
    if !cur.is_empty() {
        out.push(cur);
    }
    out
}

pub fn launch(entry: &BrowserEntry, url: &str) -> std::io::Result<()> {
    let mut cmd = std::process::Command::new(&entry.path);
    if !entry.flags.is_empty() {
        cmd.args(split_flags(&entry.flags));
    }
    cmd.arg(url).spawn().map(|_| ())
}

/// Launch `entry` with `url` and, when `remember` is set, store the
/// domain→browser pair in the config. Returns `false` on spawn failure
/// (the error is logged).
pub fn launch_and_remember(
    cfg: &Rc<RefCell<Config>>,
    entry: &BrowserEntry,
    url: &str,
    remember: bool,
) -> bool {
    config::log(&format!(
        "[bp] launching {:?} {:?} {}",
        entry.path, entry.flags, url
    ));
    match launch(entry, url) {
        Ok(_) => {
            if remember {
                let domain = config::domain_of(url);
                if !domain.is_empty() {
                    cfg.borrow_mut()
                        .remembered
                        .insert(domain, entry.name.clone());
                    config::save(&cfg.borrow());
                }
            }
            true
        }
        Err(e) => {
            config::log(&format!("[bp] launch failed: {e}"));
            false
        }
    }
}

#[cfg(test)]
mod tests {
    use super::split_flags;

    #[test]
    fn empty_flags() {
        assert!(split_flags("").is_empty());
        assert!(split_flags("   ").is_empty());
    }

    #[test]
    fn simple_flags() {
        assert_eq!(split_flags("-P Work"), vec!["-P", "Work"]);
    }

    #[test]
    fn quoted_spaces_stay_one_arg() {
        assert_eq!(
            split_flags(r#"--profile-directory="Profile 1""#),
            vec!["--profile-directory=Profile 1"]
        );
        assert_eq!(split_flags(r#"-P "My Work""#), vec!["-P", "My Work"]);
    }

    #[test]
    fn multiple_spaces_collapse() {
        assert_eq!(split_flags("  a   b  "), vec!["a", "b"]);
    }
}
