use anyhow::Result;
use rayon::prelude::*;
use std::io::{self, Write};
use std::path::{Path, PathBuf};

/// 确认删除所有目录
pub fn confirm_all() -> Result<bool> {
    print!("\n是否删除以上所有目录? [y/N] ");
    io::stdout().flush()?;
    let mut input = String::new();
    io::stdin().read_line(&mut input)?;
    Ok(input.trim().eq_ignore_ascii_case("y"))
}

/// 快速删除目录：先原子改名让原路径立即"消失"，再按一级子目录并行删除
pub fn remove_dir_all_fast(path: &Path) -> Result<()> {
    // 改名是原子操作，可避免删除到一半时被 IDE/杀软重新占用；
    // 改名失败（跨设备、被占用等）则退回原地删除
    let tmp = tmp_delete_name(path);
    let target: PathBuf = match std::fs::rename(path, &tmp) {
        Ok(()) => tmp,
        Err(_) => path.to_path_buf(),
    };

    // 一级子目录之间并行删除，Windows 上对海量小文件提速明显；
    // symlink/junction 的 file_type().is_dir() 为 false，留给最后的
    // remove_dir_all 处理（只删链接本身，不会进入目标目录）
    let subdirs: Vec<PathBuf> = std::fs::read_dir(&target)?
        .flatten()
        .filter(|e| e.file_type().map(|t| t.is_dir()).unwrap_or(false))
        .map(|e| e.path())
        .collect();
    subdirs.par_iter().try_for_each(std::fs::remove_dir_all)?;

    // 清理顶层剩余的文件、链接和空目录本身
    std::fs::remove_dir_all(&target)?;
    Ok(())
}

/// 生成同目录下的临时删除名（隐藏文件风格，带 pid 防冲突）
fn tmp_delete_name(path: &Path) -> PathBuf {
    let name = path.file_name().and_then(|n| n.to_str()).unwrap_or("dir");
    path.with_file_name(format!(".{}.rnm-del-{}", name, std::process::id()))
}

/// 格式化文件大小
pub fn format_size(size: u64) -> String {
    const KB: u64 = 1024;
    const MB: u64 = KB * 1024;
    const GB: u64 = MB * 1024;

    if size >= GB {
        format!("{:.2} GB", size as f64 / GB as f64)
    } else if size >= MB {
        format!("{:.2} MB", size as f64 / MB as f64)
    } else if size >= KB {
        format!("{:.2} KB", size as f64 / KB as f64)
    } else {
        format!("{} B", size)
    }
}

/// 计算目录大小：一级子目录之间并行统计。
/// 大小仅用于展示，遇到无权限等错误按 0 处理，不中断流程
pub fn calculate_dir_size(path: &Path) -> u64 {
    let entries: Vec<_> = match std::fs::read_dir(path) {
        Ok(rd) => rd.flatten().collect(),
        Err(_) => return 0,
    };
    entries
        .par_iter()
        .map(|e| match e.file_type() {
            Ok(t) if t.is_dir() => dir_size_seq(&e.path()),
            Ok(t) if t.is_file() => e.metadata().map(|m| m.len()).unwrap_or(0),
            _ => 0,
        })
        .sum()
}

/// 单线程统计一棵子树的大小（作为并行统计的叶子任务）
fn dir_size_seq(path: &Path) -> u64 {
    walkdir::WalkDir::new(path)
        .into_iter()
        .flatten()
        .filter(|e| e.file_type().is_file())
        .map(|e| e.metadata().map(|m| m.len()).unwrap_or(0))
        .sum()
}
