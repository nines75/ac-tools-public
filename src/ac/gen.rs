use std::{
    borrow::Cow::Borrowed,
    env,
    error::Error,
    fs::{self, File},
    io::Write,
    path::Path,
    process::Command,
};

use ac_tools_rs::{
    val::{self, base_path},
    CustomError::*,
    Message, Warning,
};
use arboard::Clipboard;
use chrono::{offset::LocalResult, Local, TimeZone};
use colored::Colorize;
use dialoguer::{theme::ColorfulTheme, Input, Select};
use serde::Deserialize;

#[derive(Deserialize, Debug)]
struct YukiContest {
    #[serde(rename = "Name")]
    name: String,
    #[serde(rename = "Date")]
    date: String,
    #[serde(rename = "ProblemIdList")]
    problem_id_list: Vec<u64>,
}

#[derive(Deserialize, Debug)]
struct RecentVirtualContest {
    title: String,
    id: String,
    start_epoch_second: i64,
}

#[derive(Deserialize, Debug)]
struct VirtualContest {
    problems: Vec<VirtualProblem>,
}

#[derive(Deserialize, Debug)]
struct VirtualProblem {
    id: String,
}

pub fn run() -> Result<(), Box<dyn Error>> {
    let choices = &["1:AtCoder", "2:Codeforces", "3:yukicoder", "4:バーチャル"];
    let choice = Select::with_theme(&ColorfulTheme::default())
        .with_prompt(format!(
            "{} コンテストの種類を選択してください",
            Message::Question
        ))
        .items(choices)
        .default(0)
        .interact()?;

    if choice == 0 {
        atcoder()?;
    }

    if choice == 1 {
        codeforces()?;
    }

    if choice == 2 {
        yukicoder()?;
    }

    if choice == 3 {
        virtual_contest()?;
    }

    return Ok(());
}

/// pathの存在確認
fn check_path(path: &Path) -> Result<bool, Box<dyn Error>> {
    if !path.is_dir() {
        fs::create_dir_all(path)?;
        return Ok(true);
    } else {
        let (current_dir_name, parent_dir_name) = val::path_name(path)?;

        return Ok(
            Warning::Overwrite(format!("{}/{}", parent_dir_name, current_dir_name)).start()?,
        );
    }
}

// 指定されたパスにコンテスト用のファイルを生成
fn make_path(contest_path: &Path, num: u8) -> Result<(), Box<dyn Error>> {
    let mut file_name_vec = vec![String::from("test")];

    for i in 'a'..('a' as u8 + num) as char {
        file_name_vec.push(i.to_string());
    }

    let template_path = val::template_path()?;

    for i in file_name_vec {
        fs::copy(&template_path, contest_path.join(format!("{}.cpp", i)))?;
    }

    return Ok(());
}

fn atcoder() -> Result<(), Box<dyn Error>> {
    let contest_name;
    let contest_id;

    loop {
        let tmp_contest_name: String = Input::with_theme(&ColorfulTheme::default())
            .with_prompt(format!("{} コンテスト名", Message::Input))
            .interact_text()?;

        let tmp_contest_id: String = Input::with_theme(&ColorfulTheme::default())
            .with_prompt(format!("{} コンテストID", Message::Input))
            .interact_text()?;

        let contest_url = format!(
            "https://atcoder.jp/contests/{}{}",
            tmp_contest_name, tmp_contest_id
        );

        match ureq::get(&contest_url).call() {
            Ok(_) => {
                contest_name = tmp_contest_name;
                contest_id = tmp_contest_id;
                println!(
                    "{}{} URL:{} から正常な応答が返されたことを確認しました",
                    "✔ ".green(),
                    Message::Info,
                    contest_url
                );
                break;
            }
            Err(error) => {
                println!("{} {}", Message::RequestError, error);
            }
        }
    }

    let contest_path_str = format!("{}/contest/{}/{}", base_path()?, contest_name, contest_id);

    let contest_path = Path::new(&contest_path_str);
    if !check_path(contest_path)? {
        return Ok(());
    }

    make_path(contest_path, 8)?;
    println!("{} コンテストの構成に成功しました", Message::Success);

    set_cd_clipboard(contest_path)?;

    check_login("https://atcoder.jp/")?;

    return Ok(());
}

fn codeforces() -> Result<(), Box<dyn Error>> {
    let contest_id: String = Input::with_theme(&ColorfulTheme::default())
        .with_prompt(format!("{} コンテスト番号", Message::Input))
        .interact_text()?;

    let contest_path_str = format!("{}/contest/codeforces/{}", base_path()?, contest_id);

    let contest_path = Path::new(&contest_path_str);
    if !check_path(contest_path)? {
        return Ok(());
    }

    make_path(contest_path, 8)?;
    println!("{} コンテストの構成に成功しました", Message::Success);

    set_cd_clipboard(contest_path)?;

    return Ok(());
}

