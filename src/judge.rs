use std::{
    borrow::Cow::{self, Borrowed, Owned},
    env,
    error::Error,
    fs::{self, File},
    io::{BufRead, BufReader, Read},
    path::Path,
    process::{Command, Output, Stdio},
};

use regex::Regex;

use crate::{submission, val::base_path, CustomError::*, Message};

pub fn run(
    testcase_path_str: String,
    problem_url: String,
    file_name: &str,
    auto: bool,
    debug: bool,
) -> Result<(), Box<dyn Error>> {
    // コンパイル
    {
        // 実行コマンドの引数生成
        let cmd;
        let mut args_cow;
        if debug {
            (cmd, args_cow) = base_commands(&file_name, "oj_nodebug_all", false)?;
        } else {
            (cmd, args_cow) = base_commands(&file_name, "oj_all", false)?;
            args_cow.append(&mut option_commands()?);
        }

        // Vec<Cow<str>>->Vec<&str>
        let args = args_cow.iter().map(|i| i.as_ref()).collect();

        // 実行
        subprocess(&cmd, args)?;
    }

    // ジャッジ
    let res;
    {
        // 少数判定用にサンプル取得
        let testcase_path = Path::new(&testcase_path_str);
        if !testcase_path.is_dir() {
            let args = vec!["d", &problem_url, "-d", &testcase_path_str];
            subprocess("oj", args)?;
        }

        // サンプル読み込み
        let mut testcase_str = String::new();
        match File::open(testcase_path.join("sample-1.out")) {
            Ok(mut testcase_file) => {
                testcase_file.read_to_string(&mut testcase_str)?;
            }
            Err(_) => {
                // サンプルがない問題も存在する
                // エラーメッセージはojのほうで出力されるので省略

                // URL書き出し
                fs::write(
                    format!("{}/library/url_latest.txt", base_path()?),
                    (problem_url + "\n" + &file_name).as_bytes(),
                )?;

                return Ok(());
            }
        }

        // スペース区切りで読み込み
        let testcase_str_vec: Vec<&str> = testcase_str.trim().split_whitespace().collect();

        // テスト用コマンド引数生成
        let file_name_path = format!("./{}", file_name);
        let mut args = vec![
            "t",
            "-N",
            "-c",
            &file_name_path,
            "-d",
            &testcase_path_str,
            "-D",
        ];

        // 少数判定
        let re = Regex::new(r"^[+-]?[0-9]+\.[0-9]+$")?;
        if re.is_match(testcase_str_vec.get(0).ok_or(IndexError)?) {
            args.push("-e");
            args.push("1e-6");
        }

        // ジャッジ実行
        res = subprocess("oj", args)?;
    }

    // 提出/submit
    if res.status.success() && auto {
        // atcoderのみ対応
        if problem_url.contains("atcoder.jp") && env::var("AC_USE_OJ").is_err() {
            // URLからコンテスト情報の取り出し(例:"abc123_a")
            let problem_data_vec: Vec<&str> = problem_url.split("/").collect();
            let mut problem_data = problem_data_vec
                .get(problem_data_vec.len() - 1)
                .ok_or(IndexError)?
                .split("_");

            submission::run(
                problem_data.next().ok_or(IndexError)?,
                problem_data.next().ok_or(IndexError)?,
                env::current_dir()?.join(format!("{}.cpp", &file_name)),
                0,
            )?;
        } else {
            if problem_url.contains("codeforces.com") {
                println!(
                    "{} Codeforcesへの自動提出には対応していません",
                    Message::Failed
                );
            } else {
                let file_name_cpp = format!("{}.cpp", file_name);
                let args = vec!["s", &problem_url, &file_name_cpp, "-y", "-w", "0"];
                subprocess("oj", args)?;
            }
        }
    }

    // URL書き出し
    fs::write(
        format!("{}/library/url_latest.txt", base_path()?),
        (problem_url + "\n" + &file_name).as_bytes(),
    )?;

    return Ok(());
}

