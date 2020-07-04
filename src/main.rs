use crawlforge::{parse_forge, CrawlForgeError, CrawlForgeOpt, ForgeKind, UrlKind};
use reqwest;
use structopt::StructOpt;
use url;

fn crawl_forge_dir(forge: ForgeKind, url: &str) -> Result<(), CrawlForgeError> {
    let url_base = url::Url::parse(url).map_err(CrawlForgeError::UrlParse)?;
    println!("{:?}", url_base);
    let body = reqwest::blocking::get(url_base)
        .map_err(CrawlForgeError::Reqwest)?
        .text()
        .map_err(CrawlForgeError::Reqwest)?;
    //println!("body = {:?}", body);

    let dirs = parse_forge(forge, UrlKind::Directory, body.as_str())?;
    println!("{:?}", dirs);

    Ok(())
}

fn main() {
    let opt = CrawlForgeOpt::from_args();
    println!("{:?}", opt);
    std::process::exit(match crawl_forge_dir(opt.forge, opt.url.as_str()) {
        Ok(_) => 0,
        Err(err) => {
            eprintln!("crawlforge: error: {}", err);
            1
        }
    });
}
