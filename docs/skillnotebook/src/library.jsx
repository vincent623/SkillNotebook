// Library — Finder column browser. Skills → directory tree → file → content.

function Library({ repo, onNew, onOpenCmd }) {
  const { skills, selectedId, selectSkill, updateFile, updateSkill, removeSkill, bumpVersion } = repo;
  const selected = skills.find(s => s.id === selectedId) || skills[0];

  // Column path: array of path segments within selected skill, ending at a file for content view
  const [path, setPath] = useState([]);
  const [editing, setEditing] = useState(false);
  const [exportOpen, setExportOpen] = useState(false);
  const [commitOpen, setCommitOpen] = useState(false);

  // When skill changes, reset path
  useEffect(() => { setPath([]); setEditing(false); }, [selectedId]);

  // Columns: root is selected skill tree; path drills in.
  const columns = useMemo(() => {
    if (!selected) return [];
    const cols = [selected.tree];
    let current = selected.tree;
    for (const seg of path) {
      const next = current.children?.find(c => c.name === seg);
      if (!next) break;
      cols.push(next);
      current = next;
    }
    return cols;
  }, [selected, path]);

  const currentFile = columns[columns.length - 1];
  const isFileOpen = currentFile?.type === 'file';

  // Validation on selected skill
  const validation = useMemo(() => selected ? window.TreeUtil.validate(selected) : null, [selected]);

  return (
    <div style={{
      display: 'grid',
      gridTemplateRows: 'auto 1fr',
      height: '100vh',
      fontFamily: 'var(--font-ui)',
      color: 'var(--ink)',
      background: 'var(--bg)',
      overflow: 'hidden',
    }}>
      <TopBar
        skill={selected}
        validation={validation}
        onNew={onNew}
        onOpenCmd={onOpenCmd}
        onExport={() => setExportOpen(true)}
        onCommit={() => setCommitOpen(true)}
        onDelete={() => { if (confirm('删除 ' + selected.name + '？')) removeSkill(selected.id); }}
      />

      {/* Columns */}
      <div style={{ display: 'flex', height: '100%', overflow: 'hidden' }}>
        {/* Column 1: skill list */}
        <SkillListColumn
          skills={skills}
          selectedId={selectedId}
          onSelect={selectSkill}
          onNew={onNew}
        />

        {/* Column 2: top-level children of selected skill (grouped dirs + files) */}
        {selected && (
          <TreeColumn
            title={selected.tree.name}
            nodes={selected.tree.children || []}
            active={path[0]}
            onSelect={(name) => setPath([name])}
            depth={0}
            hint="目录"
          />
        )}

        {/* Additional nested columns for drilling into subdirs */}
        {path.map((seg, i) => {
          const parent = columns[i + 1];
          if (!parent || parent.type !== 'dir') return null;
          return (
            <TreeColumn
              key={i}
              title={seg}
              nodes={parent.children || []}
              active={path[i + 1]}
              onSelect={(name) => setPath([...path.slice(0, i + 1), name])}
              depth={i + 1}
              hint="文件"
            />
          );
        })}

        {/* Content pane */}
        <ContentPane
          file={isFileOpen ? currentFile : null}
          skill={selected}
          path={path}
          editing={editing}
          onEdit={() => setEditing(true)}
          onDone={() => setEditing(false)}
          onChange={(content) => updateFile(selected.id, path, content)}
          validation={validation}
        />
      </div>

      {exportOpen && <ExportModal skill={selected} onClose={() => setExportOpen(false)} />}
      {commitOpen && <CommitModal skill={selected} onCommit={(msg) => { bumpVersion(selected.id, msg); setCommitOpen(false); }} onClose={() => setCommitOpen(false)} />}
    </div>
  );
}

// ── TopBar ────────────────────────────────────────────────

function TopBar({ skill, validation, onNew, onOpenCmd, onExport, onCommit, onDelete }) {
  return (
    <header style={{
      display: 'flex', alignItems: 'center', gap: 10,
      padding: '12px 18px',
      borderBottom: '1px solid var(--border)',
      background: 'var(--bg)',
      fontSize: 13,
    }}>
      <div style={{ display: 'flex', alignItems: 'center', gap: 8, flexShrink: 0 }}>
        <div style={{
          width: 22, height: 22, borderRadius: 6,
          background: 'var(--accent)', color: '#fff',
          display: 'grid', placeItems: 'center',
          fontWeight: 700, fontSize: 11,
        }}>技</div>
        <span style={{ fontWeight: 600, letterSpacing: '-0.01em' }}>技能本</span>
      </div>

      {skill && (
        <div style={{ display: 'flex', alignItems: 'center', gap: 8, minWidth: 0, flex: 1 }}>
          <span style={{ color: 'var(--ink-faint)', marginLeft: 4, flexShrink: 0 }}>›</span>
          <code style={{ fontFamily: 'var(--font-mono)', fontSize: 12, color: 'var(--ink-muted)', whiteSpace: 'nowrap', overflow: 'hidden', textOverflow: 'ellipsis', minWidth: 0 }}>{skill.name}/</code>
          <code style={{
            fontFamily: 'var(--font-mono)', fontSize: 11,
            padding: '2px 7px', background: 'var(--bg-sidebar)',
            border: '1px solid var(--border)', borderRadius: 4,
            color: 'var(--ink-muted)', flexShrink: 0,
          }}>v{skill.version}</code>
          {validation && <ValidationBadge v={validation} />}
        </div>
      )}

      {!skill && <div style={{ flex: 1 }} />}

      <button onClick={onOpenCmd} style={{
        display: 'inline-flex', alignItems: 'center', gap: 6,
        padding: '6px 11px',
        background: 'var(--bg-sidebar)',
        border: '1px solid var(--border)',
        borderRadius: 6,
        color: 'var(--ink-muted)',
        fontSize: 12.5,
        cursor: 'pointer',
        fontFamily: 'inherit',
      }}>
        <Icon name="search" size={12} /> 搜索
        <kbd style={{ fontSize: 10, fontFamily: 'var(--font-mono)', color: 'var(--ink-faint)', marginLeft: 6 }}>⌘K</kbd>
      </button>

      <Btn onClick={onNew} variant="accent" style={{ whiteSpace: 'nowrap', flexShrink: 0 }}><Icon name="wand" size={12}/> 生成 Skill</Btn>
      {skill && <Btn onClick={onCommit} variant="ghost" title="提交新版本"><Icon name="git" size={13}/></Btn>}
      {skill && <Btn onClick={onExport} variant="secondary"><Icon name="download" size={12}/> 导出</Btn>}
      {skill && <Btn onClick={onDelete} variant="ghost" title="删除"><Icon name="trash" size={13}/></Btn>}
    </header>
  );
}

