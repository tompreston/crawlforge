use soup::{NodeExt, QueryBuilderExt, Soup};
use std::path::Path;
use std::str::FromStr;
use structopt::StructOpt;
use thiserror::Error;

#[derive(StructOpt, Debug)]
#[structopt(about = "Crawl a git-forge")]
pub struct CrawlForgeOpt {
    /// URL of the forge to crawl
    pub url: String,

    /// Type of git forge
    #[structopt(short, long, env, default_value = "github")]
    pub forge: ForgeKind,
}

/// The errors which can happen when crawling a git forge
#[derive(Error, Debug)]
pub enum CrawlForgeError {
    /// When a reqwest error occurrs
    #[error("Reqwest error, {0}")]
    Reqwest(reqwest::Error),

    /// When a URL ParseError occurs
    #[error("URL ParseError, {0}, {1}")]
    UrlParse(url::ParseError, String),

    /// Unknown forge
    #[error("Parse forge error, {0}")]
    ParseForgeError(String),

    #[error("Listing not found for class {0}")]
    ListingNotFound(String),

    #[error("Failed to get href from link element")]
    FailedToGetHref(String),
}

#[derive(Copy, Clone, Debug)]
pub enum ForgeKind {
    GitHub,
    OpenGrok,
}

impl FromStr for ForgeKind {
    type Err = CrawlForgeError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "github" => Ok(ForgeKind::GitHub),
            "opengrok" => Ok(ForgeKind::OpenGrok),
            _ => Err(CrawlForgeError::ParseForgeError(s.to_string())),
        }
    }
}

pub enum UrlKind {
    Directory,
    File,
    RawFile,
}

/// Returns the git forge's raw file URL. This could just be the normal URL, a
/// modification of it, or a completely new one.
///
/// # Example
/// ```
/// # use crawlforge::{raw_file_base_url, ForgeKind};
/// assert_eq!(
///     raw_file_base_url(ForgeKind::GitHub, "https://github.com"),
///     "https://raw.githubusercontent.com"
/// );
/// assert_eq!(
///     raw_file_base_url(ForgeKind::OpenGrok, "http://10.0.0.1:8080"),
///     "http://10.0.0.1:8080"
/// );
/// ```
pub fn raw_file_base_url(forge: ForgeKind, url_base: &str) -> &str {
    match forge {
        ForgeKind::GitHub => "https://raw.githubusercontent.com",
        ForgeKind::OpenGrok => url_base,
    }
}

/// Returns a list of Strings representing different UrlKinds
fn parse_github(url_kind: UrlKind, body: &str) -> Result<Vec<String>, CrawlForgeError> {
    let svg_class = match url_kind {
        UrlKind::Directory => "octicon-file-directory",
        UrlKind::File => "octicon-file",
        UrlKind::RawFile => "octicon-file",
    };

    // Find the div repository-content, which contains the main repo links
    let rcontent_class = "repository-content";
    let rcontent = Soup::new(body)
        .tag("div")
        .attr("class", rcontent_class)
        .find()
        .ok_or_else(|| CrawlForgeError::ListingNotFound(rcontent_class.to_string()))?;

    // Within repository-content, find js-navigation-item elements which contain
    // svg elements with the class marking it as "file" or "directory".
    let nav_items: Vec<_> = rcontent
        .children()
        .filter(|child| child.is_element())
        .map(|child| {
            child
                .tag("div")
                .attr("class", "js-navigation-item")
                .find_all()
        })
        .flatten()
        .filter(|nav_item| {
            nav_item
                .tag("svg")
                .attr("class", svg_class)
                .find()
                .is_some()
        })
        .collect();

    // Extract the urls from the nav_items
    let urls: Vec<String> = nav_items
        .iter()
        .filter_map(|n| n.tag("a").attr("class", "js-navigation-open").find())
        .filter_map(|a| a.get("href"))
        .map(|href| match url_kind {
            UrlKind::RawFile => href.replace("blob/", ""),
            _ => href,
        })
        .collect();

    Ok(urls)
}

