use crawlforge::{forge_url_raw, parse_forge, CrawlForgeError, CrawlForgeOpt, ForgeKind, UrlKind};
use reqwest;
use structopt::StructOpt;
use url::Url;

fn crawl_forge_dir(forge: ForgeKind, url: &str) -> Result<(), CrawlForgeError> {
    let url_base = Url::parse(url).map_err(CrawlForgeError::UrlParse)?;
    let body = reqwest::blocking::get(url_base.clone())
        .map_err(CrawlForgeError::Reqwest)?
        .text()
        .map_err(CrawlForgeError::Reqwest)?;

    // Print the files
    let url_raw = Url::parse(forge_url_raw(forge)).map_err(CrawlForgeError::UrlParse)?;
    parse_forge(forge, UrlKind::RawFile, body.as_str())?
        .iter()
        .filter_map(|raw_file_url| url_raw.join(raw_file_url).ok())
        .for_each(|url| println!("{}", url));

    // Recurse into dirs
    let dir_urls: Vec<_> = parse_forge(forge, UrlKind::Directory, body.as_str())?
        .iter()
        .filter_map(|dir_url| url_base.join(dir_url).ok())
        .collect();
    for d in dir_urls {
        crawl_forge_dir(forge, d.as_str())?;
    }
    Ok(())
}

fn main() {
    let opt = CrawlForgeOpt::from_args();
    std::process::exit(match crawl_forge_dir(opt.forge, opt.url.as_str()) {
        Ok(_) => 0,
        Err(err) => {
            eprintln!("crawlforge: error: {}", err);
            1
        }
    });
}
