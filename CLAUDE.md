# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## 项目概述

rnm 是一个 Rust CLI 工具，用于快速删除 node_modules 目录。

## 构建命令

```bash
cargo build --release   # 发布构建
cargo run               # 开发运行
cargo test              # 测试
cargo clippy            # 代码检查
```

## CLI 参数

| 参数 | 功能 |
|------|------|
| `-d/--dir` | 起始目录 |
| `--depth` | 递归深度（0=无限） |
| `-y/--yes` | 自动确认 |
| `-q/--quiet` | 静默模式 |
| `-t/--test` | 测试模式，不删除 |

## 代码架构

三层结构：

```
main.rs → cli.rs (参数解析) → core.rs (扫描/删除逻辑)
                                ↓
                             utils.rs (工具函数)
```

**关键设计：**
- 遇到 node_modules 目录立即返回，不继续递归内部
- `scan_node_modules` 独立函数，参数用 `ScanConfig` struct 组合
- 三种输出模式：默认（带进度）、静默（无输出）、测试（详细日志）