function ValidationBadge({ v }) {
  if (v.errors.length > 0) {
    return <span style={{
      display: 'inline-flex', alignItems: 'center', gap: 4,
      padding: '2px 7px', borderRadius: 4,
      fontSize: 11, color: '#b91c1c',
      background: '#fef2f2', border: '1px solid #fecaca',
    }}><Icon name="err" size={11}/> {v.errors.length}</span>;
  }
  if (v.warnings.length > 0) {
    return <span style={{
      display: 'inline-flex', alignItems: 'center', gap: 4,
      padding: '2px 7px', borderRadius: 4,
      fontSize: 11, color: '#a16207',
      background: '#fefce8', border: '1px solid #fde68a',
    }}><Icon name="warn" size={11}/> {v.warnings.length}</span>;
  }
  return <span style={{
    display: 'inline-flex', alignItems: 'center', gap: 4,
    padding: '2px 7px', borderRadius: 4,
    fontSize: 11, color: '#15803d',
    background: '#f0fdf4', border: '1px solid #bbf7d0',
  }}><Icon name="check" size={11}/> 校验通过</span>;
}

// ── Skill list column ─────────────────────────────────────

function SkillListColumn({ skills, selectedId, onSelect, onNew }) {
  const [q, setQ] = useState('');
  const [tagFilter, setTagFilter] = useState(null);

  const allTags = useMemo(() => {
    const c = {};
    skills.forEach(s => (s.tags || []).forEach(t => { c[t] = (c[t] || 0) + 1; }));
    return Object.entries(c).sort((a,b) => b[1]-a[1]);
  }, [skills]);

  const filtered = useMemo(() => {
    let list = skills;
    if (tagFilter) list = list.filter(s => (s.tags || []).includes(tagFilter));
    if (q) {
      const ql = q.toLowerCase();
      list = list.filter(s =>
        s.name.toLowerCase().includes(ql) ||
        s.description.toLowerCase().includes(ql) ||
        (s.tags || []).some(t => t.toLowerCase().includes(ql))
      );
    }
    return list;
  }, [skills, q, tagFilter]);

  // Group: recently used (within 7 days) vs all
  const recent = filtered.filter(s => s.lastUsedAt && (Date.now() - new Date(s.lastUsedAt).getTime()) < 7 * 86400 * 1000);
  const rest = filtered.filter(s => !recent.includes(s));

  return (
    <section style={{
      width: 280,
      borderRight: '1px solid var(--border)',
      background: 'var(--bg-sidebar)',
      display: 'flex', flexDirection: 'column',
      overflow: 'hidden',
    }}>
      <ColumnHeader label={`Skills · ${skills.length}`} action={
        <button onClick={onNew} title="生成 skill" style={iconBtn}><Icon name="plus" size={12}/></button>
      } />

      {/* search */}
      <div style={{ padding: '6px 10px 4px' }}>
        <div style={{ display: 'flex', alignItems: 'center', gap: 6, padding: '6px 9px', background: 'var(--bg)', border: '1px solid var(--border)', borderRadius: 6 }}>
          <Icon name="search" size={11} style={{ color: 'var(--ink-faint)' }}/>
          <input value={q} onChange={e => setQ(e.target.value)} placeholder="搜索名称、描述、标签"
            style={{ flex: 1, border: 'none', background: 'transparent', outline: 'none', fontSize: 12, color: 'var(--ink)', minWidth: 0 }}/>
          {q && <button onClick={() => setQ('')} style={iconBtn}><Icon name="x" size={10}/></button>}
        </div>
      </div>

      {/* tag chips */}
      {allTags.length > 0 && (
        <div style={{ padding: '4px 10px 8px', display: 'flex', flexWrap: 'wrap', gap: 4 }}>
          <TagChip label="全部" count={skills.length} active={!tagFilter} onClick={() => setTagFilter(null)}/>
          {allTags.slice(0, 10).map(([t, n]) => (
            <TagChip key={t} label={t} count={n} active={tagFilter === t} onClick={() => setTagFilter(tagFilter === t ? null : t)}/>
          ))}
        </div>
      )}

      <div style={{ flex: 1, overflow: 'auto', padding: '4px 8px 16px' }}>
        {recent.length > 0 && (
          <>
            <GroupLabel label="最近使用"/>
            {recent.map(s => <SkillRow key={s.id} skill={s} active={s.id === selectedId} onClick={() => onSelect(s.id)}/>)}
            <div style={{ height: 10 }}/>
            <GroupLabel label="全部"/>
          </>
        )}
        {rest.map(s => <SkillRow key={s.id} skill={s} active={s.id === selectedId} onClick={() => onSelect(s.id)}/>)}
        {filtered.length === 0 && (
          <div style={{ padding: 24, fontSize: 11.5, color: 'var(--ink-faint)', textAlign: 'center' }}>
            没有匹配的 skill
          </div>
        )}
      </div>
    </section>
  );
}

