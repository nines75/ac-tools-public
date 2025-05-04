use std::error::Error;

use ac_tools_rs::Message;
use clap::{Parser, Subcommand};

mod debug;
mod gen;
mod init;
mod nodebug;
mod precompile;
mod service;
mod submit;
mod test;
mod testcase;

#[derive(Debug, Parser)]
#[command(
    version = "2.0.0",
    about = "A support tool for competitive programming in C++"
)]
struct Cli {
    #[clap(subcommand)]
    command: Commands,
}

#[derive(Debug, Subcommand)]
#[allow(non_camel_case_types)]
enum Commands {
    #[clap(
        visible_alias("t"),
        override_usage("ac test(t) [OPTIONS] <URL/コンテストID> <問題ID>")
    )]
    test {
        arg1: String,
        arg2: Option<String>,

        #[arg(short = 'c', long = "custom", action)]
        custom: bool,

        #[arg(short = 'a', long = "auto", action)]
        auto: bool,

        #[arg(short = 'd', long = "debug", action)]
        debug: bool,
    },

    #[clap(visible_alias("g"))]
    gen {},

    #[clap(visible_alias("p"))]
    precompile {},

    #[clap(visible_alias("m"))]
    testcase {},

    #[clap(visible_alias("s"))]
    submit {
        #[arg(short = 'o', long = "oj", action)]
        oj: bool,
    },

    #[clap(visible_alias("d"))]
    debug { file_name: String },

    #[clap(visible_alias("n"))]
    nodebug { file_name: String },

    #[clap(hide = true)]
    __init {
        #[arg(short = 'f', long = "fish", action)]
        fish: bool,

        #[arg(short = 'd', long = "dev", action)]
        dev: bool,
    },

    #[clap(
        hide = true,
        override_usage("<問題ID> [OPTIONS]\n利用可能な問題ID: a, b, c, d, e, f, g, h")
    )]
    __contest {
        #[arg(hide = true)]
        problem_alphabet: String,

        #[arg(short = 'a', long = "auto", action)]
        auto: bool,

        #[arg(short = 'd', long = "debug", action)]
        debug: bool,
    },

    #[clap(
        hide = true,
        override_usage("<コンテスト名> [OPTIONS] <コンテスト番号> <問題ID>\n利用可能なコンテスト名: abc, arc, agc")
    )]
    __atcoder {
        #[arg(hide = true)]
        contest_name: String,
        contest_id: String,
        problem_id: String,

        #[arg(short = 'a', long = "auto", action)]
        auto: bool,

        #[arg(short = 'd', long = "debug", action)]
        debug: bool,
    },

    #[clap(hide = true, override_usage("cf [OPTIONS] <コンテストID> <問題ID>"))]
    __codeforces {
        contest_id: String,
        problem_id: String,

        #[arg(short = 'a', long = "auto", action)]
        auto: bool,

        #[arg(short = 'd', long = "debug", action)]
        debug: bool,
    },

    #[clap(hide = true, override_usage("yk [OPTIONS] <問題ID>"))]
    __yukicoder {
        problem_id: String,

        #[arg(short = 'a', long = "auto", action)]
        auto: bool,

        #[arg(short = 'd', long = "debug", action)]
        debug: bool,
    },
}

fn main() {
    let args = Cli::parse();

    let mut err: Option<Box<dyn Error>> = None;
    match args.command {
        Commands::test {
            arg1,
            arg2,
            custom,
            auto,
            debug,
        } => match test::run(arg1, arg2, custom, auto, debug) {
            Ok(_) => {}
            Err(error) => {
                err = Some(error);
            }
        },
        Commands::gen {} => match gen::run() {
            Ok(_) => {}
            Err(error) => {
                err = Some(error);
            }
        },
        Commands::precompile {} => match precompile::run() {
            Ok(_) => {}
            Err(error) => {
                err = Some(error);
            }
        },
        Commands::testcase {} => match testcase::run() {
            Ok(_) => {}
            Err(error) => {
                err = Some(error);
            }
        },
        Commands::submit { oj } => match submit::run(oj) {
            Ok(_) => {}
            Err(error) => {
                err = Some(error);
            }
        },
        Commands::debug { file_name } => match debug::run(file_name) {
            Ok(_) => {}
            Err(error) => {
                err = Some(error);
            }
        },
        Commands::nodebug { file_name } => match nodebug::run(file_name) {
            Ok(_) => {}
            Err(error) => {
                err = Some(error);
            }
        },
        Commands::__init { fish, dev } => match init::run(fish, dev) {
            Ok(_) => {}
            Err(error) => {
                err = Some(error);
            }
        },
        Commands::__contest {
            problem_alphabet,
            auto,
            debug,
        } => match service::contest::run(problem_alphabet, auto, debug) {
            Ok(_) => {}
            Err(error) => {
                err = Some(error);
            }
        },
        Commands::__atcoder {
            contest_name,
            contest_id,
            problem_id,
            auto,
            debug,
        } => match service::atcoder::run(contest_name, contest_id, problem_id, auto, debug) {
            Ok(_) => {}
            Err(error) => {
                err = Some(error);
            }
        },
        Commands::__codeforces {
            contest_id,
            problem_id,
            auto,
            debug,
        } => match service::codeforces::run(contest_id, problem_id, auto, debug) {
            Ok(_) => {}
            Err(error) => {
                err = Some(error);
            }
        },
        Commands::__yukicoder {
            problem_id,
            auto,
            debug,
        } => match service::yukicoder::run(problem_id, auto, debug) {
            Ok(_) => {}
            Err(error) => {
                err = Some(error);
            }
        },
    }

    match err {
        Some(error) => {
            println!("{} {:?}", Message::Error, error);
        }
        None => {}
    }
}
