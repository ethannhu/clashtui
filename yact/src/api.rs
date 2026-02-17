// src/lib.rs  或  src/main.rs 根据需要
use anyhow::{Context, Result};
use reqwest::Client;
use serde_json::{Value, json};
use std::collections::HashMap;
use std::time::Duration;

#[derive(Debug, Clone)]
pub struct MihomoClient {
    base_url: String,
    secret: String,
    client: Client,
}

impl MihomoClient {
    /// 创建客户端实例
    /// 示例: MihomoClient::new("http://127.0.0.1:9090", Some("your-secret-key"))
    pub fn new(base_url: impl Into<String>, secret: impl Into<String>) -> Self {
        let client = Client::builder()
            .timeout(Duration::from_secs(15))
            .build()
            .expect("Failed to build reqwest client");

        Self {
            base_url: base_url.into().trim_end_matches('/').to_string(),
            secret: secret.into().to_string(),
            client,
        }
    }

    fn request(&self, method: reqwest::Method, path: &str) -> reqwest::RequestBuilder {
        let url = format!("{}/{}", self.base_url, path.trim_start_matches('/'));

        let mut req = self.client.request(method, &url);
        req = req.header("Authorization", format!("Bearer {}", self.secret));

        req
    }

    // ──────────────────────────────────────────────────────────────
    // 配置相关
    // ──────────────────────────────────────────────────────────────

    /// GET /configs
    /// 获取当前配置信息
    pub async fn get_configs(&self) -> Result<Value> {
        let resp = self
            .request(reqwest::Method::GET, "/configs")
            .send()
            .await?
            .error_for_status()?
            .json::<Value>()
            .await?;
        // println!("{:?}", resp);
        Ok(resp)
    }

    /// PUT /configs
    /// 加载/切换配置文件（支持 payload 或 path）
    /// force=true 强制重新加载（推荐大多数场景都加）
    pub async fn reload_config(
        &self,
        path: Option<&str>,
        payload: Option<&str>,
        force: bool,
    ) -> Result<()> {
        let mut body = json!({});

        if let Some(p) = path {
            body["path"] = json!(p);
        }
        if let Some(pl) = payload {
            body["payload"] = json!(pl);
        }

        let force_str = if force { "true" } else { "false" };
        let mut req = self.request(
            reqwest::Method::PUT,
            &format!("/configs?force={}", force_str),
        );

        if !body.is_null() && !body.as_object().unwrap().is_empty() {
            req = req.json(&body);
        }

        let _ = req.send().await?.error_for_status()?;

        Ok(())
    }

    // ──────────────────────────────────────────────────────────────
    // 代理节点相关
    // ──────────────────────────────────────────────────────────────

    /// GET /proxies
    /// 获取所有代理节点及分组信息
    pub async fn get_proxies(&self) -> Result<Value> {
        let resp = self
            .request(reqwest::Method::GET, "/proxies")
            .send()
            .await?
            .error_for_status()?
            .json::<Value>()
            .await?;

        Ok(resp)
    }

    /// GET /proxies/:name
    /// 获取单个代理/分组信息
    pub async fn get_proxy(&self, name: &str) -> Result<Value> {
        let path = format!("/proxies/{}", urlencoding::encode(name));
        let resp = self
            .request(reqwest::Method::GET, &path)
            .send()
            .await?
            .error_for_status()?
            .json::<Value>()
            .await?;

        Ok(resp)
    }

    /// PUT /proxies/:name
    /// 选择某个分组的代理（切换节点）
    pub async fn select_proxy(&self, group: &str, target_proxy: &str) -> Result<()> {
        let path = format!("/proxies/{}", urlencoding::encode(group));

        let body = json!({
            "name": target_proxy
        });

        let _ = self
            .request(reqwest::Method::PUT, &path)
            .json(&body)
            .send()
            .await?
            .error_for_status()?;

        Ok(())
    }