function TagChip({ label, count, active, onClick }) {
  return (
    <button onClick={onClick} style={{
      padding: '3px 8px', borderRadius: 99,
      background: active ? 'var(--ink)' : 'var(--bg)',
      color: active ? 'var(--bg)' : 'var(--ink-muted)',
      border: '1px solid ' + (active ? 'var(--ink)' : 'var(--border)'),
      fontSize: 10.5, cursor: 'pointer', fontFamily: 'inherit',
      display: 'inline-flex', alignItems: 'center', gap: 4,
    }}>
      {label}
      <span style={{ fontFamily: 'var(--font-mono)', opacity: 0.6, fontSize: 9.5 }}>{count}</span>
    </button>
  );
}

function GroupLabel({ label }) {
  return <div style={{ padding: '6px 10px 2px', fontSize: 10, color: 'var(--ink-faint)', textTransform: 'uppercase', letterSpacing: '0.08em', fontWeight: 600 }}>{label}</div>;
}

function SkillRow({ skill: s, active, onClick }) {
  return (
    <button onClick={onClick}
      style={{
        display: 'block', width: '100%', textAlign: 'left',
        padding: '9px 10px', marginBottom: 1,
        background: active ? 'var(--bg-active)' : 'transparent',
        border: 'none', borderRadius: 6, cursor: 'pointer',
        fontFamily: 'inherit', color: 'var(--ink)',
      }}
      onMouseEnter={e => { if (!active) e.currentTarget.style.background = 'var(--bg-hover)'; }}
      onMouseLeave={e => { if (!active) e.currentTarget.style.background = 'transparent'; }}
    >
      <div style={{ display: 'flex', alignItems: 'baseline', gap: 6, minWidth: 0 }}>
        <code style={{ fontFamily: 'var(--font-mono)', fontSize: 12, fontWeight: 500, color: 'var(--ink)', overflow: 'hidden', textOverflow: 'ellipsis', whiteSpace: 'nowrap', flex: 1, minWidth: 0 }}>{s.name}</code>
        <span style={{ fontSize: 10, color: 'var(--ink-faint)', fontFamily: 'var(--font-mono)', flexShrink: 0 }}>v{s.version}</span>
      </div>
      <div style={{ fontSize: 11.5, color: 'var(--ink-muted)', marginTop: 3, lineHeight: 1.4,
        display: '-webkit-box', WebkitLineClamp: 2, WebkitBoxOrient: 'vertical', overflow: 'hidden',
      }}>{s.description}</div>
      <div style={{ fontSize: 10, color: 'var(--ink-faint)', marginTop: 5, display: 'flex', alignItems: 'center', gap: 6 }}>
        {relTime(s.updatedAt)}
        {(s.tags || []).slice(0, 2).map(t => (
          <span key={t} style={{ padding: '1px 5px', background: 'var(--bg)', border: '1px solid var(--border-faint)', borderRadius: 3, color: 'var(--ink-muted)', fontFamily: 'var(--font-mono)', fontSize: 9.5 }}>{t}</span>
        ))}
      </div>
    </button>
  );
}

// ── Tree column ───────────────────────────────────────────

function TreeColumn({ title, nodes, active, onSelect, depth, hint }) {
  // Sort: dirs first, SKILL.md first among files, then alpha
  const sorted = [...nodes].sort((a, b) => {
    if (a.type !== b.type) return a.type === 'dir' ? -1 : 1;
    if (a.name === 'SKILL.md') return -1;
    if (b.name === 'SKILL.md') return 1;
    return a.name.localeCompare(b.name);
  });

  return (
    <section style={{
      width: 220,
      borderRight: '1px solid var(--border)',
      background: depth % 2 === 0 ? 'var(--bg)' : 'var(--bg-sidebar)',
      display: 'flex', flexDirection: 'column',
      overflow: 'hidden',
    }}>
      <ColumnHeader label={hint} sub={title} mono/>
      <div style={{ flex: 1, overflow: 'auto', padding: '4px 6px 16px' }}>
        {sorted.map(n => (
          <TreeRow key={n.name} node={n} active={n.name === active} onClick={() => onSelect(n.name)} />
        ))}
        {sorted.length === 0 && (
          <div style={{ padding: 20, fontSize: 11, color: 'var(--ink-faint)', textAlign: 'center' }}>空目录</div>
        )}
      </div>
    </section>
  );
}

