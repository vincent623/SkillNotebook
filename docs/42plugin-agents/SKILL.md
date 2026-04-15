---
name: 42plugin-agents
description: >
  Creates high-quality Agent Skills through multi-agent collaboration. Analyzes
  user conversation

  history to extract workflow patterns while exploring 42plugin library for
  design references.

  Use when creating skills, packaging workflows, or saying "create skill", "make
  a skill".
argument-hint: skill name or description
disable-model-invocation: true
user-invocable: true
allowed-tools: Bash(pwd:*), Bash(ls:*), Bash(42plugin:*), Read, Grep, Glob, Task
metadata:
  author: 42ailab
  version: 2.0.1
  license: Proprietary
  title: 智能技能创建
  description_zh: 通过多代理协作创建高质量技能。分析对话历史提取工作流，同时探索42plugin库学习优秀设计。
---

!`42plugin __ skill 42plugin-agents`
