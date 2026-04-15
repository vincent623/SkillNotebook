---
name: 42plugin-meta
description: >-
  Use when creating, improving, or packaging Claude Code skills. Guides through
  the complete skill creation workflow including use case analysis, resource
  planning, SKILL.md writing with proper frontmatter, and validation using
  official skills-ref tool.
allowed-tools: Bash(pwd:*), Bash(42plugin:*), Read, Grep, Glob
metadata:
  author: 42ailab
  version: 2.0.1
  title: 技能创建指南
  description_zh: >-
    用于创建、改进或打包 AgentSkills 技能。引导完成整个技能创建流程：用例分析、资源规划、编写 SKILL.md、使用官方 skills-ref
    验证。
---

## Current Context

- **Project**: !`pwd`
- **Session**: ${CLAUDE_SESSION_ID}

!`42plugin __ skill 42plugin-meta`
