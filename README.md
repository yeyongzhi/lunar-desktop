# Lunar

Lunar 是一个基于 Tauri 2、Vue 3 和 TypeScript 的 Windows 应用搜索工具。它读取系统及注册表中的应用信息，并在需要展示时限流提取、缓存应用图标。

## 开发

需要安装 Node.js、pnpm 和 Rust。

```bash
pnpm install
pnpm tauri dev
```

## 检查

```bash
pnpm build
pnpm check
pnpm test
```

- `build`：TypeScript 类型检查与前端生产构建
- `check`：前端类型检查、Rust 格式检查与 Clippy
- `test`：Rust 单元测试

## 目录

- `src/`：Vue 前端
- `src-tauri/`：Tauri/Rust 后端
- `src/commands/`：前后端命令类型与调用封装
