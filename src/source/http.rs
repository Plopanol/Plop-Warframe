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
    #[error("téléchargement de « {page} » : {}", request_reason(.source))]
    Request {
        page: String,
        #[source]
        source: reqwest::Error,
    },
    #[error("téléchargement de « {page} » : {}", status_reason(*.status))]
    Status { page: String, status: u16 },
}

/// French explanation of a transport failure; reqwest's own text is English and technical.
fn request_reason(error: &reqwest::Error) -> String {
    transport_reason(error.is_timeout(), error.is_connect(), &error.to_string())
}

fn transport_reason(timeout: bool, connect: bool, detail: &str) -> String {
    if timeout {
        "le wiki ne répond pas dans les délais, réessayez plus tard".to_owned()
    } else if connect {
        "connexion au wiki impossible, vérifiez l'accès à Internet".to_owned()
    } else {
        format!("erreur réseau ({detail})")
    }
}

fn status_reason(status: u16) -> String {
    match status {
        404 => "page introuvable sur le wiki (réponse HTTP 404)".to_owned(),
        429 => "le wiki limite les requêtes, réessayez plus tard (réponse HTTP 429)".to_owned(),
        500..=599 => {
            format!("le wiki est indisponible, réessayez plus tard (réponse HTTP {status})")
        }
        _ => format!("réponse HTTP {status} inattendue du wiki"),
    }
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn explains_transport_failures() {
        assert!(transport_reason(true, false, "x").contains("délais"));
        assert!(transport_reason(false, true, "x").contains("Internet"));
        assert_eq!(transport_reason(false, false, "x"), "erreur réseau (x)");
    }

    #[test]
    fn explains_http_statuses() {
        let error = FetchError::Status {
            page: "Module:A".into(),
            status: 503,
        };
        assert_eq!(
            error.to_string(),
            "téléchargement de « Module:A » : le wiki est indisponible, réessayez plus tard (réponse HTTP 503)"
        );
        assert!(status_reason(404).contains("introuvable"));
        assert!(status_reason(429).contains("limite"));
        assert_eq!(status_reason(302), "réponse HTTP 302 inattendue du wiki");
    }
}