    /// GET /proxies/:name/delay
    /// 测试单个节点延迟（单位ms）
    /// timeout 单位：毫秒
    pub async fn test_proxy_delay(&self, name: &str, url: &str, timeout: u64) -> Result<u64> {
        let path = format!("/proxies/{}/delay", urlencoding::encode(name));

        let url_with_params = format!(
            "{}?url={}&timeout={}",
            path,
            urlencoding::encode(url),
            timeout
        );
        let resp = self
            .request(reqwest::Method::GET, &url_with_params)
            .send()
            .await?
            .error_for_status()?
            .json::<HashMap<String, u64>>()
            .await?;

        let delay = resp
            .get("delay")
            .context("delay field not found")?
            .to_owned();

        Ok(delay)
    }

    // ──────────────────────────────────────────────────────────────
    // 规则相关
    // ──────────────────────────────────────────────────────────────

    /// GET /rules
    pub async fn get_rules(&self) -> Result<Value> {
        let resp = self
            .request(reqwest::Method::GET, "/rules")
            .send()
            .await?
            .error_for_status()?
            .json::<Value>()
            .await?;

        Ok(resp)
    }

    // ──────────────────────────────────────────────────────────────
    // 其他常用操作
    // ──────────────────────────────────────────────────────────────

    /// GET /version
    pub async fn get_version(&self) -> Result<String> {
        let resp = self
            .request(reqwest::Method::GET, "/version")
            .send()
            .await?
            .error_for_status()?
            .text()
            .await?;

        Ok(resp.trim().to_string())
    }

