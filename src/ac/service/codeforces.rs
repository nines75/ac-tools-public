use std::error::Error;

use ac_tools_rs::judge::{self, cf_converter};

pub fn run(
    contest_id: String,
    problem_id: String,
    auto: bool,
    debug: bool,
) -> Result<(), Box<dyn Error>> {
    let (testcase_path_str, problem_url) = cf_converter(&contest_id, &problem_id)?;

    judge::run(testcase_path_str, problem_url, "main", auto, debug)?;

    return Ok(());
}
