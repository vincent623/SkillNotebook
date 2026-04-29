// Storage layer — localStorage-backed skill repo + tree helpers.

const STORAGE_KEY = 'skillbook.v2';

window.Store = {
  load() {
    try {
      const raw = localStorage.getItem(STORAGE_KEY);
      if (raw) return JSON.parse(raw);
    } catch (e) {}
    // first run — seed
    const seed = { skills: window.SEED_SKILLS, selectedId: window.SEED_SKILLS[0].id };
    localStorage.setItem(STORAGE_KEY, JSON.stringify(seed));
    return seed;
  },
  save(state) {
    localStorage.setItem(STORAGE_KEY, JSON.stringify(state));
  },
  reset() {
    localStorage.removeItem(STORAGE_KEY);
    return this.load();
  }
};

// Mock file-system paths — pretend the user has a home dir and projects.
// In a real deploy these would come from the host env.
window.MockPaths = {
  home: '/Users/you',
  claudeDir: '/Users/you/.claude/skills',
  projects: [
    { name: 'acme-web', path: '/Users/you/code/acme-web' },
    { name: 'data-pipeline', path: '/Users/you/code/data-pipeline' },
    { name: 'personal-site', path: '/Users/you/code/personal-site' },
  ],
  // Where the Skillbook itself keeps skills on disk (mock)
  vault: '/Users/you/.skillbook/skills',
  // Absolute path to a skill's root
  skillRoot(name) { return this.vault + '/' + name; },
  skillMd(name) { return this.skillRoot(name) + '/SKILL.md'; },
};

// ── Storage layer ───────────────────────────────────────────

