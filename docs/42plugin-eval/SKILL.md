---
name: 42plugin-eval
description: >-
  Agent Skills evaluation system — expectation-based assertion testing for skill quality assessment.
  Uses a 2x2 matrix (model capability x practice quality) for classification, measures skill value
  through with_skill vs without_skill pass rate delta.
  Supports classification, evaluation, benchmark aggregation, and interactive report workflows.
  Use when the user wants to evaluate, test, benchmark, grade, or compare Agent Skills quality,
  run evals on a skill, measure skill performance, or iterate on skill improvements with data-driven feedback.
  Also triggers when the user mentions "评测技能", "技能质量", "skill eval", "eval benchmark".
allowed-tools: Bash(pwd:*), Bash(42plugin:*), Bash(cat:*), Bash(ls:*), Bash(mkdir:*), Bash(cp:*), Read, Write, Edit, Glob, Grep, Agent
metadata:
  author: 42ailab
  version: 2.2.0
  license: Proprietary
  title: 42plugin-eval
---

## Current Context

- **Project**: !`pwd`
- **Session**: ${CLAUDE_SESSION_ID}

!`42plugin __ skill 42plugin-eval`
