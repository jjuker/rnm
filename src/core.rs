use crate::cli::Args;
use crate::utils::{calculate_dir_size, confirm_all, format_size, remove_dir_all_fast};
use anyhow::Result;
use indicatif::{ProgressBar, ProgressStyle};
use rayon::prelude::*;
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

struct ScanConfig {
    max_depth: Option<usize>,
}

/// 与 node_modules 无关且文件量大的目录，跳过以加速扫描
const SKIP_DIRS: &[&str] = &[".git", ".svn", ".hg", "target", ".idea", ".vscode"];

/// 扫描阶段只收集路径，大小统计延后到并行阶段
fn scan_node_modules(dir: &Path, config: &ScanConfig, results: &mut Vec<PathBuf>) {
    fn visit(dir: &Path, depth: usize, config: &ScanConfig, results: &mut Vec<PathBuf>) {
        if let Some(max) = config.max_depth {
            if depth >= max {
                return;
            }
        }

        // 无权限等错误直接跳过该目录，不中断整个扫描
        let Ok(entries) = std::fs::read_dir(dir) else {
            return;
        };
        for entry in entries.flatten() {
            // file_type() 复用 read_dir 已有信息，不额外 stat；
            // symlink/junction 的 is_dir() 为 false，天然不跟随，避免重复扫描和死循环
            let Ok(ft) = entry.file_type() else { continue };
            if !ft.is_dir() {
                continue;
            }
            let name = entry.file_name();
            if name == "node_modules" {
                // 找到即收集，不递归内部
                results.push(entry.path());
            } else if !SKIP_DIRS.iter().any(|s| name == *s) {
                visit(&entry.path(), depth + 1, config, results);
            }
        }
    }

    // 起始目录本身就是 node_modules 的情况
    if dir.file_name() == Some(std::ffi::OsStr::new("node_modules")) {
        results.push(dir.to_path_buf());
        return;
    }
    visit(dir, 0, config, results);
}

pub fn run(args: Args) -> Result<()> {
    let target_dir = args.dir.as_deref().unwrap_or(Path::new("."));

    if !target_dir.exists() {
        anyhow::bail!("目录不存在: {}", target_dir.display());
    }

    let scan_start = Instant::now();

    let config = ScanConfig {
        max_depth: if args.depth > 0 {
            Some(args.depth)
        } else {
            None
        },
    };

    let mut paths: Vec<PathBuf> = Vec::new();
    scan_node_modules(target_dir, &config, &mut paths);

    if paths.is_empty() {
        if !args.quiet {
            println!("未找到 node_modules 目录");
        }
        return Ok(());
    }

    // 大小统计与扫描解耦：多个 node_modules 之间并行计算
    let results: Vec<(PathBuf, u64)> = paths
        .into_par_iter()
        .map(|p| {
            let size = calculate_dir_size(&p);
            (p, size)
        })
        .collect();

    let scan_time = scan_start.elapsed();

    let total_size: u64 = results.iter().map(|(_, s)| s).sum();

    // 默认输出 或 测试模式详细输出
    if !args.quiet {
        if args.test {
            println!("\n[TEST MODE] 扫描结果:");
            println!("────────────────────────────────────");
            for (i, (path, size)) in results.iter().enumerate() {
                println!("  {}. {}", i + 1, path.display());
                println!("     大小: {}", format_size(*size));
            }
            println!("────────────────────────────────────");
            println!(
                "总计: {} 个目录, {}",
                results.len(),
                format_size(total_size)
            );
            println!(
                "扫描耗时: {:.3}s (速度: {:.2} MB/s)",
                scan_time.as_secs_f64(),
                total_size as f64 / scan_time.as_secs_f64() / 1024.0 / 1024.0
            );
            println!("\n[TEST MODE] 未执行删除");
        } else {
            println!(
                "\n找到 {} 个 node_modules 目录 ({}):",
                results.len(),
                format_size(total_size)
            );
            for (i, (path, size)) in results.iter().enumerate() {
                println!("  {}. {} ({})", i + 1, path.display(), format_size(*size));
            }
            println!("扫描耗时: {:.2}s", scan_time.as_secs_f64());
        }
    }

    if args.test {
        return Ok(());
    }

    // 确认删除
    if !args.yes {
        if args.quiet {
            // 静默模式需要用户确认才能继续
            if !confirm_all()? {
                return Ok(());
            }
        } else {
            if !confirm_all()? {
                println!("取消删除");
                return Ok(());
            }
        }
    }

    // 执行删除
    let delete_start = Instant::now();

    if args.quiet {
        // 多个 node_modules 之间并行删除（内部还会按一级子目录再并行）
        results
            .par_iter()
            .try_for_each(|(path, _)| remove_dir_all_fast(path))?;
    } else {
        let pb = ProgressBar::new(results.len() as u64);
        pb.set_style(
            ProgressStyle::default_spinner()
                .template("{spinner} 删除中 {pos}/{len}")
                .unwrap(),
        );
        pb.enable_steady_tick(Duration::from_millis(100));

        results.par_iter().try_for_each(|(path, _)| -> Result<()> {
            remove_dir_all_fast(path)?;
            pb.inc(1);
            Ok(())
        })?;
        pb.finish();

        let delete_time = delete_start.elapsed();
        println!(
            "删除耗时: {:.2}s (速度: {:.2} MB/s)",
            delete_time.as_secs_f64(),
            total_size as f64 / delete_time.as_secs_f64() / 1024.0 / 1024.0
        );
    }

    Ok(())
}
