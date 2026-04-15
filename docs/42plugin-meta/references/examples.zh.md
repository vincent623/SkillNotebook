# 技能示例与注释

本文档提供设计良好的技能的带注释示例。

## 示例 1：PDF 处理技能

**为何有效：**
- 描述中有明确的触发条件
- 任务型结构匹配操作
- 具体的工具引用和命令
- 常见错误章节预防问题

```yaml
---
name: pdf-processing
description: "Use when manipulating PDF files - provides tools for merging, splitting, extracting text, and filling forms"
---
```

```markdown
# PDF Processing

## Overview

本技能提供常见 PDF 操作的工具，无需外部软件。

## When to Use

- 将多个 PDF 合并为一个
- 将 PDF 拆分为单独页面
- 从 PDF 中提取文本或表格
- 填写 PDF 中的表单字段

**不要用于：**
- 从头创建 PDF（使用文档创建工具）
- 直接编辑 PDF 内容（PDF 本质上是图片）

## Quick Reference

| 任务 | 命令 |
|------|------|
| 合并 PDF | `scripts/merge_pdf.py file1.pdf file2.pdf -o output.pdf` |
| 拆分 PDF | `scripts/split_pdf.py input.pdf -p 1-3,5,7-10` |
| 提取文本 | `scripts/extract_text.py input.pdf` |

## Common Mistakes

| 错误 | 修复 |
|------|------|
| 尝试编辑 PDF 文本 | PDF 不可编辑 - 重新创建文档 |
| 合并加密 PDF | 先解密或向用户索要密码 |
```

## 示例 2：API 集成技能

**为何有效：**
- 工作流型结构用于顺序流程
- 包含认证设置
- 错误处理指导
- 响应格式选项

```yaml
---
name: github-api-integrating
description: "Use when interacting with GitHub API - handles authentication, rate limiting, and provides helpers for common operations like issues, PRs, and repositories"
---
```

```markdown
# GitHub API Integration

## Overview

简化 GitHub API 交互，内置速率限制、认证和分页处理。

## When to Use

- 查询 issues、PR 或仓库
- 创建或更新 GitHub 资源
- 自动化 GitHub 工作流

## Quick Reference

| 操作 | 端点 | 需要认证 |
|------|------|----------|
| 列出仓库 | GET /users/{user}/repos | 否 |
| 创建 issue | POST /repos/{owner}/{repo}/issues | 是 |
| 获取 PR | GET /repos/{owner}/{repo}/pulls/{number} | 否 |

## Workflow

### Step 1: 认证

在环境变量中设置 token：
```bash
export GITHUB_TOKEN=ghp_xxxx
```

### Step 2: 发起请求

使用辅助函数：
```python
from scripts.github_api import github_request

# GET 请求
repos = github_request('GET', '/users/octocat/repos')

# 带数据的 POST 请求
issue = github_request('POST', '/repos/owner/repo/issues', {
    'title': 'Bug report',
    'body': 'Description here'
})
```

### Step 3: 处理分页

对于大型结果集：
```python
from scripts.github_api import github_paginate

all_issues = github_paginate('/repos/owner/repo/issues')
```

## Common Mistakes

| 错误 | 修复 |
|------|------|
| 超出速率限制 | 使用 `scripts/github_api.py` 它会处理速率限制 |
| 私有仓库返回 404 | 确保 GITHUB_TOKEN 有正确的权限范围 |
| 缺少分页 | 对列表端点使用 `github_paginate()` |
```

## 示例 3：品牌指南技能

**为何有效：**
- 参考型结构用于标准
- 具体的颜色代码和值
- 视觉示例
- 资产引用

```yaml
---
name: brand-styling
description: "Use when applying company brand to artifacts - provides official colors, typography, and logo usage guidelines"
---
```

```markdown
# Brand Styling

## Overview

使用官方指南在所有产出物上应用一致的品牌标识。

## When to Use

- 创建演示文稿或文档
- 设计 UI 组件
- 生成营销材料

## 品牌颜色

| 名称 | Hex | 用途 |
|------|-----|------|
| 主色蓝 | #2563EB | 标题、CTA |
| 辅助灰 | #6B7280 | 正文文字 |
| 强调橙 | #F97316 | 高亮 |
| 背景色 | #F9FAFB | 页面背景 |

## 字体

| 元素 | 字体 | 大小 | 字重 |
|------|------|------|------|
| H1 | Inter | 32px | 700 |
| H2 | Inter | 24px | 600 |
| 正文 | Inter | 16px | 400 |
| 代码 | Fira Code | 14px | 400 |

## Logo 使用

**可以：**
- 浅色背景使用 `assets/logo-primary.svg`
- 深色背景使用 `assets/logo-white.svg`
- 保持最小净空间 24px

**不可以：**
- 拉伸或扭曲 logo
- 更改 logo 颜色
- 将 logo 放在繁杂的背景上

## Resources

### Assets
- `assets/logo-primary.svg` - 全彩 logo
- `assets/logo-white.svg` - 白色版本
- `assets/brand-colors.json` - 代码格式的调色板
```

## 要避免的反模式

### 1. 模糊的描述
```yaml
# 差
description: "For testing"

# 好
description: "Use when tests are flaky or have race conditions - replaces arbitrary timeouts with condition polling"
```

### 2. 第一人称语态
```yaml
# 差
description: "I help you create documents"

# 好
description: "Use when creating structured documents - provides templates and formatting guidelines"
```

### 3. 叙事风格
```markdown
# 差
在我们使用 API 的经验中，我们发现你应该总是...

# 好
## 最佳实践
- API 调用总是包含错误处理
- 对列表端点使用分页
```

### 4. 多语言重复
```markdown
# 差 - 同一示例用 5 种语言
## Python
[代码]
## JavaScript
[代码]
## Ruby
[代码]
## Go
[代码]
## Rust
[代码]

# 好 - 一个优秀示例
## 实现
```python
# 注释完善的 Python 示例
# 可适配到其他语言
```
```

### 5. 过度记录基础知识
```markdown
# 差
## 什么是 PDF？
PDF（便携式文档格式）是 Adobe 开发的一种文件格式...

# 好
## PDF 操作
[直接进入有用的操作 - Claude 知道什么是 PDF]
```

## 新技能检查清单

发布前验证：

- [ ] 描述以 "Use when..." 开头
- [ ] 描述是第三人称
- [ ] 名称与目录名匹配
- [ ] 名称使用动名词形式
- [ ] 有 "When to Use" 章节
- [ ] 有 "Quick Reference" 表格
- [ ] 有 "Common Mistakes" 章节
- [ ] 少于 500 行
- [ ] 没有 [TODO] 占位符
- [ ] 每个技术有一个优秀示例
- [ ] 没有时效性信息
