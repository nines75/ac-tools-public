use std::{borrow::Cow, error::Error, fmt};

use colored::Colorize;
use dialoguer::{theme::ColorfulTheme, Confirm};
use thiserror::Error;
use CustomError::*;

pub mod judge;
pub mod submission;

pub mod val {
    use std::{
        borrow::Cow::{Borrowed, Owned},
        env,
        error::Error,
        path::{Path, PathBuf},
    };

    use crate::CustomError::*;

    /// return path of template.cpp
    pub fn template_path() -> Result<PathBuf, Box<dyn Error>> {
        let path_str = format!("{}/template.cpp", base_path()?);
        let path = PathBuf::from(path_str);

        if !path.is_file() {
            return Err(Box::new(FileNotfoundError(Borrowed("template.cpp"))));
        }

        return Ok(path);
    }

    /// return tuple (current dir name, parent dir name)
    pub fn path_name(path: &Path) -> Result<(String, String), Box<dyn Error>> {
        let current_dir_name = path
            .file_name()
            .ok_or(DirNotfoundError(Owned(path_str(path)?)))?;
        let parent_dir_name = path
            .parent()
            .ok_or(DirNotfoundError(Owned(path_str(path)?)))?
            .file_name()
            .ok_or(DirNotfoundError(Owned(path_str(path)?)))?;

        return Ok((
            current_dir_name
                .to_str()
                .ok_or(InvalidUnicodeError)?
                .to_string(),
            parent_dir_name
                .to_str()
                .ok_or(InvalidUnicodeError)?
                .to_string(),
        ));
    }

    // to_strはあくまでpathの参照なのでpathのスコープ内でしか生存できないから、CustomErrorに渡すにはStringにする必要がある
    fn path_str(path: &Path) -> Result<String, Box<dyn Error>> {
        return Ok(path.to_str().ok_or(InvalidUnicodeError)?.to_string());
    }

    pub fn base_path() -> Result<String, Box<dyn Error>> {
        match env::var("AC_BASE_PATH") {
            Ok(val) => return Ok(val.replace("~", shellexpand::tilde("~").as_ref())),
            Err(_) => {
                return Err(Box::new(EnvVarError(Borrowed("AC_BASE_PATH"))));
            }
        }
    }
}

pub enum Message {
    Question,
    Select,
    Error,
    Warning,
    Input,
    Success,
    Info,
    Failed,
    RequestError,
}

impl fmt::Display for Message {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::Question => write!(f, "[{}]", "QUESTION".bright_blue()),
            Self::Select => write!(f, "[{}]", "SELECT".green()),
            Self::Error => write!(f, "[{}]", "ERROR".bright_red()),
            Self::Warning => write!(f, "[{}]", "WARNING".bright_yellow()),
            Self::Input => write!(f, "[{}]", "INPUT".bright_purple()),
            Self::Success => write!(f, "[{}]", "SUCCESS".bright_green()),
            Self::Info => write!(f, "[{}]", "INFO".bright_cyan()),
            Self::Failed => write!(f, "[{}]", "FAILED".bright_red()),
            Self::RequestError => write!(f, "[{}]", "RequestError".bright_red()),
        }
    }
}

pub enum Warning {
    Contest,
    Overwrite(String),
    Precompile,
}

impl Warning {
    pub fn start(self) -> Result<bool, Box<dyn Error>> {
        let mut text = format!("{} ", Message::Warning);
        match self {
            Self::Contest => {
                text += "コンテストの構成を実行しますか";
            }
            Self::Overwrite(s) => {
                text += &format!("{} は既に存在しますが、上書きしますか", s);
            }
            Self::Precompile => {
                text += "プリコンパイル済みヘッダを生成しますか";
            }
        }

        let res = Confirm::with_theme(&ColorfulTheme::default())
            .with_prompt(text)
            .interact()?;

        return Ok(res);
    }
}

#[derive(Error)]
pub enum CustomError<'a> {
    #[error("InvalidJsonError")]
    InvalidJsonError,

    #[error("DirNotfoundError")]
    DirNotfoundError(Cow<'a, str>),

    #[error("FileNotfoundError")]
    FileNotfoundError(Cow<'a, str>),

    #[error("InvalidUnicodeError")]
    InvalidUnicodeError,

    #[error("IndexError")]
    IndexError,

    #[error("HtmlError")]
    HtmlError,

    #[error("InvalidCookieError")]
    InvalidCookieError,

    #[error("EnvVarError")]
    EnvVarError(Cow<'a, str>),

    #[error("TooManyArgError")]
    TooManyArgError,

    #[error("TooFewArgError")]
    TooFewArgError,

    #[error("ContestNotFoundError")]
    ContestNotFoundError,

    #[error("UnsupportedContestError")]
    UnsupportedContestError,
}

impl<'a> fmt::Debug for CustomError<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mut res = self.to_string() + ": ";
        match self {
            InvalidJsonError => res += "json形式のデータからの値の抽出に失敗しました",
            DirNotfoundError(path) => res += &format!("ディレクトリが存在しませんでした({})", path),
            FileNotfoundError(file) => res += &format!("ファイルが存在しませんでした({})", file),
            InvalidUnicodeError => res += "不正なバイト列が検出されました",
            IndexError => res += "配列の範囲外を参照しました",
            HtmlError => res += "要素が存在しませんでした",
            InvalidCookieError => res += "クッキーの抽出に失敗しました",
            EnvVarError(var) => res += &format!("環境変数'{}'が設定されていません", var),
            TooManyArgError => res += "引数が多すぎます",
            TooFewArgError => res += "引数が不足しています",
            ContestNotFoundError => res += "コンテストが見つかりませんでした",
            UnsupportedContestError => res += "対応していない種類のコンテストです",
        }
        return write!(f, "{}", res);
    }
}
