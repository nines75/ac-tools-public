use std::{
    borrow::Cow::{Borrowed, Owned},
    error::Error,
    fs::{self, OpenOptions},
    io::Write,
    path::PathBuf,
};

use chrono::Local;
use percent_encoding::percent_decode_str;

use crate::{val::base_path, CustomError::*, Message};

pub fn run(
    contest_name_id: &str,
    problem_id: &str,
    sourcecode_path: PathBuf,
    contest_type: u32,
) -> Result<(), Box<dyn Error>> {
    // atcoder
    if contest_type == 0 {
        // クッキー読み込み
        let cookie = get_cookie("atcoder.jp")?;

        // 提出
        {
            // クッキーからtoken取得
            // 1. デコード
            // 2. '; 'ごとに区切って、その中で"csrf_token"が含まれているものを抽出
            // 3. NULL文字ごとに区切る
            // 4. ':'前後でkey-valueとして、keyがcsrf_tokenのものを探す
            let decoded_cookie = percent_decode_str(&cookie).decode_utf8_lossy();
            let mut decoded_cookie_vec: Vec<&str> = Vec::new();
            for i in decoded_cookie.split("; ") {
                if i.contains("csrf_token") {
                    decoded_cookie_vec = i.split("\0\0").collect();
                    break;
                }
            }

            let mut token: Option<&str> = None;
            for i in decoded_cookie_vec {
                let mut key_value = i.split(":");
                if key_value.next().ok_or(IndexError)? == "csrf_token" {
                    token = Some(key_value.next().ok_or(IndexError)?);
                }
            }

            // ソースコード読み込み
            let sourcecode = fs::read_to_string(&sourcecode_path).or_else(|_| {
                Err(Box::new(FileNotfoundError(Owned(
                    sourcecode_path
                        .to_str()
                        .ok_or(InvalidUnicodeError)?
                        .to_string(),
                ))))
            })?;

            // POST
            let submit_url = format!("https://atcoder.jp/contests/{}/submit", contest_name_id);
            let agent = ureq::post(&submit_url).set("Cookie", &cookie);
            match agent.send_form(&[
                (
                    "data.TaskScreenName",
                    &format!("{}_{}", contest_name_id, problem_id),
                ),
                ("data.LanguageId", "5028"),
                ("sourceCode", &sourcecode),
                ("csrf_token", token.ok_or(InvalidCookieError)?),
            ]) {
                Ok(_) => {}
                Err(error) => {
                    match &error {
                        ureq::Error::Status(code, res) => {
                            // issue #24: logging
                            let mut log_file = OpenOptions::new()
                                .append(true)
                                .create(true)
                                .open(format!("{}/log.txt", base_path()?))?;
                            log_file.write_all((Local::now().to_string() + "\n").as_bytes())?;
                            log_file.write_all((submit_url + "\n").as_bytes())?;
                            log_file.write_all(token.ok_or(InvalidCookieError)?.as_bytes())?;
                            log_file.write_all(
                                format!("\nstatus code:{}\n{:?}\n\n", code, res).as_bytes(),
                            )?;
                        }
                        // httpエラー以外は無視
                        _ => {}
                    }
                    return Err(Box::new(error));
                }
            }
            println!("{} 提出に成功しました", Message::Success);
        }

        // 提出結果を取得して開く
        {
            // 提出一覧のHTMLを取得してparse
            let submission_list_url = format!(
                "https://atcoder.jp/contests/{}/submissions/me",
                contest_name_id
            );
            let html = ureq::get(&submission_list_url)
                .set("Cookie", &cookie)
                .call()?
                .into_string()?;
            let doc = scraper::Html::parse_document(&html);

            // 提出urlが書いてある要素を取り出す
            let selector = scraper::Selector::parse(
                ".table-bordered > tbody > tr:nth-child(1) > td:last-child > a",
            )?;
            let element = doc.select(&selector).next().ok_or(HtmlError)?;

            // url生成
            let submission_url = format!(
                "https://atcoder.jp{}",
                element.value().attr("href").ok_or(HtmlError)?
            );

            // ブラウザで開く
            println!(
                "{} 提出先のページを既定のブラウザで開きます: {}",
                Message::Info,
                submission_url
            );
            open::that(submission_url)?;
        }
    }

    return Ok(());
}

fn get_cookie(domain: &str) -> Result<String, Box<dyn Error>> {
    let cookies = fs::read_to_string(format!(
        "{}/.local/share/online-judge-tools/cookie.jar",
        shellexpand::tilde("~")
    ))
    .or_else(|_| Err(Box::new(FileNotfoundError(Borrowed("cookie.jar")))))?;

    let mut res: Option<String> = None;
    for cookie in cookies.lines() {
        // クッキーかどうか判定
        if !cookie.starts_with("Set-Cookie3") {
            continue;
        }

        // ドメインと認証情報が入っているかチェック
        if cookie.contains(&format!("domain=\"{}\"", domain)) && cookie.contains("REVEL_SESSION") {
            // クッキー部分を抽出
            let mut str = String::new();
            for (index, c) in cookie.chars().enumerate() {
                if index >= 13 {
                    str.push(c);
                };
            }
            // '"'を除去
            str.retain(|c| c != '"');

            res = Some(str);
        }
    }

    return Ok(res.ok_or(InvalidCookieError)?);
}
