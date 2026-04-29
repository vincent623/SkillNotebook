// UI primitives — icons, buttons, small bits shared across views.

const { useState, useEffect, useRef, useMemo, useCallback } = React;

function Icon({ name, size = 14, style = {}, className = '' }) {
  const s = { width: size, height: size, display: 'inline-block', flexShrink: 0, verticalAlign: 'middle', ...style };
  const sw = 1.6;
  const paths = {
    search:  <><circle cx="7" cy="7" r="4.5"/><path d="M10.5 10.5L14 14"/></>,
    plus:    <><path d="M8 3v10M3 8h10"/></>,
    x:       <><path d="M4 4l8 8M12 4l-8 8"/></>,
    folder:  <><path d="M2 5a1 1 0 011-1h3l1.5 1.5H13a1 1 0 011 1V12a1 1 0 01-1 1H3a1 1 0 01-1-1z"/></>,
    folderOpen: <><path d="M2 5a1 1 0 011-1h3l1.5 1.5H13a1 1 0 011 1v.5H2z"/><path d="M2 6h12l-1 6a1 1 0 01-1 1H3a1 1 0 01-1-1z"/></>,
    file:    <><path d="M4 2h5l3 3v9a1 1 0 01-1 1H4a1 1 0 01-1-1V3a1 1 0 011-1z"/><path d="M9 2v3h3"/></>,
    md:      <><rect x="2" y="3.5" width="12" height="9" rx="1"/><path d="M4.5 10V6l1.5 2 1.5-2v4M10 6v4M10 10l1.5-1.5M10 10l-1.5-1.5"/></>,
    code:    <><path d="M5.5 4.5L2 8l3.5 3.5M10.5 4.5L14 8l-3.5 3.5"/></>,
    template:<><rect x="2.5" y="3" width="11" height="10" rx="1"/><path d="M2.5 6h11M5.5 6v7"/></>,
    skill:   <><path d="M8 2l1.8 3.8L14 6.5l-3 2.9.7 4.1L8 11.5 4.3 13.5 5 9.4 2 6.5l4.2-.7z"/></>,
    chevR:   <><path d="M6 3l5 5-5 5"/></>,
    chevD:   <><path d="M3 6l5 5 5-5"/></>,
    dot:     <><circle cx="8" cy="8" r="2" fill="currentColor"/></>,
    check:   <><path d="M3 8l3.5 3.5L13 5"/></>,
    warn:    <><path d="M8 2l6.5 11H1.5z"/><path d="M8 6.5v3M8 11.5v.5"/></>,
    err:     <><circle cx="8" cy="8" r="6"/><path d="M5.5 5.5l5 5M10.5 5.5l-5 5"/></>,
    git:     <><circle cx="4" cy="4" r="1.5"/><circle cx="12" cy="8" r="1.5"/><circle cx="4" cy="12" r="1.5"/><path d="M5.5 4.5h3.5a1.5 1.5 0 011.5 1.5v.5M4 5.5v5"/></>,
    copy:    <><rect x="5" y="2" width="8" height="10" rx="1"/><path d="M3 5v8a1 1 0 001 1h7"/></>,
    download:<><path d="M8 2v8M4 7l4 4 4-4M3 13h10"/></>,
    trash:   <><path d="M3 4.5h10M6 4.5V3h4v1.5M4.5 4.5l.5 8.5a1 1 0 001 1h4a1 1 0 001-1l.5-8.5"/></>,
    sparkle: <><path d="M8 2v3M8 11v3M2 8h3M11 8h3M4.5 4.5l2 2M11.5 4.5l-2 2M4.5 11.5l2-2M11.5 11.5l-2-2"/></>,
    wand:    <><path d="M3 13l9-9M9.5 3.5L12.5 6.5"/><path d="M13 9l.5 1.5L15 11l-1.5.5L13 13l-.5-1.5L11 11l1.5-.5z"/></>,
    edit:    <><path d="M11 2l3 3-8 8H3v-3z"/></>,
    archive: <><rect x="2" y="3" width="12" height="3" rx="0.5"/><path d="M3 6v7a1 1 0 001 1h8a1 1 0 001-1V6M6 9h4"/></>,
  };
  return (
    <svg viewBox="0 0 16 16" style={s} className={className} fill="none" stroke="currentColor" strokeWidth={sw} strokeLinecap="round" strokeLinejoin="round">
      {paths[name] || paths.dot}
    </svg>
  );
}