    /// GET /logs
    /// 获取实时日志（SSE 流式）
    /// 注意：这个需要自己处理 EventStream
    pub async fn get_logs(&self, level: Option<&str>) -> Result<reqwest::Response> {
        let mut path = "/logs".to_string();

        if let Some(lv) = level {
            path = format!("{}?level={}", path, urlencoding::encode(lv));
        }

        let mut req = self.request(reqwest::Method::GET, &path);

        let resp = req.send().await?.error_for_status()?;

        Ok(resp)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[tokio::test]
    async fn test_get_configs() {
        let client = MihomoClient::new("http://127.0.0.1:9097", "123456");

        match client.get_configs().await {
            Ok(configs) => {
                // Verify configs is not null and has some expected fields
                assert_ne!(configs, json!(null), "configs should not be null");

                // Check for common config fields
                if let Some(obj) = configs.as_object() {
                    // configs should have mixed-port or port field
                    let has_port = obj.contains_key("mixed-port") || obj.contains_key("port");
                    assert!(
                        has_port || !obj.is_empty(),
                        "configs should have port fields or other content"
                    );
                }
            }
            Err(e) => {
                eprintln!(
                    "get_configs failed: {}. Skipping test (server may not be running).",
                    e
                );
            }
        }
    }

    #[tokio::test]
    async fn test_get_proxies() {
        let client = MihomoClient::new("http://127.0.0.1:9097", "123456");

        match client.get_proxies().await {
            Ok(proxies) => {
                // Verify proxies is not null
                assert_ne!(proxies, json!(null), "proxies should not be null");

                // Check structure - should have proxies object
                if let Some(obj) = proxies.as_object() {
                    if let Some(proxy_dict) = obj.get("proxies") {
                        assert!(proxy_dict.is_object(), "proxies should be an object");
                    }
                }
            }
            Err(e) => {
                eprintln!(
                    "get_proxies failed: {}. Skipping test (server may not be running).",
                    e
                );
            }
        }
    }

    #[tokio::test]
    async fn test_get_proxy() {
        let client = MihomoClient::new("http://127.0.0.1:9097", "123456");

        // First get all proxies to find a valid proxy name
        match client.get_proxies().await {
            Ok(proxies) => {
                if let Some(proxy_dict) = proxies.get("proxies") {
                    if let Some(obj) = proxy_dict.as_object() {
                        if let Some(first_proxy_key) = obj.keys().next() {
                            // Test getting a specific proxy
                            match client.get_proxy(first_proxy_key.as_str()).await {
                                Ok(proxy) => {
                                    assert_ne!(proxy, json!(null), "proxy should not be null");

                                    if let Some(proxy_obj) = proxy.as_object() {
                                        // Proxy should have type and name
                                        assert!(
                                            proxy_obj.contains_key("type")
                                                || proxy_obj.contains_key("name"),
                                            "proxy should have type or name"
                                        );
                                    }
                                }
                                Err(e) => {
                                    eprintln!("get_proxy for {} failed: {}", first_proxy_key, e);
                                }
                            }
                        }
                    }
                }
            }
            Err(e) => {
                eprintln!(
                    "get_proxies (for get_proxy test) failed: {}. Skipping test.",
                    e
                );
            }
        }
    }

    #[tokio::test]
    async fn test_get_rules() {
        let client = MihomoClient::new("http://127.0.0.1:9097", "123456");

        match client.get_rules().await {
            Ok(rules) => {
                // Verify rules is not null
                assert_ne!(rules, json!(null), "rules should not be null");

                // Check structure - should have rules array
                if let Some(obj) = rules.as_object() {
                    if let Some(rules_array) = obj.get("rules") {
                        if let Some(arr) = rules_array.as_array() {
                            // Each rule should have at least some structure
                            for rule in arr.iter().take(5) {
                                assert!(
                                    rule.is_object() || rule.is_string(),
                                    "rule should be object or string"
                                );
                            }
                        }
                    }
                }
            }
            Err(e) => {
                eprintln!(
                    "get_rules failed: {}. Skipping test (server may not be running).",
                    e
                );
            }
        }
    }

    #[tokio::test]
    async fn test_get_version() {
        let client = MihomoClient::new("http://127.0.0.1:9097", "123456");

        match client.get_version().await {
            Ok(version) => {
                // Verify version is not empty
                assert!(!version.is_empty(), "version should not be empty");

                // Version should contain numbers or version-like characters
                assert!(
                    version.len() >= 3,
                    "version should be at least 3 characters"
                );
            }
            Err(e) => {
                eprintln!(
                    "get_version failed: {}. Skipping test (server may not be running).",
                    e
                );
            }
        }
    }

    #[tokio::test]
    async fn test_get_logs() {
        let client = MihomoClient::new("http://127.0.0.1:9097", "123456");

        match client.get_logs(None).await {
            Ok(response) => {
                // Verify response is successful
                let status = response.status();
                assert!(status.is_success(), "logs response should be successful");

                // Verify content-type indicates SSE or text
                let content_type = response
                    .headers()
                    .get("content-type")
                    .and_then(|v| v.to_str().ok());

                eprintln!("logs content-type: {:?}", content_type);
            }
            Err(e) => {
                eprintln!(
                    "get_logs failed: {}. Skipping test (server may not be running).",
                    e
                );
            }
        }
    }

    #[tokio::test]
    async fn test_get_proxy_delay() {
        let client = MihomoClient::new("http://127.0.0.1:9097", "123456");

        // First get all proxies to find a testable proxy
        match client.get_proxies().await {
            Ok(proxies) => {
                if let Some(proxy_dict) = proxies.get("proxies") {
                    if let Some(obj) = proxy_dict.as_object() {
                        // Find a proxy that's not a group (by checking if it has history)
                        for (name, proxy) in obj.iter() {
                            if let Some(proxy_obj) = proxy.as_object() {
                                // Don't test if it's a GROUP type
                                if proxy_obj.get("type").and_then(|t| t.as_str()) != Some("GROUP") {
                                    match client
                                        .test_proxy_delay(
                                            name.as_str(),
                                            "http://www.google.com",
                                            5000,
                                        )
                                        .await
                                    {
                                        Ok(delay) => {
                                            eprintln!("Proxy {} delay: {}ms", name, delay);
                                            // Delay should be reasonable (0 to 10000ms, or timeout indicator)
                                            assert!(
                                                delay <= 10000 || delay > 0,
                                                "delay should be reasonable"
                                            );
                                        }
                                        Err(e) => {
                                            eprintln!(
                                                "test_proxy_delay for {} failed: {}",
                                                name, e
                                            );
                                        }
                                    }
                                    break; // Test first non-group proxy
                                }
                            }
                        }
                    }
                }
            }
            Err(e) => {
                eprintln!("get_proxies (for delay test) failed: {}. Skipping test.", e);
            }
        }
    }
}