function TreeRow({ node, active, onClick }) {
  const iconName = window.TreeUtil.iconFor(node);
  const isDir = node.type === 'dir';
  return (
    <button onClick={onClick}
      style={{
        display: 'flex', alignItems: 'center', gap: 8,
        width: '100%', padding: '6px 8px', marginBottom: 1,
        background: active ? 'var(--bg-active)' : 'transparent',
        border: 'none', borderRadius: 5, cursor: 'pointer',
        fontFamily: 'inherit', fontSize: 12.5,
        color: 'var(--ink)', textAlign: 'left',
      }}
      onMouseEnter={e => { if (!active) e.currentTarget.style.background = 'var(--bg-hover)'; }}
      onMouseLeave={e => { if (!active) e.currentTarget.style.background = 'transparent'; }}
    >
      <Icon name={iconName} size={13} style={{
        color: isDir ? 'var(--accent)' : (node.name === 'SKILL.md' ? 'var(--accent)' : 'var(--ink-faint)'),
      }}/>
      <span style={{
        fontFamily: node.name === 'SKILL.md' || !isDir ? 'var(--font-mono)' : 'inherit',
        fontSize: isDir ? 12.5 : 12,
        fontWeight: node.name === 'SKILL.md' ? 600 : 400,
        flex: 1, overflow: 'hidden', textOverflow: 'ellipsis', whiteSpace: 'nowrap',
      }}>{node.name}</span>
      {isDir && <Icon name="chevR" size={10} style={{ color: 'var(--ink-faint)' }}/>}
    </button>
  );
}

// ── Column header ─────────────────────────────────────────

function ColumnHeader({ label, sub, mono, action }) {
  return (
    <div style={{
      padding: '10px 12px 8px',
      borderBottom: '1px solid var(--border-faint)',
      display: 'flex', alignItems: 'center', gap: 6,
      background: 'transparent',
    }}>
      <div style={{ fontSize: 10.5, color: 'var(--ink-faint)', textTransform: 'uppercase', letterSpacing: '0.08em', fontWeight: 600 }}>
        {label}
      </div>
      {sub && <code style={{ fontFamily: 'var(--font-mono)', fontSize: 11, color: 'var(--ink-muted)', marginLeft: 2 }}>{sub}</code>}
      {action && <div style={{ marginLeft: 'auto' }}>{action}</div>}
    </div>
  );
}

const iconBtn = {
  background: 'none', border: 'none',
  color: 'var(--ink-faint)', cursor: 'pointer',
  padding: 2, display: 'grid', placeItems: 'center',
};

// ── Content pane ──────────────────────────────────────────

function ContentPane({ file, skill, path, editing, onEdit, onDone, onChange, validation }) {
  if (!file) {
    return (
      <section style={{ flex: 1, display: 'grid', placeItems: 'center', color: 'var(--ink-faint)', background: 'var(--bg)' }}>
        <div style={{ textAlign: 'center', maxWidth: 360 }}>
          <Icon name="archive" size={32}/>
          <div style={{ marginTop: 14, fontSize: 14, color: 'var(--ink-muted)' }}>
            {skill ? '从左侧选择一个文件' : '请先生成一个 skill'}
          </div>
          {skill && validation && (validation.errors.length || validation.warnings.length) > 0 && (
            <ValidationPanel v={validation} style={{ marginTop: 24, textAlign: 'left' }} />
          )}
        </div>
      </section>
    );
  }

  const kind = window.TreeUtil.kind(file.name);
  const isMarkdown = kind === 'markdown' || kind === 'skill' || kind === 'changelog';

  return (
    <section style={{ flex: 1, display: 'flex', flexDirection: 'column', background: 'var(--bg)', overflow: 'hidden' }}>
      <header style={{
        display: 'flex', alignItems: 'center', gap: 10,
        padding: '10px 18px',
        borderBottom: '1px solid var(--border)',
        fontSize: 12,
      }}>
        <Icon name={window.TreeUtil.iconFor(file)} size={13} style={{ color: file.name === 'SKILL.md' ? 'var(--accent)' : 'var(--ink-faint)' }}/>
        <code style={{ fontFamily: 'var(--font-mono)', fontSize: 12, color: 'var(--ink)' }}>
          {[skill.name, ...path].join('/')}
        </code>
        <span style={{ color: 'var(--ink-faint)', fontSize: 11 }}>· {window.TreeUtil.wordCount(file.content)} 词</span>
        <div style={{ flex: 1 }}/>
        {editing ? (
          <Btn onClick={onDone} variant="primary">完成</Btn>
        ) : (
          <Btn onClick={onEdit} variant="secondary"><Icon name="edit" size={12}/> 编辑</Btn>
        )}
        <Btn onClick={() => { navigator.clipboard?.writeText(file.content || ''); }} variant="ghost" title="复制"><Icon name="copy" size={12}/></Btn>
      </header>

      <div style={{ flex: 1, overflow: 'auto' }}>
        {editing ? (
          <textarea
            value={file.content || ''}
            onChange={e => onChange(e.target.value)}
            style={{
              width: '100%', height: '100%', minHeight: '100%',
              border: 'none', outline: 'none', resize: 'none',
              padding: '28px 32px',
              fontFamily: 'var(--font-mono)', fontSize: 13, lineHeight: 1.7,
              background: 'var(--bg)', color: 'var(--ink)',
              boxSizing: 'border-box',
            }}
          />
        ) : isMarkdown ? (
          <div style={{ padding: '32px 44px', maxWidth: 760 }}>
            {file.name === 'SKILL.md' && <FrontmatterCard content={file.content}/>}
            <Markdown source={file.content}/>
            {file.name === 'SKILL.md' && validation && <ValidationPanel v={validation} style={{ marginTop: 40 }}/>}
          </div>
        ) : (
          <pre style={{
            margin: 0, padding: '28px 32px',
            fontFamily: 'var(--font-mono)', fontSize: 12.5, lineHeight: 1.7,
            color: 'var(--ink)', background: 'var(--bg)',
            whiteSpace: 'pre-wrap', wordBreak: 'break-word',
          }}>{file.content}</pre>
        )}
      </div>
    </section>
  );
}

