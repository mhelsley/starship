use indexmap::{indexmap, IndexMap};
use serde::{Deserialize, Serialize};
use url::Url;
use urlencoding::decode;

#[derive(Clone, Deserialize, Serialize)]
#[cfg_attr(
    feature = "config-schema",
    derive(schemars::JsonSchema),
    schemars(deny_unknown_fields)
)]
#[serde(default)]
pub struct GitBranchConfig<'a> {
    pub format: &'a str,
    pub symbol: &'a str,
    pub style: &'a str,
    pub truncation_length: i64,
    pub truncation_symbol: &'a str,
    pub only_attached: bool,
    pub always_show_remote: bool,
    pub ignore_branches: Vec<&'a str>,
    pub disabled: bool,
    pub remote_symbols: IndexMap<String, &'a str>,
}

impl<'a> GitBranchConfig<'a> {
    pub fn get_remote_symbol(&self, url: &str) -> Option<&'a str> {
        let url = decode(url).ok()?;
        let url = Url::parse(&url).ok()?;
        let mut host: &str = url.host_str()?;

        match self.remote_symbols.get(host) {
            None => {}
            x => {
                return x.copied();
            }
        }

        // Make sure we always start with . to signify matching
        // a domain name rather than hostname.
        if !host.starts_with('.') {
            host = &host[host.find('.')?..];
        }

        while !host.is_empty() && host.contains(".") {
            match self.remote_symbols.get(host) {
                None => {}
                found_symbol => {
                    return found_symbol.copied();
                }
            }

            // Skip the first .
            let mut next_start: usize = 1;
            while !host.is_char_boundary(next_start) {
                next_start += 1;
            }
            host = &host[next_start..];

            // The next substring starting with '.'
            next_start = host.find('.')?;
            host = &host[next_start..];
        }
        None
    }
}

impl<'a> Default for GitBranchConfig<'a> {
    fn default() -> Self {
        GitBranchConfig {
            format: "on [$symbol$branch(:$remote_branch)]($style) ",
            symbol: " ",
            style: "bold purple",
            truncation_length: std::i64::MAX,
            truncation_symbol: "…",
            only_attached: false,
            always_show_remote: false,
            ignore_branches: vec![],
            disabled: false,
            remote_symbols: indexmap! {
                // unicode from: https://www.nerdfonts.com/cheat-sheet
                String::from("bitbucket.org") => "\u{f171}", // devicon: \xe6\x03
                String::from("github.com") => "\u{f09b}", // devicon: \xe6\x09
                String::from("gitlab.com") => "\u{f296}",
                String::from("gitlab.gnome.org") => "\u{f02ac}",
                //String::from("git.code.sf.net") => "",
                String::from(".googlesource.com") => "\u{f1a0}",
                String::from("kernel.org") => "\u{f17c}",
                String::from("launchpad.net") => "\u{f13c}", // \u{f7df}
                // String::from("savannah.gnu.org") => "\xe7\x79", // devicon: \xe6\x79
            },
        }
    }
}
