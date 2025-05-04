use std::error::Error;

use ac_tools_rs::judge::{self, yuki_converter};

pub fn run(problem_id: String, auto: bool, debug: bool) -> Result<(), Box<dyn Error>> {
    let (testcase_path_str, problem_url) = yuki_converter(&problem_id, false)?;

    judge::run(testcase_path_str, problem_url, "main", auto, debug)?;

    return Ok(());
}