function FrontmatterCard({ content }) {
  const { meta } = window.TreeUtil.parseFrontmatter(content);
  if (!meta.name) return null;
  return (
    <div style={{
      padding: '14px 16px', marginBottom: 28,
      background: 'var(--bg-sidebar)',
      border: '1px solid var(--border)',
      borderRadius: 8,
      fontSize: 12.5, lineHeight: 1.6,
    }}>
      <div style={{ fontSize: 10, color: 'var(--ink-faint)', textTransform: 'uppercase', letterSpacing: '0.08em', fontWeight: 600, marginBottom: 6 }}>Frontmatter</div>
      <div style={{ display: 'grid', gridTemplateColumns: '80px 1fr', gap: '6px 14px' }}>
        <code style={{ fontFamily: 'var(--font-mono)', color: 'var(--ink-muted)' }}>name:</code>
        <code style={{ fontFamily: 'var(--font-mono)', color: 'var(--ink)' }}>{meta.name}</code>
        <code style={{ fontFamily: 'var(--font-mono)', color: 'var(--ink-muted)' }}>description:</code>
        <div style={{ color: 'var(--ink)' }}>{meta.description}</div>
      </div>
    </div>
  );
}

function ValidationPanel({ v, style = {} }) {
  const all = [...v.errors, ...v.warnings];
  if (all.length === 0) {
    return (
      <div style={{ ...style, display: 'flex', alignItems: 'center', gap: 8, padding: '12px 14px', background: '#f0fdf4', border: '1px solid #bbf7d0', borderRadius: 8, fontSize: 12.5, color: '#15803d' }}>
        <Icon name="check" size={14}/> 全部校验通过 · {v.wc || 0} 词
      </div>
    );
  }
  return (
    <div style={{ ...style, padding: '14px 16px', background: 'var(--bg-sidebar)', border: '1px solid var(--border)', borderRadius: 8, fontSize: 12.5 }}>
      <div style={{ fontSize: 10, color: 'var(--ink-faint)', textTransform: 'uppercase', letterSpacing: '0.08em', fontWeight: 600, marginBottom: 10 }}>校验结果</div>
      {all.map((w, i) => (
        <div key={i} style={{ display: 'flex', alignItems: 'center', gap: 8, padding: '4px 0' }}>
          <Icon name={w.level === 'error' ? 'err' : 'warn'} size={12} style={{ color: w.level === 'error' ? '#b91c1c' : '#a16207' }}/>
          <span style={{ color: 'var(--ink)' }}>{w.msg}</span>
        </div>
      ))}
    </div>
  );
}

// ── Commit modal ──────────────────────────────────────────

function CommitModal({ skill, onCommit, onClose }) {
  const [msg, setMsg] = useState('');
  const inputRef = useRef(null);
  useEffect(() => { inputRef.current?.focus(); }, []);
  const parts = skill.version.split('.').map(Number); parts[2] = (parts[2] || 0) + 1;
  const newVer = parts.join('.');
  return (
    <div onClick={onClose} style={modalBackdrop}>
      <div onClick={e => e.stopPropagation()} style={{ ...modalCard, width: 460 }}>
        <div style={{ fontSize: 10, color: 'var(--ink-faint)', textTransform: 'uppercase', letterSpacing: '0.08em', marginBottom: 4 }}>提交新版本</div>
        <div style={{ fontSize: 16, fontWeight: 600, marginBottom: 18 }}>{skill.name}</div>
        <div style={{ display: 'flex', alignItems: 'center', gap: 8, marginBottom: 14, fontSize: 12, color: 'var(--ink-muted)' }}>
          <code style={verPill}>{skill.version}</code>
          <span>→</span>
          <code style={{ ...verPill, color: 'var(--accent)' }}>{newVer}</code>
        </div>
        <input ref={inputRef} value={msg} onChange={e => setMsg(e.target.value)}
          onKeyDown={e => { if (e.key === 'Enter' && msg) onCommit(msg); }}
          placeholder="描述此次变更…"
          style={{ width: '100%', padding: '10px 12px', border: '1px solid var(--border)', borderRadius: 6, fontSize: 13, fontFamily: 'inherit', background: 'var(--bg-sidebar)', outline: 'none', color: 'var(--ink)', boxSizing: 'border-box' }}/>
        <div style={{ display: 'flex', justifyContent: 'flex-end', gap: 8, marginTop: 18 }}>
          <Btn onClick={onClose} variant="ghost">取消</Btn>
          <Btn onClick={() => onCommit(msg || '更新')} variant="primary">提交 · 写入 CHANGELOG</Btn>
        </div>
      </div>
    </div>
  );
}

// ── Export modal ──────────────────────────────────────────

