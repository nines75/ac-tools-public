use std::error::Error;

use ac_tools_rs::{judge, val::base_path, CustomError::*};

pub fn run(
    arg1: String,
    arg2: Option<String>,
    custom: bool,
    auto: bool,
    debug: bool,
) -> Result<(), Box<dyn Error>> {
    let problem_url;
    let contest_name_id;
    let testcase_path_str;

    if custom {
        contest_name_id = arg1;
        let problem_id = match arg2 {
            Some(val) => val,
            None => {
                return Err(Box::new(TooFewArgError));
            }
        };

        problem_url = format!(
            "https://atcoder.jp/contests/{}/tasks/{}_{}",
            contest_name_id, contest_name_id, problem_id
        );

        testcase_path_str = format!(
            "{}/test/custom/{}/{}",
            base_path()?,
            contest_name_id,
            problem_id
        );
    }
    // url
    else {
        problem_url = arg1;

        testcase_path_str = format!("{}/test/url", base_path()?);
    }

    judge::run(testcase_path_str, problem_url, "main", auto, debug)?;

    return Ok(());
}
