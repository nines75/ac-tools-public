use std::{
    borrow::Cow::Borrowed,
    error::Error,
    process::{Command, Stdio},
};

use ac_tools_rs::{
    judge::{header_commands, option_commands},
    CustomError::*,
    Message, Warning,
};
use colored::Colorize;
use rayon::iter::{IntoParallelIterator, ParallelIterator};

pub fn run() -> Result<(), Box<dyn Error>> {
    if !Warning::Precompile.start()? {
        return Ok(());
    }

    let headers = vec![
        "all",
        "debug_all",
        "nodebug_all",
        "oj_all",
        "oj_nodebug_all",
    ];

    let mut args_vec = Vec::new();

    // all.hpp
    let (cmd, mut args) = header_commands("all", true)?;
    args.append(&mut option_commands()?);
    args_vec.push(args);

    // debug_all.hpp
    let (_, mut args) = header_commands("debug_all", true)?;
    args.push(Borrowed("-g"));
    args_vec.push(args);

    // nodebug_all.hpp
    let (_, args) = header_commands("nodebug_all", true)?;
    args_vec.push(args);

    // oj_all.hpp
    let (_, mut args) = header_commands("oj_all", false)?;
    args.append(&mut option_commands()?);
    args_vec.push(args);

    // oj_nodebug_all.hpp
    let (_, args) = header_commands("oj_nodebug_all", false)?;
    args_vec.push(args);

    // スレッドに入れていく
    match (0..args_vec.len()).into_par_iter().try_for_each(|i| {
        // Vec<Cow<str>->Vec<&str>
        let args = args_vec
            .get(i)
            .ok_or(IndexError)?
            .iter()
            .map(|i| i.as_ref())
            .collect();

        subprocess(&cmd, args, headers.get(i).ok_or(IndexError)?)
    }) {
        Ok(_) => {}
        // エラーの型が違うのでStringにしてから変換する必要がある
        Err(error) => return Err(error.to_string().into()),
    }

    return Ok(());
}

// 非同期処理は戻り値を変えないといけないのでjudge.rsとは別に必要
fn subprocess(
    command: &str,
    args: Vec<&str>,
    header_name: &str,
) -> Result<(), Box<dyn Error + Send + Sync>> {
    println!(
        "{} {}.hppをコンパイルしています...",
        Message::Info,
        header_name.bold()
    );

    Command::new(command)
        .args(args)
        .stdin(Stdio::inherit())
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .output()?;

    println!(
        "{} {}.hppのコンパイルに成功しました✅",
        Message::Success,
        header_name.bold()
    );

    return Ok(());
}