function ExportModal({ skill, onClose }) {
  const [tab, setTab] = useState('local');
  const MP = window.MockPaths || {
    projects: [{ name: 'my-project', path: '/Users/you/code/my-project' }],
    claudeDir: '/Users/you/.claude/skills',
    skillRoot: (n) => '/Users/you/.skillbook/skills/' + n,
    skillMd: (n) => '/Users/you/.skillbook/skills/' + n + '/SKILL.md',
  };
  const [targetProject, setTargetProject] = useState(MP.projects[0].path);

  const skillRoot = MP.skillRoot(skill.name);
  const skillMdPath = MP.skillMd(skill.name);

  const lnGlobal = `ln -s ${skillRoot} ${MP.claudeDir}/${skill.name}`;
  const lnProject = `ln -s ${skillRoot} ${targetProject}/.claude/skills/${skill.name}`;

  const copyToClipboard = async (text, label) => {
    try { await navigator.clipboard.writeText(text); toast(label + ' 已复制'); }
    catch { toast('复制失败'); }
  };

  const downloadZip = async () => {
    // Build a proper zip using a minimal pure-JS zip encoder (store-only, no compression).
    const flat = window.TreeUtil.flatten(skill.tree);
    const files = flat.map(f => ({
      path: skill.name + '/' + f.path.join('/'),
      content: f.node.content || ''
    }));
    const blob = makeZip(files);
    const a = document.createElement('a');
    a.href = URL.createObjectURL(blob);
    a.download = skill.name + '.zip';
    a.click();
    setTimeout(() => URL.revokeObjectURL(a.href), 1000);
    toast('已下载 ' + skill.name + '.zip');
  };

  const bashAll = useMemo(() => {
    const flat = window.TreeUtil.flatten(skill.tree);
    const lines = [`# 从脚本重建 ${skill.name}`, `mkdir -p ${skill.name}`, `cd ${skill.name}`];
    const dirs = new Set();
    flat.forEach(({ path }) => { for (let i = 1; i < path.length; i++) dirs.add(path.slice(0, i).join('/')); });
    [...dirs].sort().forEach(d => lines.push(`mkdir -p ${d}`));
    flat.forEach(({ path, node }) => {
      lines.push(`cat > ${path.join('/')} <<'SKILL_EOF'\n${node.content}\nSKILL_EOF`);
    });
    return lines.join('\n');
  }, [skill]);

  return (
    <div onClick={onClose} style={modalBackdrop}>
      <div onClick={e => e.stopPropagation()} style={{ ...modalCard, width: 680, maxHeight: '84vh', display: 'flex', flexDirection: 'column', padding: 0 }}>
        <header style={{ padding: '18px 22px 14px', borderBottom: '1px solid var(--border)', display: 'flex', alignItems: 'center', gap: 10 }}>
          <Icon name="download" size={16}/>
          <div style={{ fontSize: 15, fontWeight: 600 }}>使用 <code style={{ fontFamily: 'var(--font-mono)' }}>{skill.name}</code></div>
          <div style={{ flex: 1 }}/>
          <button onClick={onClose} style={{ background: 'none', border: 'none', cursor: 'pointer', color: 'var(--ink-faint)' }}><Icon name="x" size={14}/></button>
        </header>

        <div style={{ display: 'flex', gap: 2, padding: '6px 14px 0', borderBottom: '1px solid var(--border)' }}>
          {[
            ['local','本机使用','link'],
            ['share','分享他人','send'],
            ['raw','原始脚本','code'],
          ].map(([k, l]) => (
            <button key={k} onClick={() => setTab(k)} style={{
              padding: '9px 14px', background: 'none', border: 'none',
              borderBottom: tab === k ? '2px solid var(--ink)' : '2px solid transparent',
              marginBottom: -1, cursor: 'pointer', fontFamily: 'inherit',
              fontSize: 12.5, color: tab === k ? 'var(--ink)' : 'var(--ink-muted)',
              fontWeight: tab === k ? 600 : 400,
            }}>{l}</button>
          ))}
        </div>

        <div style={{ flex: 1, overflow: 'auto', minHeight: 0, padding: '20px 22px' }}>
          {tab === 'local' && (
            <div>
              <p style={{ fontSize: 12.5, color: 'var(--ink-muted)', margin: '0 0 14px', lineHeight: 1.6 }}>
                本地使用推荐：<b>软链接</b>（skill 更新自动同步）或<b>复制路径</b>（直接交给 Claude Code）。
              </p>

              <ActionCard
                title="复制 SKILL.md 绝对路径"
                subtitle="粘贴给 Claude Code，它会直接读取该路径"
                body={skillMdPath}
                action={() => copyToClipboard(skillMdPath, 'SKILL.md 路径')}
                cta="复制路径"
              />

              <ActionCard
                title="软链到 ~/.claude/skills（全局可用）"
                subtitle="所有项目都能用，skill 更新即时生效"
                body={lnGlobal}
                action={() => copyToClipboard(lnGlobal, '软链接命令')}
                cta="复制命令"
              />

              <div style={{ padding: 14, background: 'var(--bg-sidebar)', border: '1px solid var(--border)', borderRadius: 8, marginBottom: 10 }}>
                <div style={{ fontSize: 12.5, color: 'var(--ink)', fontWeight: 600, marginBottom: 4 }}>软链到某个项目 <code style={{ fontFamily: 'var(--font-mono)', fontSize: 11 }}>.claude/skills/</code></div>
                <div style={{ fontSize: 11.5, color: 'var(--ink-muted)', marginBottom: 10 }}>选择项目，我会生成对应命令</div>
                <div style={{ display: 'flex', gap: 6, flexWrap: 'wrap', marginBottom: 10 }}>
                  {MP.projects.map(p => (
                    <button key={p.path} onClick={() => setTargetProject(p.path)} style={{
                      padding: '5px 10px', borderRadius: 5,
                      background: targetProject === p.path ? 'var(--ink)' : 'var(--bg)',
                      color: targetProject === p.path ? 'var(--bg)' : 'var(--ink)',
                      border: '1px solid ' + (targetProject === p.path ? 'var(--ink)' : 'var(--border)'),
                      fontSize: 11.5, cursor: 'pointer', fontFamily: 'var(--font-mono)',
                    }}>{p.name}</button>
                  ))}
                </div>
                <pre style={{ ...codeBlock, marginBottom: 10 }}>{lnProject}</pre>
                <Btn onClick={() => copyToClipboard(lnProject, '项目软链接命令')} variant="primary"><Icon name="copy" size={11}/> 复制命令</Btn>
              </div>

              <div style={{ fontSize: 11, color: 'var(--ink-faint)', marginTop: 14, padding: '10px 12px', background: 'var(--bg-sidebar)', borderRadius: 6, lineHeight: 1.6 }}>
                💡 skill 在磁盘的位置（模拟）：<code style={{ fontFamily: 'var(--font-mono)' }}>{skillRoot}</code>
              </div>
            </div>
          )}

          {tab === 'share' && (
            <div>
              <p style={{ fontSize: 12.5, color: 'var(--ink-muted)', margin: '0 0 14px', lineHeight: 1.6 }}>
                分享给他人推荐：<b>ZIP 包</b>，对方解压后按本机使用流程链到 <code style={{ fontFamily: 'var(--font-mono)' }}>~/.claude/skills</code>。
              </p>

              <div style={{ padding: 14, background: 'var(--bg-sidebar)', border: '1px solid var(--border)', borderRadius: 8 }}>
                <div style={{ display: 'flex', alignItems: 'center', gap: 10, marginBottom: 10 }}>
                  <Icon name="archive" size={16} style={{ color: 'var(--accent)' }}/>
                  <div>
                    <div style={{ fontSize: 13, fontWeight: 600 }}>{skill.name}.zip</div>
                    <div style={{ fontSize: 11, color: 'var(--ink-faint)' }}>
                      {window.TreeUtil.flatten(skill.tree).length} 个文件 · v{skill.version}
                    </div>
                  </div>
                  <div style={{ flex: 1 }}/>
                  <Btn onClick={downloadZip} variant="accent"><Icon name="download" size={12}/> 下载 ZIP</Btn>
                </div>
                <pre style={{ ...codeBlock, maxHeight: 180, margin: 0 }}>{
                  window.TreeUtil.flatten(skill.tree).map(f => skill.name + '/' + f.path.join('/')).join('\n')
                }</pre>
              </div>

              <div style={{ marginTop: 16, padding: 14, background: 'var(--bg)', border: '1px dashed var(--border)', borderRadius: 8 }}>
                <div style={{ fontSize: 12, fontWeight: 600, marginBottom: 6 }}>对方使用步骤</div>
                <ol style={{ margin: 0, paddingLeft: 18, fontSize: 12, color: 'var(--ink-muted)', lineHeight: 1.7 }}>
                  <li>解压 <code style={{ fontFamily: 'var(--font-mono)', fontSize: 11 }}>{skill.name}.zip</code></li>
                  <li>移动到 <code style={{ fontFamily: 'var(--font-mono)', fontSize: 11 }}>~/.claude/skills/{skill.name}</code></li>
                  <li>Claude Code 自动识别</li>
                </ol>
              </div>
            </div>
          )}

          {tab === 'raw' && (
            <div>
              <p style={{ fontSize: 12.5, color: 'var(--ink-muted)', margin: '0 0 14px', lineHeight: 1.6 }}>
                不想下载文件时的备选：一段 bash 粘贴运行即可重建整个 skill 目录。
              </p>
              <pre style={{ ...codeBlock, maxHeight: 380 }}>{bashAll}</pre>
              <Btn onClick={() => copyToClipboard(bashAll, 'Bash 脚本')} variant="primary" style={{ marginTop: 12 }}><Icon name="copy" size={12}/> 复制脚本</Btn>
            </div>
          )}
        </div>
      </div>
    </div>
  );
}

