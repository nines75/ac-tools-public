use std::{
    borrow::Cow::Borrowed,
    env,
    error::Error,
    fs,
    io::{stdin, Read},
    path::Path,
};

use ac_tools_rs::{
    val::{self, base_path},
    CustomError::*,
    Message,
};
use dialoguer::{theme::ColorfulTheme, Input, Select};

pub fn run() -> Result<(), Box<dyn Error>> {
    let current_dir = env::current_dir()?;
    let (current_dir_name, parent_dir_name) = val::path_name(&current_dir)?;

    if !(parent_dir_name == "codeforces") {
        println!(
            "{} Codeforces以外のコンテストのテストケース作成はできません",
            Message::Failed
        );
        return Ok(());
    }

    let problem_alphabet: String = Input::with_theme(&ColorfulTheme::default())
        .with_prompt(format!("{} 問題ID", Message::Input))
        .interact_text()?;

    let choices = &["1:追加", "2:削除"];
    let choice = Select::with_theme(&ColorfulTheme::default())
        .with_prompt(format!(
            "{} コンテストの種類を選択してください",
            Message::Question
        ))
        .items(choices)
        .default(0)
        .interact()?;

    // add
    if choice == 0 {
        let testcase_path_str = format!(
            "{}/test/codeforces/{}/{}",
            base_path()?,
            current_dir_name,
            problem_alphabet
        );
        let testcase_path = Path::new(&testcase_path_str);

        if !testcase_path.is_dir() {
            fs::create_dir_all(testcase_path)?;
        }

        // 現在のファイル数を数える
        let cnt = fs::read_dir(&testcase_path)?.count();

        let input_file_name = format!("sample-{}.in", cnt / 2 + 1);
        let output_file_name = format!("sample-{}.out", cnt / 2 + 1);

        // write input
        {
            println!("{}", "標準入力を入力してください(Ctrl+Dで終了)");
            let mut input = Vec::new();
            stdin().read_to_end(&mut input)?;

            fs::write(testcase_path.join(input_file_name), input)?;
        }

        // write output
        {
            println!("{}", "標準出力を入力してください(Ctrl+Dで終了)");
            let mut input = Vec::new();
            stdin().read_to_end(&mut input)?;

            fs::write(testcase_path.join(output_file_name), input)?;
        }

        println!("{} テストケースの作成に成功しました", Message::Success);
    }

    // delete
    if choice == 1 {
        let testcase_path_str = format!(
            "{}/test/codeforces/{}/{}",
            base_path()?,
            current_dir_name,
            problem_alphabet
        );
        let testcase_path = Path::new(&testcase_path_str);

        if !testcase_path.is_dir() || fs::read_dir(&testcase_path)?.count() == 0 {
            return Err(Box::new(FileNotfoundError(Borrowed("testcase"))));
        }

        let testcase_files = fs::read_dir(&testcase_path)?;
        let cnt_str = (fs::read_dir(&testcase_path)?.count() / 2).to_string();

        // 最後に追加されたテストケースの入出力のみ削除する
        for file in testcase_files {
            let file_name = file?.file_name();
            let file_name_str = file_name.to_str().ok_or(InvalidUnicodeError)?;
            if file_name_str.contains(&cnt_str) {
                println!("{} {}を削除しています...", Message::Info, file_name_str);
                fs::remove_file(testcase_path.join(file_name))?;
            }
        }

        println!("{} テストケースの削除に成功しました", Message::Success);
    }

    return Ok(());
}
