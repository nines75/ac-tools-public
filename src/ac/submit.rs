use std::{
    env,
    error::Error,
    fs,
    io::{BufRead, BufReader},
};

use ac_tools_rs::{judge::subprocess, submission, val::base_path, CustomError::*, Message};

pub fn run(oj: bool) -> Result<(), Box<dyn Error>> {
    // BufReaderを使ったほうが効率がよい
    // https://doc.rust-jp.rs/rust-by-example-ja/std_misc/file/read_lines.html
    let problem_file = fs::File::open(format!("{}/library/url_latest.txt", base_path()?))?;
    let mut problem_file_buf = BufReader::new(problem_file).lines();

    let problem_url;
    let file_name;
    // get index 0
    match problem_file_buf.next() {
        Some(val) => problem_url = val?,
        None => return Err(Box::new(IndexError)),
    }
    // get index 1
    match problem_file_buf.next() {
        Some(val) => file_name = val?,
        None => return Err(Box::new(IndexError)),
    }

    if problem_url.contains("codeforces.com") {
        println!(
            "{} Codeforcesへの自動提出には対応していません",
            Message::Failed
        );
        return Ok(());
    }

    if !oj && problem_url.contains("atcoder.jp") && env::var("AC_USE_OJ").is_err() {
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
        let file_name_cpp = format!("{}.cpp", file_name);
        let args = vec!["s", &problem_url, &file_name_cpp, "-y", "-w", "0"];
        subprocess("oj", args)?;
    }

    return Ok(());
}
