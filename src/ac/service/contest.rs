use std::{
    borrow::Cow::Borrowed,
    env,
    error::Error,
    fs::File,
    io::{BufRead, BufReader},
};

use ac_tools_rs::{
    judge::{self, ac_converter, cf_converter, yuki_converter},
    val,
    CustomError::*,
};

pub fn run(problem_alphabet: String, auto: bool, debug: bool) -> Result<(), Box<dyn Error>> {
    let current_dir = env::current_dir()?;
    let (current_dir_name, parent_dir_name) = val::path_name(&current_dir)?;

    if parent_dir_name == "codeforces" {
        codeforces(problem_alphabet, auto, debug)?;
    } else if parent_dir_name == "yukicoder" {
        yukicoder(problem_alphabet, auto, debug)?;
    } else if current_dir_name == "virtual" {
        virtual_contest(problem_alphabet, auto, debug)?;
    } else {
        atcoder(problem_alphabet, auto, debug)?;
    }

    return Ok(());
}

// コンテストごとの各関数の引数の命名規則
// atcoder/codeforces -> problem_alphabet=contest_idとなるので、引数の時点でproblem_idでok
// yukicoder/virtual -> ファイルから取得して変換が必要なのでproblem_alphabetのままにする

fn atcoder(problem_id: String, auto: bool, debug: bool) -> Result<(), Box<dyn Error>> {
    // ディレクトリ名取得
    let current_dir = env::current_dir()?;
    let (contest_id, contest_name) = val::path_name(&current_dir)?;

    let (testcase_path_str, problem_url) = ac_converter(&contest_name, &contest_id, &problem_id)?;
    judge::run(testcase_path_str, problem_url, &problem_id, auto, debug)?;

    return Ok(());
}

fn codeforces(problem_id: String, auto: bool, debug: bool) -> Result<(), Box<dyn Error>> {
    let current_dir = env::current_dir()?;
    let (contest_id, _) = val::path_name(&current_dir)?;

    let (testcase_path_str, problem_url) = cf_converter(&contest_id, &problem_id)?;
    judge::run(testcase_path_str, problem_url, &problem_id, auto, debug)?;

    return Ok(());
}

fn yukicoder(problem_alphabet: String, auto: bool, debug: bool) -> Result<(), Box<dyn Error>> {
    // ファイル読み込み
    let current_dir = env::current_dir()?;
    let problem_file = File::open(current_dir.join("problems.txt"))
        .or_else(|_| Err(Box::new(FileNotfoundError(Borrowed("problems.txt")))))?;
    let mut problems = Vec::new();
    for i in BufReader::new(problem_file).lines() {
        problems.push(i?);
    }

    // アルファベットから何個目の問題かを調べる
    let base_char = 'a'.to_ascii_lowercase() as usize;
    let index = problem_alphabet
        .chars()
        .nth(0)
        .ok_or(IndexError)?
        .to_ascii_lowercase() as usize
        - base_char;
    let problem_id = problems.get(index).ok_or(IndexError)?;

    let (testcase_path_str, problem_url) = yuki_converter(problem_id, true)?;
    judge::run(
        testcase_path_str,
        problem_url,
        &problem_alphabet,
        auto,
        debug,
    )?;

    return Ok(());
}

fn virtual_contest(
    problem_alphabet: String,
    auto: bool,
    debug: bool,
) -> Result<(), Box<dyn Error>> {
    // ファイル読み込み
    let current_dir = env::current_dir()?;
    let problem_file = File::open(current_dir.join("virtual_problems.txt")).or_else(|_| {
        Err(Box::new(FileNotfoundError(Borrowed(
            "virtual_problems.txt",
        ))))
    })?;
    let mut problems: Vec<String> = Vec::new();
    for i in BufReader::new(problem_file).lines() {
        problems.push(i?);
    }

    // アルファベットから何個目の問題かを調べる
    let base_char = 'a'.to_ascii_lowercase() as usize;
    let index = problem_alphabet
        .chars()
        .nth(0)
        .ok_or(IndexError)?
        .to_ascii_lowercase() as usize
        - base_char;
    let contest_name = problems.get(index * 3).ok_or(IndexError)?;
    let contest_id = problems.get(index * 3 + 1).ok_or(IndexError)?;
    let problem_id = problems.get(index * 3 + 2).ok_or(IndexError)?;

    let (testcase_path_str, problem_url) = ac_converter(contest_name, contest_id, problem_id)?;
    judge::run(
        testcase_path_str,
        problem_url,
        &problem_alphabet,
        auto,
        debug,
    )?;

    return Ok(());
}
