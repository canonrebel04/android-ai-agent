pub struct HttpClient {
    client: reqwest::Client,
}

impl HttpClient {
    pub fn new() -> Self {
        Self {
            client: reqwest::Client::builder()
                .timeout(std::time::Duration::from_secs(120))
                .build()
                .expect("failed to build HTTP client"),
        }
    }

    pub fn inner(&self) -> &reqwest::Client {
        &self.client
    }

    /// Send a cached request — applies cache_control markers
    /// before sending to reduce costs for supported providers.
    pub fn send_cached(
        &self,
        url: String,
        headers: Vec<(String, String)>,
        body: serde_json::Value,
        modified_body: Option<serde_json::Value>,
        extra_headers: Vec<(String, String)>,
    ) -> impl std::future::Future<Output = Result<reqwest::Response, reqwest::Error>> {
        let client = self.client.clone();
        async move {
            let request_body = modified_body.unwrap_or(body);
            let mut req = client.post(&url).json(&request_body);

            for (k, v) in &headers {
                req = req.header(k.as_str(), v.as_str());
            }
            for (k, v) in &extra_headers {
                req = req.header(k.as_str(), v.as_str());
            }

            req.send().await
        }
    }
}

impl Default for HttpClient {
    fn default() -> Self {
        Self::new()
    }
}
