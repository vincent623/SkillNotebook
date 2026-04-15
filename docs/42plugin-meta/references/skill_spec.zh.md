# Claude 技能规范

## 概述

技能是一个包含指令、脚本和资源的文件夹，Claude 会动态加载它来更好地执行特定任务。文件夹必须包含 `SKILL.md` 文件才能被识别为技能。

## 文件夹结构

### 最小结构
```
skill-name/
└── SKILL.md
```

### 完整结构
```
skill-name/
├── SKILL.md          # 必需：入口点
├── scripts/          # 可选：可执行代码
│   ├── helper.py
│   └── process.sh
├── references/       # 可选：参考文档
│   ├── api_docs.md
│   └── schema.md
└── assets/           # 可选：输出文件
    ├── template.pptx
    └── logo.png
```

## SKILL.md 格式

### YAML Frontmatter（必需）

```yaml
---
name: skill-name
description: "Use when [触发条件] - [做什么]"
---
```

**必需属性：**
- `name`：连字符格式的标识符
  - 仅限小写 Unicode 字母数字 + 连字符
  - 必须与目录名匹配
  - 最多 64 字符
- `description`：技能做什么以及何时使用
  - 最多 1024 字符
  - 应以 "Use when..." 开头
  - 第三人称语态

**可选属性：**
- `license`：应用于技能的许可证
- `allowed-tools`：预先批准运行的工具列表（仅 Claude Code）
- `metadata`：自定义字符串键值对的映射

### Markdown 正文

格式无限制。推荐章节：
- Overview（概述）
- When to Use（何时使用）
- Quick Reference（快速参考）
- 主要内容（工作流/任务/指南）
- Common Mistakes（常见错误）
- Resources（资源）

## 资源类型

### scripts/
用于确定性操作的可执行代码。

**何时使用：**
- 相同代码反复重写
- 需要确定性可靠性
- 复杂的文件操作

**优点：**
- Token 高效（可执行而无需加载）
- 结果确定性
- 需要时 Claude 可以修补

### references/
按需加载到上下文中的文档。

**何时使用：**
- API 文档
- 数据库模式
- 领域知识
- 详细的工作流指南
- 内容 > 100 行

**最佳实践：**
- 保持 SKILL.md 精简，将细节移到这里
- 对大文件（>10k 词）包含 grep 搜索模式
- 不要与 SKILL.md 重复

### assets/
用于输出的文件（不加载到上下文）。

**何时使用：**
- 模板（pptx、docx）
- 图片和图标
- 字体
- 样板代码
- 示例数据

## 渐进式披露

技能使用三级加载：

1. **第 1 级 - 元数据**（约 100 词）
   - 始终在上下文中
   - 仅 name + description
   - 决定技能何时触发

2. **第 2 级 - SKILL.md 正文**（<5k 词）
   - 技能触发时加载
   - 主要指令和工作流

3. **第 3 级 - 捆绑资源**（无限制）
   - Claude 按需加载
   - 脚本可执行而无需读取

## 命名约定

### 技能名称
- 使用动名词形式：`creating-skills`，而非 `skill-creation`
- 小写加连字符：`data-analyzing`
- 描述性：`pdf-processing` 而非 `helper`
- 避免保留词：anthropic、claude

### 描述格式
```yaml
# 好
description: "Use when tests have race conditions - replaces timeouts with polling"

# 差 - 太模糊
description: "For testing"

# 差 - 第一人称
description: "I help with flaky tests"
```

## 质量指南

### 内容
- 一个优秀示例胜过多个平庸示例
- 质疑每句话：Claude 需要这个吗？
- 默认假设：Claude 很聪明
- 只添加 Claude 尚不知道的信息

### 结构
- SKILL.md 保持在 500 行以下
- 将参考内容移到 references/
- 使用表格进行快速查找
- 包含 "When to Use" 和 "When NOT to use"

### 维护
- 保持内容长青（无日期）
- 使用一致的术语
- 用 Claude Haiku、Sonnet 和 Opus 测试

## 版本历史

- 1.0（2025-10-16）：公开发布
