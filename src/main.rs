use reqwest;
use soup::Soup;
use std::io;
use structopt::StructOpt;
use thiserror::Error;
use url;

#[derive(StructOpt, Debug)]
#[structopt(about = "Crawl a git-forge")]
struct CrawlForgeOpt {
    /// URL of the forge to crawl
    url: String,

    /// Type of git forge
    #[structopt(short, long, env, default_value = "github")]
    forge: String,
}

/// The errors which can happen when crawling a git forge
#[derive(Error, Debug)]
enum CrawlForgeError {
    /// When an IO error occurrs
    #[error("IO error, {0}")]
    IO(io::Error),

    /// When a reqwest error occurrs
    #[error("Reqwest error, {0}")]
    ReqwestError(reqwest::Error),

    /// When a URL ParseError occurs
    #[error("URL ParseError, {0}")]
    UrlParseError(url::ParseError),
}

fn crawl_forge_dir(url: &str) -> Result<(), CrawlForgeError> {
    let url_base = url::Url::parse(url).map_err(CrawlForgeError::UrlParseError)?;
    println!("{:?}", url_base);
    let body = reqwest::blocking::get(url_base)
        .map_err(CrawlForgeError::ReqwestError)?
        .text()
        .map_err(CrawlForgeError::ReqwestError)?;
    //println!("body = {:?}", body);

    // parse_dirs
    let s = Soup::new(body);
    let details = s
        .tag("div")
        .attr("class", "js-details-container")
        .find()
        .expect("Couldn't find tag 'div js-details-container'");
    println!(details);

    Ok(())
}

fn main() {
    let opt = CrawlForgeOpt::from_args();
    println!("{:?}", opt);
    std::process::exit(match crawl_forge_dir(opt.url.as_str()) {
        Ok(_) => 0,
        Err(err) => {
            eprintln!("crawlforge: error: {}", err);
            1
        }
    });
}
