use serde_json::json;
use thirtyfour::{By, ChromiumLikeCapabilities, DesiredCapabilities, WebDriver, WebElement};
use url::Url;

pub struct Web {
    driver: WebDriver,
}

impl Web {
    pub async fn new() -> anyhow::Result<Web> {
        let mut caps = DesiredCapabilities::chrome();
        caps.add_arg("--headless").unwrap();
        caps.add_experimental_option(
            "prefs",
            json!({
                "protocol_handler": {
                    "excluded_schemes": {
                        "pulsesecure": "false"
                    }
                }
            }),
        )?;
        let driver = WebDriver::new("http://localhost:9515", caps).await?;
        Ok(Web { driver })
    }
    pub async fn goto(&mut self, url: &str) -> anyhow::Result<()> {
        self.driver.goto(url).await?;
        Ok(())
    }

    pub async fn _find(&mut self, by: By) -> anyhow::Result<WebElement> {
        let elem = self.driver.find(by.clone()).await?;
        Ok(elem)
    }

    pub async fn send_text(&mut self, by: By, text: &str) -> anyhow::Result<()> {
        let elem = self.driver.find(by.clone()).await?;
        elem.send_keys(text).await?;
        Ok(())
    }

    pub async fn click(&mut self, by: By) -> anyhow::Result<()> {
        let elem = self.driver.find(by.clone()).await?;
        elem.click().await?;
        Ok(())
    }

    pub async fn enter_iframe(&mut self, by: By) -> anyhow::Result<()> {
        let elem = self.driver.find(by.clone()).await?;
        elem.enter_frame().await?;
        Ok(())
    }

    pub async fn try_find_repeatedly(&mut self, by: By, limit: i32) -> anyhow::Result<WebElement> {
        for _ in 0..limit {
            if let Ok(elem) = self.driver.find(by.clone()).await {
                return Ok(elem);
            };
            tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
        }
        anyhow::bail!("Timeout")
    }

    pub async fn current_url(&mut self) -> anyhow::Result<Url> {
        let url = self.driver.current_url().await?;
        Ok(url)
    }

    pub async fn quit(self) -> anyhow::Result<()> {
        self.driver.quit().await?;
        Ok(())
    }
}