function Btn({ children, onClick, variant = 'secondary', style = {}, title, disabled }) {
  const base = {
    display: 'inline-flex', alignItems: 'center', gap: 5,
    padding: '6px 11px', borderRadius: 6,
    fontSize: 12.5, cursor: disabled ? 'not-allowed' : 'pointer',
    fontFamily: 'inherit', lineHeight: 1.2, fontWeight: 500,
    opacity: disabled ? 0.5 : 1, transition: 'background 0.08s',
  };
  const variants = {
    primary: { background: 'var(--ink)', color: 'var(--bg)', border: '1px solid var(--ink)' },
    accent:  { background: 'var(--accent)', color: '#fff', border: '1px solid var(--accent)' },
    secondary: { background: 'var(--bg)', color: 'var(--ink)', border: '1px solid var(--border)' },
    ghost: { background: 'transparent', color: 'var(--ink-muted)', border: '1px solid transparent' },
  };
  return (
    <button onClick={disabled ? null : onClick} title={title} disabled={disabled}
      style={{ ...base, ...variants[variant], ...style }}
      onMouseEnter={e => { if (variant === 'ghost' && !disabled) e.currentTarget.style.background = 'var(--bg-hover)'; }}
      onMouseLeave={e => { if (variant === 'ghost') e.currentTarget.style.background = 'transparent'; }}>
      {children}
    </button>
  );
}

function relTime(iso) {
  const d = new Date(iso);
  const diff = (Date.now() - d.getTime()) / 1000;
  if (diff < 60) return '刚刚';
  if (diff < 3600) return Math.floor(diff / 60) + ' 分钟前';
  if (diff < 86400) return Math.floor(diff / 3600) + ' 小时前';
  if (diff < 86400 * 7) return Math.floor(diff / 86400) + ' 天前';
  return d.toLocaleDateString('zh-CN', { month: 'short', day: 'numeric' });
}

// Markdown renderer (same as before, inline)
function renderMarkdown(src) {
  const escape = (s) => s.replace(/&/g, '&amp;').replace(/</g, '&lt;').replace(/>/g, '&gt;');
  const inline = (s) => s
    .replace(/`([^`]+)`/g, '<code>$1</code>')
    .replace(/\*\*([^*]+)\*\*/g, '<strong>$1</strong>')
    .replace(/\*([^*]+)\*/g, '<em>$1</em>');
  // strip frontmatter
  src = (src || '').replace(/^---\n[\s\S]*?\n---\n?/, '');
  const lines = src.split('\n');
  const out = [];
  let inCode = false, codeBuf = [];
  let inList = false;
  for (let i = 0; i < lines.length; i++) {
    let l = lines[i];
    if (l.startsWith('```')) {
      if (!inCode) { inCode = true; codeBuf = []; }
      else { inCode = false; out.push('<pre><code>' + escape(codeBuf.join('\n')) + '</code></pre>'); }
      continue;
    }
    if (inCode) { codeBuf.push(l); continue; }
    if (l.startsWith('# ')) { if (inList) { out.push('</' + inList + '>'); inList = false; } out.push('<h1>' + inline(escape(l.slice(2))) + '</h1>'); }
    else if (l.startsWith('## ')) { if (inList) { out.push('</' + inList + '>'); inList = false; } out.push('<h2>' + inline(escape(l.slice(3))) + '</h2>'); }
    else if (l.startsWith('### ')) { if (inList) { out.push('</' + inList + '>'); inList = false; } out.push('<h3>' + inline(escape(l.slice(4))) + '</h3>'); }
    else if (/^\d+\. /.test(l)) {
      if (inList !== 'ol') { if (inList) out.push('</' + inList + '>'); out.push('<ol>'); inList = 'ol'; }
      out.push('<li>' + inline(escape(l.replace(/^\d+\. /, ''))) + '</li>');
    }
    else if (l.startsWith('- ')) {
      if (inList !== 'ul') { if (inList) out.push('</' + inList + '>'); out.push('<ul>'); inList = 'ul'; }
      out.push('<li>' + inline(escape(l.slice(2))) + '</li>');
    }
    else if (l.startsWith('> ')) {
      if (inList) { out.push('</' + inList + '>'); inList = false; }
      out.push('<blockquote>' + inline(escape(l.slice(2))) + '</blockquote>');
    }
    else if (l.trim() === '') { if (inList) { out.push('</' + inList + '>'); inList = false; } out.push(''); }
    else { if (inList) { out.push('</' + inList + '>'); inList = false; } out.push('<p>' + inline(escape(l)) + '</p>'); }
  }
  if (inList) out.push('</' + inList + '>');
  return out.join('\n');
}

