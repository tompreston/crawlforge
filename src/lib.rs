use soup::{NodeExt, QueryBuilderExt, Soup};
use std::str::FromStr;
//use std::io;
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
    ///// When an IO error occurrs
    //#[error("IO error, {0}")]
    //IO(io::Error),
    /// When a reqwest error occurrs
    #[error("Reqwest error, {0}")]
    Reqwest(reqwest::Error),

    /// When a URL ParseError occurs
    #[error("URL ParseError, {0}")]
    UrlParse(url::ParseError),

    /// Unknown forge
    #[error("Parse forge error, {0}")]
    ParseForgeError(String),

    #[error("Listing not found")]
    ListingNotFound(),

    #[error("Failed to get href from link element")]
    FailedToGetHref(String),
}

#[derive(Debug)]
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
}

fn parse_github(url_kind: UrlKind, body: &str) -> Result<Vec<String>, CrawlForgeError> {
    let svg_class = match url_kind {
        UrlKind::Directory => "octicon-file-directory",
        UrlKind::File => "octicon-file",
    };
    let mut urls: Vec<String> = vec![];

    let s = Soup::new(body);

    // Find the repository-content div
    let dcont = s
        .tag("div")
        .attr("class", "repository-content")
        .find()
        .ok_or(CrawlForgeError::ListingNotFound())?;

    // Within repository-content
    // find js-navigation-item elements which contain svg elements with the
    // given file or directory class
    // Inside there, find js-navigation-open links
    for child in dcont.children().filter(|child| child.is_element()) {
        let dir_items = child
            .tag("div")
            .attr("class", "js-navigation-item")
            .find_all()
            .filter(|item| item.tag("svg").attr("class", svg_class).find().is_some());
        for d in dir_items {
            d.tag("a")
                .attr("class", "js-navigation-open")
                .find_all()
                .for_each(|a| match a.get("href") {
                    Some(href) => urls.push(href),
                    None => urls.push("".to_string()),
                })
        }
    }

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
}