/// Returns a list of Strings representing different UrlKinds
fn parse_opengrok(
    url_kind: UrlKind,
    url_path: &str,
    body: &str,
) -> Result<Vec<String>, CrawlForgeError> {
    let url_path_mod = match url_kind {
        UrlKind::RawFile => url_path.replace("xref", "raw"),
        _ => url_path.to_string(),
    };

    // Find the table body, which contains the main repo links
    let tbody_str = "tbody";
    let tbody = Soup::new(body)
        .tag(tbody_str)
        .find()
        .ok_or_else(|| CrawlForgeError::ListingNotFound(tbody_str.to_string()))?;

    // Now grab all the relative links in the second column.
    // Directories end with "/"
    let urls: Vec<_> = tbody
        .children()
        .filter_map(|row| row.tag("td").find_all().nth(1))
        .filter_map(|name_col| name_col.tag("a").find())
        .filter_map(|a| a.get("href"))
        .filter(|href| href != "..")
        .filter(|href| match url_kind {
            UrlKind::Directory => href.ends_with('/'),
            _ => !href.ends_with('/'),
        })
        .map(|href| {
            Path::new(url_path_mod.as_str())
                .join(href)
                .to_string_lossy()
                .into_owned()
        })
        .collect();

    Ok(urls)
}

/// Returns a list of directory links
pub fn parse_forge<'a>(
    forge_kind: ForgeKind,
    url_kind: UrlKind,
    url_path: &str,
    body: &'a str,
) -> Result<Vec<String>, CrawlForgeError> {
    match forge_kind {
        ForgeKind::GitHub => parse_github(url_kind, body),
        ForgeKind::OpenGrok => parse_opengrok(url_kind, url_path, body),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    mod data;
    use data::{BODY_GITHUB, BODY_OPENGROK};

    #[test]
    fn test_parse_dirs_github() {
        let urls = parse_github(UrlKind::Directory, BODY_GITHUB).unwrap();
        assert_eq!(urls[0], "/tompreston/sup/tree/master/.github/workflows");
        assert_eq!(urls[1], "/tompreston/sup/tree/master/src");
    }

    #[test]
    fn test_parse_files_github() {
        let urls = parse_github(UrlKind::File, BODY_GITHUB).unwrap();
        assert_eq!(urls[0], "/tompreston/sup/blob/master/.gitignore");
        assert_eq!(urls[1], "/tompreston/sup/blob/master/.travis.yml");
        assert_eq!(urls[2], "/tompreston/sup/blob/master/Cargo.lock");
        assert_eq!(urls[3], "/tompreston/sup/blob/master/Cargo.toml");
        assert_eq!(urls[4], "/tompreston/sup/blob/master/README.md");
    }

    #[test]
    fn test_parse_raw_files_github() {
        let urls = parse_github(UrlKind::RawFile, BODY_GITHUB).unwrap();
        assert_eq!(urls[0], "/tompreston/sup/master/.gitignore");
        assert_eq!(urls[1], "/tompreston/sup/master/.travis.yml");
        assert_eq!(urls[2], "/tompreston/sup/master/Cargo.lock");
        assert_eq!(urls[3], "/tompreston/sup/master/Cargo.toml");
        assert_eq!(urls[4], "/tompreston/sup/master/README.md");
    }

    #[test]
    fn test_parse_dirs_opengrok() {
        let urls =
            parse_opengrok(UrlKind::Directory, "/xref/AGL/metalayers/", BODY_OPENGROK).unwrap();
        assert_eq!(urls[0], "/xref/AGL/metalayers/meta-agl/");
        assert_eq!(urls[1], "/xref/AGL/metalayers/meta-agl-demo/");
        assert_eq!(urls[2], "/xref/AGL/metalayers/meta-agl-extra/");
        assert_eq!(urls[3], "/xref/AGL/metalayers/meta-amb/");
        assert_eq!(urls[4], "/xref/AGL/metalayers/meta-genivi-demo/");
        assert_eq!(urls[5], "/xref/AGL/metalayers/meta-intel-iot-security/");
        assert_eq!(urls[6], "/xref/AGL/metalayers/meta-ivi/");
        assert_eq!(urls[7], "/xref/AGL/metalayers/meta-openembedded/");
        assert_eq!(urls[8], "/xref/AGL/metalayers/meta-qt5/");
        assert_eq!(urls[9], "/xref/AGL/metalayers/meta-rust/");
        assert_eq!(urls[10], "/xref/AGL/metalayers/meta-security-isafw/");
        assert_eq!(urls[11], "/xref/AGL/metalayers/poky/");
    }

    #[test]
    fn test_parse_files_opengrok() {
        let urls = parse_opengrok(UrlKind::File, "/xref/AGL/metalayers/", BODY_OPENGROK).unwrap();
        assert_eq!(urls[0], "/xref/AGL/metalayers/foofile");
    }
}