fn yukicoder() -> Result<(), Box<dyn Error>> {
    let contest_id: String;
    let contest_data: YukiContest;

    loop {
        let tmp_contest_id: String = Input::with_theme(&ColorfulTheme::default())
            .with_prompt(format!("{} コンテストID", Message::Input))
            .interact_text()?;

        let api_url = format!("https://yukicoder.me/api/v1/contest/id/{}", tmp_contest_id);

        // ここはエラーが起きてもユーザの入力ミスの可能性があるので、この場でリトライする
        match get_request(&api_url) {
            Ok(res) => {
                match serde_json::from_value(res) {
                    Ok(data) => contest_data = data,
                    Err(_) => return Err(Box::new(InvalidJsonError)),
                }
                contest_id = tmp_contest_id;
                break;
            }
            Err(error) => {
                // エラーでもメッセージだけ表示して終了せずに続行
                println!("{} {}", Message::RequestError, error);
            }
        }
    }

    println!("{} コンテスト名: {}", Message::Info, contest_data.name);
    println!(
        "{} 開始時刻: {}",
        Message::Info,
        contest_data.date.replace("T", " ")
    );

    if !Warning::Contest.start()? {
        return Ok(());
    }

    let contest_path_str = format!("{}/contest/yukicoder/{}", base_path()?, contest_id);

    let contest_path = Path::new(&contest_path_str);
    if !check_path(contest_path)? {
        return Ok(());
    }

    let mut problem_file = File::create(contest_path.join("problems.txt"))?;

    for id in &contest_data.problem_id_list {
        problem_file.write_all((id.to_string() + "\n").as_bytes())?;
    }

    // 問題の個数
    let num = contest_data.problem_id_list.len() as u8;
    make_path(contest_path, num)?;

    println!("{} コンテストの構成に成功しました", Message::Success);

    set_cd_clipboard(contest_path)?;

    check_login("https://yukicoder.me/")?;

    return Ok(());
}

