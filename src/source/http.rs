//! Polite downloader for wiki modules (`?action=raw`): explicit User-Agent, timeouts and a
//! pause between requests.

use std::time::Duration;

use reqwest::blocking::Client;
use reqwest::header::LAST_MODIFIED;

pub const WIKI_BASE_URL: &str = "https://wiki.warframe.com/w/";

const USER_AGENT: &str = concat!(
    "Plop-Warframe/",
    env!("CARGO_PKG_VERSION"),
    " (application de bureau personnelle ; lecture des tables de drop)"
);

#[derive(Debug, thiserror::Error)]
pub enum FetchError {
    #[error("client HTTP : {0}")]
    Client(#[source] reqwest::Error),
    #[error("téléchargement de « {page} » : {source}")]
    Request {
        page: String,
        #[source]
        source: reqwest::Error,
    },
    #[error("téléchargement de « {page} » : réponse HTTP {status}")]
    Status { page: String, status: u16 },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FetchedPage {
    pub text: String,
    /// `Last-Modified` header, when the server sends one.
    pub last_modified: Option<String>,
}

/// Blocking client: call it from a worker thread, never from the UI thread.
pub struct ModuleFetcher {
    client: Client,
    base_url: String,
    pause: Duration,
    requests: u32,
}

impl ModuleFetcher {
    pub fn new() -> Result<Self, FetchError> {
        Self::with_base_url(WIKI_BASE_URL, Duration::from_secs(1))
    }

    pub fn with_base_url(base_url: &str, pause: Duration) -> Result<Self, FetchError> {
        let client = Client::builder()
            .user_agent(USER_AGENT)
            .connect_timeout(Duration::from_secs(15))
            .timeout(Duration::from_secs(60))
            .build()
            .map_err(FetchError::Client)?;
        Ok(Self {
            client,
            base_url: base_url.to_owned(),
            pause,
            requests: 0,
        })
    }

    /// Downloads the raw wikitext of `page` (e.g. `Module:DropTables/data`). Requests are
    /// sequential and spaced by the pause, to spare the wiki.
    pub fn fetch_raw(&mut self, page: &str) -> Result<FetchedPage, FetchError> {
        if self.requests > 0 {
            std::thread::sleep(self.pause);
        }
        self.requests += 1;

        let url = format!("{}{}?action=raw", self.base_url, page);
        let request_error = |source| FetchError::Request {
            page: page.to_owned(),
            source,
        };
        let response = self.client.get(url).send().map_err(request_error)?;
        let status = response.status();
        if !status.is_success() {
            return Err(FetchError::Status {
                page: page.to_owned(),
                status: status.as_u16(),
            });
        }
        let last_modified = response
            .headers()
            .get(LAST_MODIFIED)
            .and_then(|value| value.to_str().ok())
            .map(str::to_owned);
        let text = response.text().map_err(request_error)?;
        Ok(FetchedPage {
            text,
            last_modified,
        })
    }
}
