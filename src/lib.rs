use soup::{NodeExt, QueryBuilderExt, Soup};
use std::str::FromStr;
use structopt::StructOpt;
use thiserror::Error;
use url;

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
    #[error("URL ParseError, {0}")]
    UrlParse(url::ParseError),

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
    Github,
}

impl FromStr for ForgeKind {
    type Err = CrawlForgeError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s == "github" {
            Ok(ForgeKind::Github)
        } else {
            Err(CrawlForgeError::ParseForgeError(s.to_string()))
        }
    }
}

pub enum UrlKind {
    Directory,
    File,
    RawFile,
}

/// Returns the git forge's raw file URL
///
/// # Example
/// ```
/// # use crawlforge::{forge_url_raw, ForgeKind};
/// assert_eq!(forge_url_raw(ForgeKind::Github), "https://raw.githubusercontent.com");
/// ```
pub fn forge_url_raw(forge: ForgeKind) -> &'static str {
    match forge {
        ForgeKind::Github => "https://raw.githubusercontent.com",
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
        .ok_or(CrawlForgeError::ListingNotFound(rcontent_class.to_string()))?;

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

/// Returns a list of directory links
pub fn parse_forge<'a>(
    forge_kind: ForgeKind,
    url_kind: UrlKind,
    body: &'a str,
) -> Result<Vec<String>, CrawlForgeError> {
    match forge_kind {
        ForgeKind::Github => parse_github(url_kind, body),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    mod data;
    use data::BODY_GITHUB;

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
}
