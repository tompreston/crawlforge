use crawlforge::{
    parse_forge, raw_file_base_url, CrawlForgeError, CrawlForgeOpt, ForgeKind, UrlKind,
};
use structopt::StructOpt;
use url::Url;

fn crawl_forge_dir(forge: ForgeKind, url: &str) -> Result<(), CrawlForgeError> {
    let url_base = Url::parse(url).map_err(|e| CrawlForgeError::UrlParse(e, url.to_string()))?;
    let body = reqwest::blocking::get(url_base.clone())
        .map_err(CrawlForgeError::Reqwest)?
        .text()
        .map_err(CrawlForgeError::Reqwest)?;

    // Print the files
    let url_base_raw = raw_file_base_url(forge, url)?;
    let url_raw = Url::parse(url_base_raw.as_str())
        .map_err(|e| CrawlForgeError::UrlParse(e, url_base_raw.to_string()))?;
    parse_forge(forge, UrlKind::RawFile, url, body.as_str())?
        .iter()
        .filter_map(|raw_file_base_url| url_raw.join(raw_file_base_url).ok())
        .for_each(|url| println!("{}", url));

    // Recurse into dirs
    let dir_urls: Vec<_> = parse_forge(forge, UrlKind::Directory, url, body.as_str())?
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
