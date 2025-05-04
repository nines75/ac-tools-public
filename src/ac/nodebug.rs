use std::error::Error;

use ac_tools_rs::judge::{base_commands, subprocess};

pub fn run(file_name: String) -> Result<(), Box<dyn Error>> {
    // コンパイル
    {
        let (cmd, args_cow) = base_commands(&file_name, "nodebug_all", true)?;
        let args = args_cow.iter().map(|i| i.as_ref()).collect();
        subprocess(&cmd, args)?;
    }

    // 実行
    subprocess(&format!("./{}", file_name), Vec::new())?;

    return Ok(());
}
