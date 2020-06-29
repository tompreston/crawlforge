use reqwest;
use soup::{NodeExt, QueryBuilderExt, Soup};
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
    pub forge: String,
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
    #[error("Unknown forge, {0}")]
    UnknownForge(String),

    #[error("Listing not found")]
    ListingNotFound(),
}

fn parse_dirs_github(body: &str) -> Result<Vec<&str>, CrawlForgeError> {
    let dir_urls: Vec<&str> = vec![];

    let s = Soup::new(body);
    let details = s
        .tag("div")
        .attr("class", "js-details-container")
        .find()
        .ok_or(CrawlForgeError::ListingNotFound())?;

    // TODO how do I query "details"? Do I have a create a new Soup instance?
    details
        .tag("div")
        .attr("class", "js-navigation-item")
        .find_all()
        .for_each(|row| println!("{:?}", row.display()));

    Ok(dir_urls)
}

/// Returns a list of directory links
pub fn parse_dirs<'a>(forge: &str, body: &'a str) -> Result<Vec<&'a str>, CrawlForgeError> {
    if forge == "github" {
        parse_dirs_github(body)
    } else {
        Err(CrawlForgeError::UnknownForge(forge.to_string()))
    }
}
