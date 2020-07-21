use soup::{NodeExt, QueryBuilderExt, Soup};
use std::path::Path;
use std::str::FromStr;
use structopt::StructOpt;
use thiserror::Error;

#[derive(StructOpt, Debug)]
#[structopt(about = "Crawl a git-forge")]
pub struct CrawlForgeOpt {
    /// URL of the forge to crawl
    pub url: url::Url,

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

    /// When a URL CannotBeABase
    #[error("URL ParseError, {0}")]
    UrlCannotBeABase(url::Url),

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

/// Returns the base_url. Copied from the Rust cookbook, although I added the err_url.
/// https://rust-lang-nursery.github.io/rust-cookbook/web/url.html
fn base_url(mut url: url::Url) -> Result<url::Url, CrawlForgeError> {
    let err_url = url.clone();
    match url.path_segments_mut() {
        Ok(mut path) => {
            path.clear();
        }
        Err(_) => {
            return Err(CrawlForgeError::UrlCannotBeABase(err_url));
        }
    }

    url.set_query(None);

    Ok(url)
}

/// Returns the git forge's base URL for raw files
///
/// # Example
/// ```
/// # use crawlforge::{forge_base_url_raw, ForgeKind};
/// # use url::Url;
/// let gh_url = Url::parse("https://github.com/tompreston/sup/blob/master/README.md").unwrap();
/// let gh_base_url_raw = Url::parse("https://raw.githubusercontent.com").unwrap();
/// assert_eq!(forge_base_url_raw(ForgeKind::GitHub, &gh_url).ok(), Some(gh_base_url_raw));
///
/// let og_url = Url::parse("http://10.0.0.1/xref/foo/bar").unwrap();
/// let og_base_url_raw = Url::parse("http://10.0.0.1/").unwrap();
/// assert_eq!(forge_base_url_raw(ForgeKind::OpenGrok, &og_url).ok(), Some(og_base_url_raw));
/// ```
pub fn forge_base_url_raw(forge: ForgeKind, u: &url::Url) -> Result<url::Url, CrawlForgeError> {
    let base_url_raw = match forge {
        ForgeKind::GitHub => url::Url::parse("https://raw.githubusercontent.com")
            .expect("static URL should be correct"),
        ForgeKind::OpenGrok => base_url(u.clone())?,
    };
    Ok(base_url_raw)
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
    root: &url::Url,
    body: &str,
) -> Result<Vec<String>, CrawlForgeError> {
    let root_mod = match url_kind {
        UrlKind::RawFile => root.path().replace("xref", "raw"),
        _ => root.path().to_string(),
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
            Path::new(root_mod.as_str())
                .join(href)
                .to_string_lossy()
                .into_owned()
        })
        .collect();

    Ok(urls)
}

/// Returns a list of directory links
pub fn parse_forge(
    forge_kind: ForgeKind,
    url_kind: UrlKind,
    root: &url::Url,
    body: &str,
) -> Result<Vec<String>, CrawlForgeError> {
    match forge_kind {
        ForgeKind::GitHub => parse_github(url_kind, body),
        ForgeKind::OpenGrok => parse_opengrok(url_kind, root, body),
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
        let root = url::Url::parse("http://10.0.0.1:8080/xref/AGL/metalayers/").unwrap();
        let urls = parse_opengrok(UrlKind::Directory, &root, BODY_OPENGROK).unwrap();
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
        let root = url::Url::parse("http://10.0.0.1:8080/xref/AGL/metalayers/").unwrap();
        let urls = parse_opengrok(UrlKind::File, &root, BODY_OPENGROK).unwrap();
        assert_eq!(urls[0], "/xref/AGL/metalayers/foofile");
    }

    #[test]
    fn test_parse_raw_files_opengrok() {
        let root = url::Url::parse("http://10.0.0.1:8080/xref/AGL/metalayers/").unwrap();
        let urls = parse_opengrok(UrlKind::RawFile, &root, BODY_OPENGROK).unwrap();
        assert_eq!(urls[0], "/raw/AGL/metalayers/foofile");
    }

    #[test]
    fn test_url_username_password() {
        let u =
            url::Url::parse("http://foo1.bar:testpass@10.0.0.1:8080/xref/AGL/metalayers/").unwrap();
        assert_eq!(u.username(), "foo1.bar");
        assert_eq!(u.password(), Some("testpass"));
    }

    // Remember, '@' is %40 and '#' is %23 in URLs
    #[test]
    fn test_url_username_password_encoded() {
        let u =
            url::Url::parse("http://foo1.bar:test%40%23pass@10.0.0.1:8080/xref/AGL/metalayers/")
                .unwrap();
        assert_eq!(u.username(), "foo1.bar");
        assert_eq!(u.password(), Some("test%40%23pass"));
    }
}