function ActionCard({ title, subtitle, body, action, cta }) {
  return (
    <div style={{ padding: 14, background: 'var(--bg-sidebar)', border: '1px solid var(--border)', borderRadius: 8, marginBottom: 10 }}>
      <div style={{ fontSize: 12.5, color: 'var(--ink)', fontWeight: 600, marginBottom: 3 }}>{title}</div>
      <div style={{ fontSize: 11.5, color: 'var(--ink-muted)', marginBottom: 10 }}>{subtitle}</div>
      <div style={{ display: 'flex', alignItems: 'center', gap: 10 }}>
        <code style={{ flex: 1, padding: '8px 10px', background: 'var(--bg)', border: '1px solid var(--border-faint)', borderRadius: 5, fontFamily: 'var(--font-mono)', fontSize: 11.5, color: 'var(--ink)', overflow: 'hidden', textOverflow: 'ellipsis', whiteSpace: 'nowrap' }}>{body}</code>
        <Btn onClick={action} variant="primary"><Icon name="copy" size={11}/> {cta}</Btn>
      </div>
    </div>
  );
}

// Minimal ZIP encoder — store method (no compression). Enough for text skills.
function makeZip(files) {
  const encoder = new TextEncoder();
  const crcTable = (() => {
    const t = new Uint32Array(256);
    for (let n = 0; n < 256; n++) {
      let c = n;
      for (let k = 0; k < 8; k++) c = (c & 1) ? (0xEDB88320 ^ (c >>> 1)) : (c >>> 1);
      t[n] = c >>> 0;
    }
    return t;
  })();
  const crc32 = (buf) => {
    let c = 0xFFFFFFFF;
    for (let i = 0; i < buf.length; i++) c = crcTable[(c ^ buf[i]) & 0xFF] ^ (c >>> 8);
    return (c ^ 0xFFFFFFFF) >>> 0;
  };

  const fileRecords = [];
  const centralRecords = [];
  let offset = 0;

  for (const f of files) {
    const data = encoder.encode(f.content);
    const nameBytes = encoder.encode(f.path);
    const crc = crc32(data);
    const size = data.length;

    // Local file header (30 bytes + name)
    const lfh = new Uint8Array(30 + nameBytes.length);
    const dv = new DataView(lfh.buffer);
    dv.setUint32(0, 0x04034b50, true);  // sig
    dv.setUint16(4, 20, true);          // version
    dv.setUint16(6, 0, true);           // flags
    dv.setUint16(8, 0, true);           // method = store
    dv.setUint16(10, 0, true);          // time
    dv.setUint16(12, 0, true);          // date
    dv.setUint32(14, crc, true);
    dv.setUint32(18, size, true);
    dv.setUint32(22, size, true);
    dv.setUint16(26, nameBytes.length, true);
    dv.setUint16(28, 0, true);          // extra len
    lfh.set(nameBytes, 30);
    fileRecords.push(lfh, data);

    // Central directory record (46 + name)
    const cd = new Uint8Array(46 + nameBytes.length);
    const cv = new DataView(cd.buffer);
    cv.setUint32(0, 0x02014b50, true);
    cv.setUint16(4, 20, true);
    cv.setUint16(6, 20, true);
    cv.setUint16(8, 0, true);
    cv.setUint16(10, 0, true);
    cv.setUint16(12, 0, true);
    cv.setUint16(14, 0, true);
    cv.setUint32(16, crc, true);
    cv.setUint32(20, size, true);
    cv.setUint32(24, size, true);
    cv.setUint16(28, nameBytes.length, true);
    cv.setUint16(30, 0, true);
    cv.setUint16(32, 0, true);
    cv.setUint16(34, 0, true);
    cv.setUint16(36, 0, true);
    cv.setUint32(38, 0, true);
    cv.setUint32(42, offset, true);
    cd.set(nameBytes, 46);
    centralRecords.push(cd);

    offset += lfh.length + data.length;
  }

  // End of central directory
  const centralStart = offset;
  const centralSize = centralRecords.reduce((s, r) => s + r.length, 0);
  const eocd = new Uint8Array(22);
  const ev = new DataView(eocd.buffer);
  ev.setUint32(0, 0x06054b50, true);
  ev.setUint16(4, 0, true);
  ev.setUint16(6, 0, true);
  ev.setUint16(8, files.length, true);
  ev.setUint16(10, files.length, true);
  ev.setUint32(12, centralSize, true);
  ev.setUint32(16, centralStart, true);
  ev.setUint16(20, 0, true);

  return new Blob([...fileRecords, ...centralRecords, eocd], { type: 'application/zip' });
}

