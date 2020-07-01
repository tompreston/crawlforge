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

    #[error("Failed to get href from link element")]
    FailedToGetHref(String),
}

fn parse_dirs_github(body: &str) -> Result<Vec<&str>, CrawlForgeError> {
    let urls: Vec<&str> = vec![];

    let s = Soup::new(body);

    // Find the js-details-container
    let dcont = s
        .tag("div")
        .attr("class", "js-details-container")
        .find()
        .ok_or(CrawlForgeError::ListingNotFound())?;

    // Within js-details-container,
    // find js-navigation-item elements which contain svg elements with the
    // class "octicon-file-directory".
    // Inside there, find js-navigation-open links
    for child in dcont.children().filter(|child| child.is_element()) {
        let dir_items = child
            .tag("div")
            .attr("class", "js-navigation-item")
            .find_all()
            .filter(|item| {
                item.tag("svg")
                    .attr("class", "octicon-file-directory")
                    .find()
                    .is_some()
            });
        for d in dir_items {
            d.tag("a")
                .attr("class", "js-navigation-open")
                .find_all()
                .for_each(|a| match a.get("href") {
                    Some(href) => urls.push(href.as_str()),
                    None => urls.push(""),
                })
        }
    }

    Ok(urls)
}

/// Returns a list of directory links
pub fn parse_dirs<'a>(forge: &str, body: &'a str) -> Result<Vec<&'a str>, CrawlForgeError> {
    if forge == "github" {
        parse_dirs_github(body)
    } else {
        Err(CrawlForgeError::UnknownForge(forge.to_string()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_dirs_github() {
        let body = r#"
<div class="js-details-container">
    <div class="js-navigation-container">
      <div class="sr-only" role="row">
        <div role="columnheader">Type</div>
        <div role="columnheader">Name</div>
        <div role="columnheader" class="d-none d-md-block">Latest commit message</div>
        <div role="columnheader">Commit time</div>
      </div>

        <div role="row" class="js-navigation-item navigation-focus">
          <div role="gridcell" class="mr-3 flex-shrink-0" style="width: 16px;">
            <svg height="16" class="octicon octicon-file-directory color-blue-3" color="blue-3" aria-label="Directory" viewBox="0 0 16 16" version="1.1" width="16" role="img"><path fill-rule="evenodd" d="M1.75 1A1.75 1.75 0 000 2.75v10.5C0 14.216.784 15 1.75 15h12.5A1.75 1.75 0 0016 13.25v-8.5A1.75 1.75 0 0014.25 3h-6.5a.25.25 0 01-.2-.1l-.9-1.2c-.33-.44-.85-.7-1.4-.7h-3.5z"></path></svg>


          </div>

          <div role="rowheader" class="flex-auto min-width-0 col-md-2 mr-3">
            <span class="css-truncate css-truncate-target d-block width-fit"><a class="js-navigation-open link-gray-dark" title="This path skips through empty directories" id="812567e6c1916614728dd37787c921b2-4a916965d2930ecc6ca032c78de4363f21c2e04e" href="/tompreston/sup/tree/master/.github/workflows"><span class="text-gray-light">.github/</span>workflows</a></span>
          </div>

          <div role="gridcell" class="flex-auto min-width-0 d-none d-md-block col-5 mr-3 commit-message">
              <span class="css-truncate css-truncate-target d-block width-fit">
                    <a data-pjax="true" title="Create rust.yml" class="link-gray" href="/tompreston/sup/commit/e2a3a1c0820a618de0cc33351ae28acc6b3dea1f">Create rust.yml</a>
              </span>
          </div>

          <div role="gridcell" class="text-gray-light text-right" style="width:100px;">
              <time-ago datetime="2020-06-16T06:46:08Z" class="no-wrap" title="16 Jun 2020, 07:46 BST">16 days ago</time-ago>
          </div>

        </div>
        <div role="row" class="js-navigation-item">
          <div role="gridcell" class="mr-3 flex-shrink-0" style="width: 16px;">
            <svg height="16" class="octicon octicon-file-directory color-blue-3" color="blue-3" aria-label="Directory" viewBox="0 0 16 16" version="1.1" width="16" role="img"><path fill-rule="evenodd" d="M1.75 1A1.75 1.75 0 000 2.75v10.5C0 14.216.784 15 1.75 15h12.5A1.75 1.75 0 0016 13.25v-8.5A1.75 1.75 0 0014.25 3h-6.5a.25.25 0 01-.2-.1l-.9-1.2c-.33-.44-.85-.7-1.4-.7h-3.5z"></path></svg>


          </div>

          <div role="rowheader" class="flex-auto min-width-0 col-md-2 mr-3">
            <span class="css-truncate css-truncate-target d-block width-fit"><a class="js-navigation-open link-gray-dark" title="src" id="25d902c24283ab8cfbac54dfa101ad31-f6d2d7d8d9dd414edcad086cee817a9d171d48b1" href="/tompreston/sup/tree/master/src">src</a></span>
          </div>

          <div role="gridcell" class="flex-auto min-width-0 d-none d-md-block col-5 mr-3 commit-message">
              <span class="css-truncate css-truncate-target d-block width-fit">
                    <a data-pjax="true" title="cli: Add alias for commands" class="link-gray" href="/tompreston/sup/commit/2189dc2dc24d442b1f1933597bc0bba646dcb29d">cli: Add alias for commands</a>
              </span>
          </div>

          <div role="gridcell" class="text-gray-light text-right" style="width:100px;">
              <time-ago datetime="2020-06-23T14:15:48Z" class="no-wrap" title="23 Jun 2020, 15:15 BST">8 days ago</time-ago>
          </div>

        </div>
        <div role="row" class="js-navigation-item">
          <div role="gridcell" class="mr-3 flex-shrink-0" style="width: 16px;">
            <svg height="16" class="octicon octicon-file text-gray-light" color="gray-light" aria-label="File" viewBox="0 0 16 16" version="1.1" width="16" role="img"><path fill-rule="evenodd" d="M3.75 1.5a.25.25 0 00-.25.25v11.5c0 .138.112.25.25.25h8.5a.25.25 0 00.25-.25V6H9.75A1.75 1.75 0 018 4.25V1.5H3.75zm5.75.56v2.19c0 .138.112.25.25.25h2.19L9.5 2.06zM2 1.75C2 .784 2.784 0 3.75 0h5.086c.464 0 .909.184 1.237.513l3.414 3.414c.329.328.513.773.513 1.237v8.086A1.75 1.75 0 0112.25 15h-8.5A1.75 1.75 0 012 13.25V1.75z"></path></svg>


          </div>

          <div role="rowheader" class="flex-auto min-width-0 col-md-2 mr-3">
            <span class="css-truncate css-truncate-target d-block width-fit"><a class="js-navigation-open link-gray-dark" title=".gitignore" id="a084b794bc0759e7a6b77810e01874f2-ea8c4bf7f35f6f77f75d92ad8ce8349f6e81ddba" href="/tompreston/sup/blob/master/.gitignore">.gitignore</a></span>
          </div>

          <div role="gridcell" class="flex-auto min-width-0 d-none d-md-block col-5 mr-3 commit-message">
              <span class="css-truncate css-truncate-target d-block width-fit">
                    <a data-pjax="true" title="Initial commit" class="link-gray" href="/tompreston/sup/commit/cfc838d18d83401fb570e6676560f89254f35f54">Initial commit</a>
              </span>
          </div>

          <div role="gridcell" class="text-gray-light text-right" style="width:100px;">
              <time-ago datetime="2020-05-24T12:29:29Z" class="no-wrap" title="24 May 2020, 13:29 BST">last month</time-ago>
          </div>

        </div>
        <div role="row" class="Box-row Box-row--focus-gray py-2 d-flex position-relative js-navigation-item">
          <div role="gridcell" class="mr-3 flex-shrink-0" style="width: 16px;">
            <svg height="16" class="octicon octicon-file text-gray-light" color="gray-light" aria-label="File" viewBox="0 0 16 16" version="1.1" width="16" role="img"><path fill-rule="evenodd" d="M3.75 1.5a.25.25 0 00-.25.25v11.5c0 .138.112.25.25.25h8.5a.25.25 0 00.25-.25V6H9.75A1.75 1.75 0 018 4.25V1.5H3.75zm5.75.56v2.19c0 .138.112.25.25.25h2.19L9.5 2.06zM2 1.75C2 .784 2.784 0 3.75 0h5.086c.464 0 .909.184 1.237.513l3.414 3.414c.329.328.513.773.513 1.237v8.086A1.75 1.75 0 0112.25 15h-8.5A1.75 1.75 0 012 13.25V1.75z"></path></svg>


          </div>

          <div role="rowheader" class="flex-auto min-width-0 col-md-2 mr-3">
            <span class="css-truncate css-truncate-target d-block width-fit"><a class="js-navigation-open link-gray-dark" title=".travis.yml" id="354f30a63fb0907d4ad57269548329e3-df3e27d79308ea28a19710863d452a91cda4c777" href="/tompreston/sup/blob/master/.travis.yml">.travis.yml</a></span>
          </div>

          <div role="gridcell" class="flex-auto min-width-0 d-none d-md-block col-5 mr-3 commit-message">
              <span class="css-truncate css-truncate-target d-block width-fit">
                    <a data-pjax="true" title="Add travis.yml" class="link-gray" href="/tompreston/sup/commit/d083cc4256452b01d26192c5ee619af5a2a1791c">Add travis.yml</a>
              </span>
          </div>

          <div role="gridcell" class="text-gray-light text-right" style="width:100px;">
              <time-ago datetime="2020-05-24T13:00:52Z" class="no-wrap" title="24 May 2020, 14:00 BST">last month</time-ago>
          </div>

        </div>
        <div role="row" class="Box-row Box-row--focus-gray py-2 d-flex position-relative js-navigation-item ">
          <div role="gridcell" class="mr-3 flex-shrink-0" style="width: 16px;">
            <svg height="16" class="octicon octicon-file text-gray-light" color="gray-light" aria-label="File" viewBox="0 0 16 16" version="1.1" width="16" role="img"><path fill-rule="evenodd" d="M3.75 1.5a.25.25 0 00-.25.25v11.5c0 .138.112.25.25.25h8.5a.25.25 0 00.25-.25V6H9.75A1.75 1.75 0 018 4.25V1.5H3.75zm5.75.56v2.19c0 .138.112.25.25.25h2.19L9.5 2.06zM2 1.75C2 .784 2.784 0 3.75 0h5.086c.464 0 .909.184 1.237.513l3.414 3.414c.329.328.513.773.513 1.237v8.086A1.75 1.75 0 0112.25 15h-8.5A1.75 1.75 0 012 13.25V1.75z"></path></svg>


          </div>

          <div role="rowheader" class="flex-auto min-width-0 col-md-2 mr-3">
            <span class="css-truncate css-truncate-target d-block width-fit"><a class="js-navigation-open link-gray-dark" title="Cargo.lock" id="d2ede298d9a75c849170d6ef285eda1e-de850a321ecd86740d87b0c9e46512dc422954aa" href="/tompreston/sup/blob/master/Cargo.lock">Cargo.lock</a></span>
          </div>

          <div role="gridcell" class="flex-auto min-width-0 d-none d-md-block col-5 mr-3 commit-message">
              <span class="css-truncate css-truncate-target d-block width-fit">
                    <a data-pjax="true" title="Remove standup_error file, use thiserror" class="link-gray" href="/tompreston/sup/commit/e0e1f1ab33a3ff0c931194de96a733414f1e7a20">Remove standup_error file, use thiserror</a>
              </span>
          </div>

          <div role="gridcell" class="text-gray-light text-right" style="width:100px;">
              <time-ago datetime="2020-06-07T11:16:25Z" class="no-wrap" title="7 Jun 2020, 12:16 BST">24 days ago</time-ago>
          </div>

        </div>
        <div role="row" class="Box-row Box-row--focus-gray py-2 d-flex position-relative js-navigation-item ">
          <div role="gridcell" class="mr-3 flex-shrink-0" style="width: 16px;">
            <svg height="16" class="octicon octicon-file text-gray-light" color="gray-light" aria-label="File" viewBox="0 0 16 16" version="1.1" width="16" role="img"><path fill-rule="evenodd" d="M3.75 1.5a.25.25 0 00-.25.25v11.5c0 .138.112.25.25.25h8.5a.25.25 0 00.25-.25V6H9.75A1.75 1.75 0 018 4.25V1.5H3.75zm5.75.56v2.19c0 .138.112.25.25.25h2.19L9.5 2.06zM2 1.75C2 .784 2.784 0 3.75 0h5.086c.464 0 .909.184 1.237.513l3.414 3.414c.329.328.513.773.513 1.237v8.086A1.75 1.75 0 0112.25 15h-8.5A1.75 1.75 0 012 13.25V1.75z"></path></svg>


          </div>

          <div role="rowheader" class="flex-auto min-width-0 col-md-2 mr-3">
            <span class="css-truncate css-truncate-target d-block width-fit"><a class="js-navigation-open link-gray-dark" title="Cargo.toml" id="80398c5faae3c069e4e6aa2ed11b28c0-6c05c05618b459f19c31cd866925624b62dc6e80" href="/tompreston/sup/blob/master/Cargo.toml">Cargo.toml</a></span>
          </div>

          <div role="gridcell" class="flex-auto min-width-0 d-none d-md-block col-5 mr-3 commit-message">
              <span class="css-truncate css-truncate-target d-block width-fit">
                    <a data-pjax="true" title="Remove standup_error file, use thiserror" class="link-gray" href="/tompreston/sup/commit/e0e1f1ab33a3ff0c931194de96a733414f1e7a20">Remove standup_error file, use thiserror</a>
              </span>
          </div>

          <div role="gridcell" class="text-gray-light text-right" style="width:100px;">
              <time-ago datetime="2020-06-07T11:16:25Z" class="no-wrap" title="7 Jun 2020, 12:16 BST">24 days ago</time-ago>
          </div>

        </div>
        <div role="row" class="Box-row Box-row--focus-gray py-2 d-flex position-relative js-navigation-item ">
          <div role="gridcell" class="mr-3 flex-shrink-0" style="width: 16px;">
            <svg height="16" class="octicon octicon-file text-gray-light" color="gray-light" aria-label="File" viewBox="0 0 16 16" version="1.1" width="16" role="img"><path fill-rule="evenodd" d="M3.75 1.5a.25.25 0 00-.25.25v11.5c0 .138.112.25.25.25h8.5a.25.25 0 00.25-.25V6H9.75A1.75 1.75 0 018 4.25V1.5H3.75zm5.75.56v2.19c0 .138.112.25.25.25h2.19L9.5 2.06zM2 1.75C2 .784 2.784 0 3.75 0h5.086c.464 0 .909.184 1.237.513l3.414 3.414c.329.328.513.773.513 1.237v8.086A1.75 1.75 0 0112.25 15h-8.5A1.75 1.75 0 012 13.25V1.75z"></path></svg>


          </div>

          <div role="rowheader" class="flex-auto min-width-0 col-md-2 mr-3">
            <span class="css-truncate css-truncate-target d-block width-fit"><a class="js-navigation-open link-gray-dark" title="README.md" id="04c6e90faac2675aa89e2176d2eec7d8-eae4a22dfef67e93578df4679ab982328e5b1f3b" href="/tompreston/sup/blob/master/README.md">README.md</a></span>
          </div>

          <div role="gridcell" class="flex-auto min-width-0 d-none d-md-block col-5 mr-3 commit-message">
              <span class="css-truncate css-truncate-target d-block width-fit">
                    <a data-pjax="true" title="cli: Add alias for commands" class="link-gray" href="/tompreston/sup/commit/2189dc2dc24d442b1f1933597bc0bba646dcb29d">cli: Add alias for commands</a>
              </span>
          </div>

          <div role="gridcell" class="text-gray-light text-right" style="width:100px;">
              <time-ago datetime="2020-06-23T14:15:48Z" class="no-wrap" title="23 Jun 2020, 15:15 BST">8 days ago</time-ago>
          </div>

        </div>
    </div>
    <div class="Details-content--shown Box-footer d-md-none p-0">
      <button type="button" class="d-block btn-link js-details-target width-full px-3 py-2" aria-expanded="false">
        View code
      </button>
    </div>
  </div>"#;

        parse_dirs_github(body).unwrap();
        panic!();
    }
}
