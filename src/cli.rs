use clap::Parser;
use std::path::PathBuf;

#[derive(Parser)]
#[command(author, version, about = "Remove node_modules directories")]
pub struct Args {
    /// 起始目录
    #[arg(short, long)]
    pub dir: Option<PathBuf>,

    /// 递归深度 (0 = 无限)
    #[arg(long, default_value = "0")]
    pub depth: usize,

    /// 自动确认删除
    #[arg(short, long)]
    pub yes: bool,

    /// 静默模式，不输出日志
    #[arg(short, long)]
    pub quiet: bool,

    /// 测试模式：不删除，输出详细日志
    #[arg(short, long)]
    pub test: bool,
}

pub fn parse_args() -> Args {
    Args::parse()
}