mod mail;
mod config;
mod web;

use std::time::Duration;

use mail::MailHelper;
use config::{Config, GmailConfig, KaistConfig};
use web::Web;

use thirtyfour::prelude::*;
use tokio::time;

const CONFIG_PATH: &str = "setting.toml";
const SENDER_QUERY: &str = "FROM no-reply-kvpn@kaist.ac.kr";

fn login_mail(config: &GmailConfig) -> anyhow::Result<MailHelper> {
    let id = &config.id;
    let pw = &config.imap_token;
    match MailHelper::login(id, pw) {
        Some(mail) => Ok(mail),
        None => anyhow::bail!("Cannot login to the mail server")
    }
}

async fn login_web(web: &mut Web, config: &KaistConfig) -> anyhow::Result<()> {
    let id = &config.id;
    let pw = &config.pw;
    web.goto("https://kvpn.kaist.ac.kr/").await?;
    web.send_text(By::XPath(r#"//*[@id="frmLogin_4"]/div/div/ul/li[1]/input"#), id).await?;
    web.send_text(By::XPath(r#"//*[@id="frmLogin_4"]/div/div/ul/li[2]/input"#), pw).await?;
    web.click(By::XPath(r#"//*[@id="frmLogin_4"]/div/button"#)).await?;
    Ok(())
}

async fn enter_email_auth_mode(web: &mut Web) -> anyhow::Result<()> {
    web.enter_iframe(By::XPath(r#"//*[@id="tp_frame"]"#)).await?;
    web.try_find_repeatedly(By::Id("menu_email"), 10).await?.click().await?;
    Ok(())
}

async fn fetch_auth_mail_text(mail_helper: &mut MailHelper, prev_max_uid: i64) -> anyhow::Result<String> {
    for _ in 0..30 {
        let uid = get_max_uid(mail_helper);
        if uid > prev_max_uid {
            return Ok(mail_helper.read_text(uid as u32, "body[text]"));
        }
        println!("Waiting for the auth mail to arrive...");
        time::sleep(Duration::from_secs(1)).await;
    }
    anyhow::bail!("Timeout")
}

fn extract_auth_code_from_text(text: &str) -> String {
    // TODO: refinement
    let mut text = text.to_string();
    let i = text.find("Please enter OTP").unwrap();
    let mut a = text.split_off(i);
    let k = "</td>";
    let ii = a.find(k).unwrap() - 6 - 1;
    let mut aa = a.split_off(ii);
    let _ = aa.split_off(6);
    aa
}

async fn fetch_auth_code(mail_helper: &mut MailHelper, prev_max_uid: i64) -> anyhow::Result<String> {
    let text = fetch_auth_mail_text(mail_helper, prev_max_uid).await?;
    let code = extract_auth_code_from_text(&text);
    println!("Auth code: {}", code);
    Ok(code)
}

async fn pass_auth(web: &mut Web, mail_helper: &mut MailHelper, prev_max_uid: i64) -> anyhow::Result<()> {
    let auth_code = fetch_auth_code(mail_helper, prev_max_uid).await?;
    web.send_text(By::Id("otp_input"), &auth_code).await?;
    web.click(By::Id("login_btn")).await?;
    Ok(())
}

async fn wait_for_client(web: &mut Web) -> anyhow::Result<()> {
    for _ in 0..30 {
        if let Ok(url) = web.current_url().await {
            if url.path().contains("dana/home/index.cgi") {
                return Ok(());
            }
        }
        println!("Waiting for the client to be launched...");
        time::sleep(Duration::from_secs(1)).await;
    }
    anyhow::bail!("Timeout")
}

async fn enjoy(mail_helper: &mut MailHelper, config: Config, prev_max_uid: i64, web: &mut Web) -> anyhow::Result<()> {
    login_web(web, &config.kaist).await?;
    enter_email_auth_mode(web).await?;
    pass_auth(web, mail_helper, prev_max_uid).await?;
    wait_for_client(web).await?;
    Ok(())
}

fn get_max_uid(mail_helper: &mut MailHelper) -> i64 {
    let uids = mail_helper.search(SENDER_QUERY);
    match uids.iter().max() {
        Some(uid) => *uid as i64,
        None => -1
    }
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let config = config::load(CONFIG_PATH)?;
    let mut mail_helper = login_mail(&config.gmail)?;
    let prev_max_uid: i64 = get_max_uid(&mut mail_helper);
    let mut web = Web::new().await?;
    match enjoy(&mut mail_helper, config, prev_max_uid, &mut web).await {
        Ok(_) => {}
        Err(err) => { println!("{:?}", err) }
    }
    web.quit().await?;
    Ok(())
}