use std::{
    borrow::Cow::Borrowed,
    error::Error,
    fs,
    path::{Path, PathBuf},
};

use ac_tools_rs::{
    judge::{base_commands, subprocess},
    val::base_path,
    CustomError::*,
};
use glob::glob;

pub fn run(file_name: String) -> Result<(), Box<dyn Error>> {
    // 既存のcoredumpファイルを削除
    {
        let coredump_path_str = format!("{}/tmp", base_path()?);
        let folder = fs::read_dir(Path::new(&coredump_path_str))?;
        for file in folder {
            fs::remove_file(file?.path())?;
        }
    }

    // コンパイル
    {
        let (cmd, mut args_cow) = base_commands(&file_name, "debug_all", true)?;
        args_cow.push(Borrowed("-g"));

        // Vec<Cow<str>>->Vec<&str>
        let args = args_cow.iter().map(|i| i.as_ref()).collect();

        subprocess(&cmd, args)?;
    }

    let file_name_path = format!("./{}", file_name);
    
    // リソース制限を解除して実行
    // この方法だと"Segmentation fault"のメッセージが出ない
    {
        let args = vec!["--core=unlimited", &file_name_path];
        subprocess("prlimit", args)?;
    }

    // gdb
    {
        // 存在すればcoredumpを取得
        let mut coredump: Option<PathBuf> = None;
        {
            let pattern = format!("{}/tmp/core.*", base_path()?);
            for file in glob(&pattern)? {
                coredump = Some(file?);
            }
        }

        match coredump {
            Some(coredump_path) => {
                let args: Vec<&str> = vec![
                    &file_name_path,
                    coredump_path.to_str().ok_or(InvalidUnicodeError)?,
                ];
                subprocess("gdb", args)?;
            }
            None => return Err(Box::new(FileNotfoundError(Borrowed("coredump")))),
        }
    }

    return Ok(());
}
