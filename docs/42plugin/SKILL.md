---
name: 42plugin
description: >-
  Search, install, and manage AI plugins to extend Claude's capabilities.
  Publish prompts and skills to share with the community.
  Use when the user mentions plugins, skills, 42plugin, or wants to find/install
  new capabilities for Claude.
allowed-tools: Bash(pwd:*), Bash(42plugin:*), Read, Grep, Glob
metadata:
  author: 42ailab
  version: 2.1.0
  license: Proprietary
  title: 42plugin
---

## Current Context

- **Project**: !`pwd`
- **Session**: ${CLAUDE_SESSION_ID}

!`42plugin __ skill 42plugin`
