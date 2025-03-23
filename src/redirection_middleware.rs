use surf::middleware::{Middleware, Next};
use surf::{Client, Request, Response, Result, StatusCode, Url};

pub struct RedirectMiddleware {
    max_redirects: usize,
}

impl RedirectMiddleware {
    pub fn new(max_redirects: usize) -> Self {
        Self { max_redirects }
    }
}

#[surf::utils::async_trait]
impl Middleware for RedirectMiddleware {
    async fn handle(&self, req: Request, client: Client, _: Next<'_>) -> Result<Response> {
        let mut req = req;
        for _ in 0..self.max_redirects {
            let res = client.send(req.clone()).await?;
            if res.status().is_redirection() {
                if let Some(location) = res.header("Location") {
                    req = Request::new(req.method(), Url::parse(location.last().as_str()).unwrap());
                    continue;
                }
            }
            return Ok(res);
        }
        Err(surf::Error::from_str(
            StatusCode::TooManyRequests,
            "Too many redirects, Increase max_redirections in DLStartConfig",
        ))
    }
}
