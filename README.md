# rnm - Node Modules Cleaner

快速删除 node_modules 目录的命令行工具。

## 安装

```bash
cargo install rnm
```

或下载预编译二进制文件。

## 使用

```bash
# 在当前目录扫描并删除
rnm

# 指定目录
rnm -d /path/to/project

# 测试模式（不删除，只输出信息）
rnm -t

# 自动确认删除
rnm -y

# 静默模式
rnm -q

# 限制递归深度
rnm --depth 2
```

## 参数

| 参数 | 说明 |
|------|------|
| `-d, --dir` | 起始目录，默认当前目录 |
| `--depth` | 递归深度，0 表示无限 |
| `-y, --yes` | 自动确认删除 |
| `-q, --quiet` | 静默模式，无输出 |
| `-t, --test` | 测试模式，不删除，输出详细信息 |

## 特性

- 遇到 node_modules 目录不继续递归内部
- 显示目录大小和扫描耗时
- 测试模式输出性能指标（扫描/删除速度）
- 跨平台支持（Windows/macOS/Linux）