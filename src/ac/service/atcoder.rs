use std::error::Error;

use ac_tools_rs::judge::{self, ac_converter};

pub fn run(
    contest_name: String,
    contest_id: String,
    problem_id: String,
    auto: bool,
    debug: bool,
) -> Result<(), Box<dyn Error>> {
    let (testcase_path_str, problem_url) = ac_converter(&contest_name, &contest_id, &problem_id)?;

    judge::run(testcase_path_str, problem_url, "main", auto, debug)?;

    return Ok(());
}