pub fn base_commands<'a>(
    file_name: &'a str,
    header_name: &str,
    local: bool,
) -> Result<(String, Vec<Cow<'a, str>>), Box<dyn Error>> {
    // 存在確認
    {
        let testcase_path = env::current_dir()?;
        if !testcase_path.join(format!("{}.cpp", file_name)).is_file() {
            return Err(Box::new(FileNotfoundError(Owned(format!(
                "{}.cpp",
                file_name
            )))));
        }
    }

    let commands_file = fs::File::open(format!("{}/setting/cpp.txt", base_path()?))
        .or_else(|_| Err(Box::new(FileNotfoundError(Borrowed("cpp.txt")))))?;
    let commands_file_buf = BufReader::new(commands_file).lines();

    let base_path_str = base_path()?;
    let mut res_cmd = String::new();
    let mut res_vec: Vec<Cow<str>> = Vec::new();
    for (index, val) in commands_file_buf.into_iter().enumerate() {
        let cmd = val?
            .replace("{FILE_NAME}", file_name)
            .replace("{BASE_PATH}", &base_path_str)
            .replace("{HEADER_NAME}", header_name);

        if index == 0 {
            res_cmd = cmd;
        } else {
            res_vec.push(Owned(cmd));
        }
    }

    if local {
        res_vec.push(Borrowed("-DLOCAL"));
    }

    return Ok((res_cmd, res_vec));
}

pub fn option_commands<'a>() -> Result<Vec<Cow<'a, str>>, Box<dyn Error>> {
    let options_file = fs::File::open(format!("{}/setting/cpp_options.txt", base_path()?))
        .or_else(|_| Err(Box::new(FileNotfoundError(Borrowed("cpp_options.txt")))))?;
    let options_file_buf = BufReader::new(options_file).lines();

    let mut res = Vec::new();
    for i in options_file_buf {
        res.push(Owned(i?));
    }

    return Ok(res);
}

pub fn header_commands(
    file_name: &str,
    local: bool,
) -> Result<(String, Vec<Cow<str>>), Box<dyn Error>> {
    // 存在確認
    let header_path_str = format!("{}/library/header/{}.hpp", base_path()?, file_name);
    if !Path::new(&header_path_str).is_file() {
        return Err(Box::new(FileNotfoundError(Owned(format!(
            "{}.hpp",
            file_name
        )))));
    }

    let header_file = fs::File::open(format!("{}/setting/cpp_header.txt", base_path()?))
        .or_else(|_| Err(Box::new(FileNotfoundError(Borrowed("cpp_header.txt")))))?;
    let header_file_buf = BufReader::new(header_file).lines();

    let base_path_str = base_path()?;
    let mut res_cmd = String::new();
    let mut res_vec: Vec<Cow<str>> = Vec::new();
    for (index, val) in header_file_buf.into_iter().enumerate() {
        let cmd = val?
            .replace("{FILE_NAME}", file_name)
            .replace("{BASE_PATH}", &base_path_str)
            .replace("{HEADER_PATH}", &header_path_str);
        if index == 0 {
            res_cmd = cmd;
        } else {
            res_vec.push(Owned(cmd));
        }
    }

    if local {
        res_vec.push(Borrowed("-DLOCAL"));
    }

    return Ok((res_cmd, res_vec));
}

pub fn ac_converter(
    contest_name: &str,
    contest_id: &str,
    problem_id: &str,
) -> Result<(String, String), Box<dyn Error>> {
    let testcase_path_str = format!(
        "{}/test/{}/{}/{}",
        base_path()?,
        contest_name,
        contest_id,
        problem_id
    );
    let problem_url = format!(
        "https://atcoder.jp/contests/{}{}/tasks/{}{}_{}",
        contest_name, contest_id, contest_name, contest_id, problem_id
    );

    return Ok((testcase_path_str, problem_url));
}

pub fn cf_converter(
    contest_id: &str,
    problem_id: &str,
) -> Result<(String, String), Box<dyn Error>> {
    let testcase_path_str = format!(
        "{}/test/codeforces/{}/{}",
        base_path()?,
        contest_id,
        problem_id
    );
    let problem_url = format!(
        "https://codeforces.com/contest/{}/problem/{}",
        contest_id,
        problem_id.to_ascii_uppercase()
    );

    return Ok((testcase_path_str, problem_url));
}

pub fn yuki_converter(
    problem_id: &str,
    is_contest: bool,
) -> Result<(String, String), Box<dyn Error>> {
    let testcase_path_str = format!("{}/test/yukicoder/{}", base_path()?, problem_id);

    let problem_url;
    if is_contest {
        problem_url = format!("https://yukicoder.me/problems/{}", problem_id);
    } else {
        problem_url = format!("https://yukicoder.me/problems/no/{}", problem_id);
    }

    return Ok((testcase_path_str, problem_url));
}

pub fn subprocess(command: &str, args: Vec<&str>) -> Result<Output, Box<dyn Error>> {
    return Ok(Command::new(command)
        .args(args)
        .stdin(Stdio::inherit())
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .output()?);
}
