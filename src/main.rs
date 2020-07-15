use crawlforge::{
    forge_base_url_raw, parse_forge, CrawlForgeError, CrawlForgeOpt, ForgeKind, UrlKind,
};
use structopt::StructOpt;

fn crawl_forge_dir(forge: ForgeKind, root: url::Url) -> Result<(), CrawlForgeError> {
    let body = reqwest::blocking::get(root.clone())
        .map_err(CrawlForgeError::Reqwest)?
        .text()
        .map_err(CrawlForgeError::Reqwest)?;

    // Print the files
    let base_url_raw = crate::forge_base_url_raw(forge, &root)?;
    parse_forge(forge, UrlKind::RawFile, body.as_str())?
        .iter()
        .filter_map(|raw_file_url| base_url_raw.join(raw_file_url).ok())
        .for_each(|url| println!("{}", url));

    // Recurse into dirs
    let dir_urls: Vec<_> = parse_forge(forge, UrlKind::Directory, body.as_str())?
        .iter()
        .filter_map(|dir_url| root.join(dir_url).ok())
        .collect();
    for d in dir_urls {
        crawl_forge_dir(forge, d)?;
    }
    Ok(())
}

fn main() {
    let opt = CrawlForgeOpt::from_args();
    std::process::exit(match crawl_forge_dir(opt.forge, opt.url) {
        Ok(_) => 0,
        Err(err) => {
            eprintln!("crawlforge: error: {}", err);
            1
        }
    });
}