// Toast
function toast(msg) {
  const el = document.createElement('div');
  el.textContent = msg;
  el.style.cssText = 'position:fixed;bottom:24px;left:50%;transform:translateX(-50%);background:#18181b;color:#fff;padding:9px 16px;border-radius:6px;font-size:12.5px;font-family:inherit;z-index:1000;box-shadow:0 8px 24px rgba(0,0,0,0.2);opacity:0;transition:opacity 0.15s';
  document.body.appendChild(el);
  requestAnimationFrame(() => { el.style.opacity = '1'; });
  setTimeout(() => { el.style.opacity = '0'; setTimeout(() => el.remove(), 200); }, 1600);
}

const modalBackdrop = {
  position: 'fixed', inset: 0,
  background: 'rgba(0,0,0,0.3)', backdropFilter: 'blur(2px)',
  display: 'grid', placeItems: 'center',
  zIndex: 100,
};
const modalCard = {
  background: 'var(--bg)',
  borderRadius: 12,
  border: '1px solid var(--border)',
  padding: 22,
  boxShadow: '0 20px 50px rgba(0,0,0,0.2)',
};
const verPill = {
  fontFamily: 'var(--font-mono)', fontSize: 11,
  padding: '2px 7px', background: 'var(--bg-sidebar)',
  border: '1px solid var(--border)', borderRadius: 4,
  color: 'var(--ink-muted)',
};
const codeBlock = {
  margin: 0, padding: '12px 14px',
  background: 'var(--bg-sidebar)',
  border: '1px solid var(--border)',
  borderRadius: 6,
  fontFamily: 'var(--font-mono)', fontSize: 11.5, lineHeight: 1.6,
  color: 'var(--ink)', overflow: 'auto',
  whiteSpace: 'pre', maxHeight: 340,
};

Object.assign(window, { Library });
