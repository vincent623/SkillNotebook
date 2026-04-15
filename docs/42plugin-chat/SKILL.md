---
name: 42plugin-chat
description: Export current conversation to ./chats directory
model: haiku
allowed-tools: Bash(pwd:*), Bash(42plugin:*)
disable-model-invocation: true
user-invocable: true
metadata:
  author: 42ailab
  version: 1.0.3
  license: Proprietary
  title: 活水对话导出
  description_zh: 快捷导出当前活水对话。
---

## Current Context

- **Project**: !`pwd`
- **Session**: ${CLAUDE_SESSION_ID}

!`42plugin __ skill 42plugin-chat`