fn virtual_contest() -> Result<(), Box<dyn Error>> {
    // 本番用/コンテスト一覧の取得
    let api_url = "https://kenkoooo.com/atcoder/internal-api/contest/recent";
    let contests: Vec<RecentVirtualContest>;

    match get_request(api_url) {
        Ok(res) => {
            match serde_json::from_value(res) {
                Ok(data) => contests = data,
                Err(_) => return Err(Box::new(InvalidJsonError)),
            }
        }
        Err(error) => {
            println!("{} コンテスト一覧の取得に失敗しました", Message::Failed);
            return Err(Box::new(error));
        }
    };

    let choices = &["1:環境変数", "2:IDから検索"];
    let choice = Select::with_theme(&ColorfulTheme::default())
        .with_prompt(format!("{} 検索方法を選択してください", Message::Question))
        .items(choices)
        .default(0)
        .interact()?;

    let mut contest_data: Option<RecentVirtualContest> = None;

    if choice == 0 {
        // 環境変数から取り出す
        let contest_name = match env::var("AC_CONTEST_NAME") {
            Ok(val) => val,
            Err(_) => {
                return Err(Box::new(EnvVarError(Borrowed("contest_name"))));
            }
        };

        let contest_name_vec: Vec<&str> = contest_name.split(" ").collect();

        // vectorに要素が二つ以上ある場合のみ実行
        let mut choice = 0;
        if contest_name_vec.len() > 1 {
            choice = Select::with_theme(&ColorfulTheme::default())
                .with_prompt(format!(
                    "{} 検索するコンテスト名を選択してください",
                    Message::Question
                ))
                .items(&contest_name_vec)
                .default(0)
                .interact()?;
        }
        let contest_name: &str = contest_name_vec.get(choice).ok_or(IndexError)?;

        // 指定したコンテスト名が含まれているものがあるか確認
        for contest in contests {
            if contest.title.contains(contest_name) {
                contest_data = Some(contest);
                break;
            }
        }
    } else {
        let contest_id: String = Input::with_theme(&ColorfulTheme::default())
            .with_prompt(format!("{} コンテストID:", Message::Input))
            .interact_text()?;

        for contest in contests {
            if contest.id == contest_id {
                contest_data = Some(contest);
                break;
            }
        }
    }

    let contest_data = match contest_data {
        Some(val) => val,
        None => {
            return Err(Box::new(ContestNotFoundError));
        }
    };

    let datetime = match Local.timestamp_opt(contest_data.start_epoch_second, 0) {
        // 現地時間が複数ある場合は早いほうと遅いほうの二つが返ってくる可能性がある(LocalResult::Ambiguous)
        // JSTで使う限りは関係なし
        LocalResult::Single(val) => val,
        LocalResult::Ambiguous(val, _) => val,
        LocalResult::None => {
            return Err("エポック秒の変換に失敗しました".into());
        }
    };

    println!("{} コンテスト名: {}", Message::Info, contest_data.title);
    println!("{} 開始時刻: {}", Message::Info, datetime);

    if !Warning::Contest.start()? {
        return Ok(());
    }

    // 本番用/問題情報の取得
    let api_url = format!(
        "https://kenkoooo.com/atcoder/internal-api/contest/get/{}",
        contest_data.id
    );

    let contest_data: VirtualContest;
    match get_request(&api_url) {
        Ok(res) => match serde_json::from_value(res) {
            Ok(data) => contest_data = data,
            Err(_) => return Err(Box::new(InvalidJsonError)),
        },
        Err(error) => {
            println!("{} 問題情報の取得に失敗しました", Message::Failed);
            return Err(Box::new(error));
        }
    };

    let contest_path_str = format!("{}/contest/virtual", base_path()?);
    let contest_path = Path::new(&contest_path_str);
    if !check_path(contest_path)? {
        return Ok(());
    }

    let mut problem_file = File::create(contest_path.join("virtual_problems.txt"))?;

    for problem in &contest_data.problems {
        // example: abc123_a
        let problem_data: Vec<&str> = problem.id.split("_").collect();

        if problem_data.len() != 2 {
            return Err(Box::new(UnsupportedContestError));
        }

        // 一連の操作で配列外参照を検知するようにはなっているが、"_"以前が3文字以上あればどんな文字列でも通してしまう
        // おそらく"3文字のコンテスト名+コンテスト番号"以外で使うことはない
        // 文字数+文字種で正しいフォーマットか検出することもできるが、結局検出にも限界があるし自分で使うだけならそこまでやる意味もなさそう

        let contest_name_id: &str = problem_data.get(0).ok_or(IndexError)?;
        let problem_id: &str = problem_data.get(1).ok_or(IndexError)?;

        // contest name
        if let Some(val) = contest_name_id.get(..3) {
            problem_file.write_all(val.as_bytes())?;
            problem_file.write_all(b"\n")?;
        } else {
            return Err(Box::new(UnsupportedContestError));
        }
        // contest id
        if let Some(val) = contest_name_id.get(3..) {
            problem_file.write_all(val.as_bytes())?;
            problem_file.write_all(b"\n")?;
        } else {
            return Err(Box::new(UnsupportedContestError));
        }
        // problem id
        problem_file.write_all(problem_id.as_bytes())?;
        problem_file.write_all(b"\n")?;
    }

    let num = contest_data.problems.len() as u8;
    make_path(contest_path, num)?;

    println!("{} コンテストの構成に成功しました", Message::Success);

    set_cd_clipboard(contest_path)?;

    return Ok(());
}

fn get_request(url: &str) -> Result<serde_json::Value, ureq::Error> {
    let response = ureq::get(&url).call()?;

    let res = response.into_json()?;
    return Ok(res);
}

fn check_login(url: &str) -> Result<(), Box<dyn Error>> {
    let args = vec!["login", url, "--check"];

    // ここでは出力は必要ないのでjudge::subprocessは使わない
    let res = Command::new("oj").args(args).output()?;

    if res.status.success() {
        println!("{} {} にログインしています", Message::Info, url);
    } else {
        println!("{} {} にログインしていません", Message::Warning, url);
    }

    return Ok(());
}

fn set_cd_clipboard(path: &Path) -> Result<(), Box<dyn Error>> {
    let mut clipboard = Clipboard::new()?;

    let text = String::from("cd ") + path.to_str().ok_or(InvalidUnicodeError)?;
    clipboard.set_text(text)?;

    println!(
        "{} 移動コマンドがクリップボードにコピーされました",
        Message::Success
    );

    return Ok(());
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use crate::gen::check_path;

    // ToDo: 標準入力が必要な部分を追加する
    #[test]
    #[should_panic]
    fn test_check_path() {
        let path = Path::new("/");
        check_path(path).unwrap();
    }

    #[test]
    #[should_panic]
    fn test_make_path() {
        let path = Path::new("/test");
        check_path(path).unwrap();
    }
}