function Markdown({ source }) {
  const html = useMemo(() => renderMarkdown(source), [source]);
  return <div className="prose" dangerouslySetInnerHTML={{ __html: html }} />;
}

// ── Main state hook ──────────────────────────────────────────

function useSkillRepo() {
  const [state, setState] = useState(() => window.Store.load());

  const save = (next) => {
    setState(next);
    window.Store.save(next);
  };

  return {
    skills: state.skills,
    selectedId: state.selectedId,
    selectSkill: (id) => save({ ...state, selectedId: id }),
    addSkill: (skill) => save({ ...state, skills: [skill, ...state.skills], selectedId: skill.id }),
    touchSkill: (id) => {
      const skills = state.skills.map(s => s.id === id ? { ...s, lastUsedAt: new Date().toISOString() } : s);
      save({ ...state, skills });
    },
    updateSkill: (id, patch) => {
      const skills = state.skills.map(s => s.id === id ? { ...s, ...patch, updatedAt: new Date().toISOString() } : s);
      save({ ...state, skills });
    },
    updateFile: (id, path, content) => {
      const skills = state.skills.map(s => {
        if (s.id !== id) return s;
        const tree = JSON.parse(JSON.stringify(s.tree));
        window.TreeUtil.setFileContent(tree, path, content);
        return { ...s, tree, updatedAt: new Date().toISOString() };
      });
      save({ ...state, skills });
    },
    removeSkill: (id) => {
      const skills = state.skills.filter(s => s.id !== id);
      save({ ...state, skills, selectedId: skills[0]?.id });
    },
    bumpVersion: (id, message) => {
      const skills = state.skills.map(s => {
        if (s.id !== id) return s;
        const parts = s.version.split('.').map(Number);
        parts[2] = (parts[2] || 0) + 1;
        const newVer = parts.join('.');
        const tree = JSON.parse(JSON.stringify(s.tree));
        const cl = tree.children.find(c => c.name === 'CHANGELOG.md');
        const entry = `## ${newVer} — ${new Date().toISOString().slice(0, 10)}\n- ${message}\n\n`;
        if (cl) {
          cl.content = cl.content.replace(/^(# Changelog\n\n)/, `$1${entry}`);
        } else {
          tree.children.push({ type: 'file', name: 'CHANGELOG.md', content: `# Changelog\n\n${entry}` });
        }
        return { ...s, version: newVer, tree, updatedAt: new Date().toISOString() };
      });
      save({ ...state, skills });
    },
    reset: () => save(window.Store.reset())
  };
}

Object.assign(window, { Icon, Btn, relTime, renderMarkdown, Markdown, useSkillRepo });