window.TreeUtil = {
  // Walk the tree and return node at path array ['references', 'colors.md']
  findByPath(root, path) {
    let node = root;
    for (const seg of path) {
      if (!node.children) return null;
      node = node.children.find(c => c.name === seg);
      if (!node) return null;
    }
    return node;
  },

  // All files flattened, returning {path: [...], node}
  flatten(root, prefix = []) {
    const out = [];
    if (root.type === 'file') {
      out.push({ path: prefix, node: root });
    } else if (root.children) {
      for (const c of root.children) {
        out.push(...this.flatten(c, [...prefix, c.name]));
      }
    }
    return out;
  },

  // Mutate: update file content at path
  setFileContent(root, path, content) {
    const node = this.findByPath(root, path);
    if (node && node.type === 'file') node.content = content;
  },

  // Parse frontmatter from SKILL.md content
  parseFrontmatter(md) {
    const m = /^---\n([\s\S]*?)\n---\n?([\s\S]*)$/.exec(md || '');
    if (!m) return { meta: {}, body: md || '' };
    const meta = {};
    for (const line of m[1].split('\n')) {
      const kv = /^([a-zA-Z_-]+):\s*(.*)$/.exec(line);
      if (kv) meta[kv[1]] = kv[2].replace(/^["']|["']$/g, '');
    }
    return { meta, body: m[2] };
  },

  // Word count — Chinese chars each count 1, Latin split on whitespace
  wordCount(text) {
    if (!text) return 0;
    const cn = (text.match(/[\u4e00-\u9fa5]/g) || []).length;
    const en = (text.replace(/[\u4e00-\u9fa5]/g, ' ').trim().match(/\S+/g) || []).length;
    return cn + en;
  },

  // Validate whole skill — returns {ok, warnings:[], errors:[]}
  validate(skill) {
    const errs = [], warns = [];
    const root = skill.tree;
    const skillMd = root.children?.find(c => c.name === 'SKILL.md');
    if (!skillMd) {
      errs.push({ level: 'error', msg: '缺少 SKILL.md' });
      return { ok: false, errors: errs, warnings: warns };
    }
    const { meta, body } = this.parseFrontmatter(skillMd.content);
    if (!meta.name) errs.push({ level: 'error', msg: 'frontmatter 缺少 name' });
    if (!meta.description) errs.push({ level: 'error', msg: 'frontmatter 缺少 description' });
    if (meta.description && meta.description.length < 30) {
      warns.push({ level: 'warn', msg: 'description 过短（建议 ≥30 字）' });
    }
    const wc = this.wordCount(body);
    if (wc > 2000) warns.push({ level: 'warn', msg: `SKILL.md 正文 ${wc} 词，超过 2000 词建议` });

    // referenced paths — look for `references/xxx.md`, `scripts/xxx.sh`, `templates/xxx`
    const refs = [...(body.matchAll(/`([a-z_\-]+\/[a-z0-9_\-.]+)`/gi))].map(m => m[1]);
    const flat = this.flatten(root).map(f => f.path.join('/'));
    for (const ref of refs) {
      if (!flat.includes(ref)) {
        warns.push({ level: 'warn', msg: `引用了不存在的文件：${ref}` });
      }
    }
    return { ok: errs.length === 0, errors: errs, warnings: warns, wc };
  },

  // File-extension based kind
  kind(name) {
    if (name === 'SKILL.md') return 'skill';
    if (name === 'CHANGELOG.md') return 'changelog';
    if (name.endsWith('.md')) return 'markdown';
    if (name.endsWith('.py')) return 'python';
    if (name.endsWith('.sh')) return 'bash';
    if (name.endsWith('.template')) return 'template';
    if (name.endsWith('.json')) return 'json';
    if (name.endsWith('.html')) return 'html';
    return 'text';
  },

  // Tiny dir icon glyph — drawn w/ SVG downstream
  iconFor(node) {
    if (node.type === 'dir') return 'folder';
    const k = this.kind(node.name);
    if (k === 'skill') return 'skill';
    if (k === 'markdown' || k === 'changelog') return 'md';
    if (k === 'python' || k === 'bash') return 'code';
    if (k === 'template') return 'template';
    return 'file';
  }
};

// ── Mock AI generator ────────────────────────────────────────
// In a real deploy we'd call window.claude.complete — this returns
// a deterministic, well-shaped skill so the demo never stalls.

window.MockAI = {
  // Turn a user sentence into a kebab-case name
  nameFrom(text) {
    const stops = new Set(['a','an','the','for','to','of','and','or','in','on','with','my','our','i','you']);
    const words = (text || '').toLowerCase()
      .replace(/[^a-z0-9\s\u4e00-\u9fa5]/g, ' ')
      .split(/\s+/).filter(w => w && !stops.has(w)).slice(0, 3);
    if (!words.length) return 'new-skill';
    return words.join('-').slice(0, 40);
  },

  // Generate a full skill tree from a prompt
  generate(prompt, opts = {}) {
    const name = opts.name || this.nameFrom(prompt);
    const desc = opts.description
      || this._deriveDescription(prompt);
    const topic = prompt.trim() || 'generic workflow';

    const tree = {
      type: 'dir', name, children: [
        {
          type: 'file', name: 'SKILL.md',
          content: this._skillMd(name, desc, topic)
        },
        {
          type: 'dir', name: 'references', children: [
            { type: 'file', name: 'guidelines.md', content: this._guidelines(topic) },
            { type: 'file', name: 'examples.md', content: this._examples(topic) }
          ]
        },
        {
          type: 'dir', name: 'scripts', children: [
            { type: 'file', name: 'run.sh', content: this._runSh(name) }
          ]
        },
        {
          type: 'dir', name: 'templates', children: [
            { type: 'file', name: 'output.md.template', content: this._template(topic) }
          ]
        },
        {
          type: 'file', name: 'CHANGELOG.md',
          content: `# Changelog\n\n## 0.1.0 — ${new Date().toISOString().slice(0,10)}\n- Initial release (generated from: "${prompt.slice(0,80)}")\n`
        }
      ]
    };

    return {
      id: name + '-' + Math.random().toString(36).slice(2, 6),
      name, displayName: this._title(name),
      description: desc,
      version: '0.1.0',
      createdAt: new Date().toISOString(),
      updatedAt: new Date().toISOString(),
      tags: this._tags(prompt),
      tree
    };
  },

  _deriveDescription(prompt) {
    const base = prompt.trim().replace(/[.。]$/, '');
    if (!base) return 'Use this skill when the user asks for the task described in SKILL.md.';
    return `${base}. Use when the user asks to perform this task or mentions related keywords.`;
  },
  _title(kebab) {
    return kebab.split('-').map(w => w[0]?.toUpperCase() + w.slice(1)).join(' ');
  },
  _tags(prompt) {
    const all = (prompt || '').toLowerCase();
    const candidates = [
      ['api','api'], ['review','review'], ['pdf','pdf'], ['docs','docs'],
      ['brand','brand'], ['commit','git'], ['security','security'],
      ['slide','deck'], ['deck','deck'], ['data','data'],
      ['extract','extract'], ['report','report']
    ];
    return candidates.filter(([kw]) => all.includes(kw)).map(([,t]) => t).slice(0, 3);
  },
  _skillMd(name, desc, topic) {
    return `---
name: ${name}
description: ${desc}
---

# ${this._title(name)}

This skill helps Claude ${topic.toLowerCase()}.

## When to use

Invoke this skill when the user:
- Asks to ${topic.toLowerCase()}
- Mentions keywords related to this task
- Runs \`/${name}\` explicitly

## Workflow

1. Read the user's input and clarify any ambiguity.
2. Consult \`references/guidelines.md\` for the rules.
3. Look at \`references/examples.md\` for patterns.
4. Produce output following \`templates/output.md.template\`.
5. If automation helps, run \`scripts/run.sh\`.

## Output expectations

Keep output concise. Surface the most important signal first. If something is uncertain, say so explicitly rather than guessing.
`;
  },
  _guidelines(topic) {
    return `# Guidelines

Rules for ${topic}:

- **Be specific.** Vague output is worse than no output.
- **Cite sources.** When referencing input data, include line numbers or quotes.
- **Fail loud.** If a required input is missing, ask — don't invent.

## Edge cases

- Empty input → ask the user to clarify.
- Conflicting input → surface both options, pick one, explain why.
- Malformed input → repair silently only when safe; otherwise flag.
`;
  },
  _examples(topic) {
    return `# Examples

## Good

> User: "${topic}"
>
> Assistant: *(concise, structured output)*

## Bad

> User: "${topic}"
>
> Assistant: *(rambling, unstructured, no structure)*

## Edge case

> User: provides incomplete data
>
> Assistant: asks one clarifying question before proceeding.
`;
  },
  _runSh(name) {
    return `#!/bin/bash
# Run helper for ${name}
# Usage: ./run.sh [args...]

set -e

echo "Running ${name}..."
# TODO: implement
`;
  },
  _template(topic) {
    return `# {{TITLE}}

_Generated by ${topic}_

## Summary
{{SUMMARY}}

## Details
{{DETAILS}}

## Next steps
{{NEXT_STEPS}}
`;
  },

  // Re-derive description from a SKILL.md body
  deriveDescription(body) {
    const firstPara = (body || '').split('\n\n').find(p => p.trim() && !p.startsWith('#'));
    if (!firstPara) return 'Use this skill when relevant.';
    const sentence = firstPara.replace(/\s+/g, ' ').trim().split(/(?<=[.。])\s/)[0];
    return sentence.slice(0, 180) + (sentence.length > 180 ? '…' : '');
  },

  // Suggest which paragraphs in SKILL.md body should be split into references/
  suggestSplits(body) {
    const sections = [];
    const parts = (body || '').split(/^## /m);
    for (let i = 1; i < parts.length; i++) {
      const sec = parts[i];
      const title = sec.split('\n')[0].trim();
      const wc = window.TreeUtil.wordCount(sec);
      if (wc > 250) {
        sections.push({ title, wc, suggestedFile: 'references/' + title.toLowerCase().replace(/\s+/g, '-') + '.md' });
      }
    }
    return sections;
  }
};
