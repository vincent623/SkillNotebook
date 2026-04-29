// Generator — 42skill-style wizard: describe → generate → preview → save.

function Generator({ onSave, onClose, initialPrompt = '' }) {
  const [step, setStep] = useState('input'); // input | generating | preview
  const [mode, setMode] = useState('text'); // text | files | url
  const [prompt, setPrompt] = useState(initialPrompt);
  const [droppedFiles, setDroppedFiles] = useState([]);
  const [url, setUrl] = useState('');
  const [skill, setSkill] = useState(null);
  const [selectedPath, setSelectedPath] = useState(['SKILL.md']);

  const synthesizePrompt = () => {
    if (mode === 'text') return prompt;
    if (mode === 'files') {
      const names = droppedFiles.map(f => f.name).join('、');
      return `基于以下本地文件/目录整理出一个可复用 skill：${names}。归纳其中的流程、模板与规范，使 Claude 能在类似任务里复刻同样的风格与步骤。`;
    }
    if (mode === 'url') {
      return `抓取 ${url} 的内容并提炼为 skill：把其中的指南、清单、示例归入 references/，把可自动化的步骤写成 scripts/。`;
    }
    return '';
  };

  const canGo = mode === 'text' ? !!prompt.trim() : mode === 'files' ? droppedFiles.length > 0 : !!url.trim();

  const generate = async () => {
    if (!canGo) return;
    setStep('generating');
    await new Promise(r => setTimeout(r, 900));
    const finalPrompt = synthesizePrompt();
    const seed = mode === 'files' ? droppedFiles[0]?.name?.replace(/\.[^.]+$/, '') : (mode === 'url' ? hostFromUrl(url) : '');
    const result = window.MockAI.generate(finalPrompt, seed ? { name: window.MockAI.nameFrom(seed) } : {});
    // If files mode, inject a reference file showing the source files used
    if (mode === 'files' && droppedFiles.length > 0) {
      result.tree.children.find(c => c.name === 'references').children.unshift({
        type: 'file', name: 'source-inventory.md',
        content: `# 原始资料\n\n本 skill 从以下本地文件/目录提炼：\n\n${droppedFiles.map(f => `- \`${f.path || f.name}\`${f.size ? ` (${formatSize(f.size)})` : ''}`).join('\n')}\n`
      });
    }
    if (mode === 'url' && url) {
      result.tree.children.find(c => c.name === 'references').children.unshift({
        type: 'file', name: 'source.md',
        content: `# 来源\n\n本 skill 基于以下网页提炼：\n\n- ${url}\n\n> 抓取时间：${new Date().toISOString().slice(0, 10)}\n`
      });
    }
    setSkill(result);
    setSelectedPath(['SKILL.md']);
    setStep('preview');
  };

  const regenerate = () => {
    const result = window.MockAI.generate(synthesizePrompt(), { name: skill.name });
    setSkill(result);
  };

  const renameSkill = (newName) => {
    const kebab = newName.toLowerCase().replace(/[^a-z0-9]+/g, '-').replace(/^-|-$/g, '');
    if (!kebab) return;
    setSkill({ ...skill, name: kebab, displayName: window.MockAI._title(kebab), tree: { ...skill.tree, name: kebab } });
  };

  const regenDescription = () => {
    const skillMd = skill.tree.children.find(c => c.name === 'SKILL.md');
    const { body } = window.TreeUtil.parseFrontmatter(skillMd.content);
    const newDesc = window.MockAI.deriveDescription(body);
    const updated = skillMd.content.replace(/description: .+/, 'description: ' + newDesc);
    const tree = JSON.parse(JSON.stringify(skill.tree));
    tree.children.find(c => c.name === 'SKILL.md').content = updated;
    setSkill({ ...skill, description: newDesc, tree });
  };

  return (
    <div style={modalBackdrop}>
      <div onClick={e => e.stopPropagation()} style={{
        ...modalCard, width: step === 'preview' ? 960 : 620,
        maxHeight: '86vh', display: 'flex', flexDirection: 'column',
        padding: step === 'preview' ? 0 : 28,
        transition: 'width 0.2s',
      }}>
        {step === 'input' && (
          <InputStep
            mode={mode} setMode={setMode}
            prompt={prompt} setPrompt={setPrompt}
            droppedFiles={droppedFiles} setDroppedFiles={setDroppedFiles}
            url={url} setUrl={setUrl}
            canGo={canGo}
            onGo={generate} onClose={onClose}
          />
        )}
        {step === 'generating' && <GeneratingStep prompt={synthesizePrompt()}/>}
        {step === 'preview' && (
          <PreviewStep
            skill={skill}
            selectedPath={selectedPath}
            setSelectedPath={setSelectedPath}
            onRegen={regenerate}
            onRename={renameSkill}
            onRegenDesc={regenDescription}
            onBack={() => setStep('input')}
            onSave={() => { onSave(skill); onClose(); }}
            onClose={onClose}
          />
        )}
      </div>
    </div>
  );
}

function hostFromUrl(url) {
  try { return new URL(url).hostname.replace(/^www\./, '').split('.')[0]; }
  catch { return 'from-url'; }
}
function formatSize(n) {
  if (n < 1024) return n + ' B';
  if (n < 1024*1024) return (n/1024).toFixed(1) + ' KB';
  return (n/(1024*1024)).toFixed(1) + ' MB';
}

// ── Step 1: input ─────────────────────────────────────────

function InputStep({ mode, setMode, prompt, setPrompt, droppedFiles, setDroppedFiles, url, setUrl, canGo, onGo, onClose }) {
  return (
    <div>
      <div style={{ display: 'flex', alignItems: 'center', gap: 10, marginBottom: 6 }}>
        <div style={{ width: 26, height: 26, borderRadius: 7, background: 'var(--accent)', color: '#fff', display: 'grid', placeItems: 'center' }}>
          <Icon name="wand" size={14}/>
        </div>
        <div style={{ fontSize: 16, fontWeight: 600 }}>生成新 Skill</div>
        <button onClick={onClose} style={{ marginLeft: 'auto', background: 'none', border: 'none', cursor: 'pointer', color: 'var(--ink-faint)' }}><Icon name="x" size={14}/></button>
      </div>
      <div style={{ fontSize: 13, color: 'var(--ink-muted)', marginBottom: 18, lineHeight: 1.5 }}>
        选择一种输入源，AI 会生成完整目录结构。
      </div>

      {/* Mode tabs */}
      <div style={{ display: 'flex', gap: 4, marginBottom: 16, padding: 3, background: 'var(--bg-sidebar)', borderRadius: 8, border: '1px solid var(--border)' }}>
        {[
          ['text', '文字描述', 'edit'],
          ['files', '本地文件/目录', 'folder'],
          ['url', '网页链接', 'link'],
        ].map(([k, l, icon]) => (
          <button key={k} onClick={() => setMode(k)} style={{
            flex: 1, padding: '9px 10px',
            background: mode === k ? 'var(--bg)' : 'transparent',
            border: 'none', borderRadius: 5,
            boxShadow: mode === k ? '0 1px 2px rgba(0,0,0,0.04)' : 'none',
            cursor: 'pointer', fontFamily: 'inherit',
            fontSize: 12.5, color: mode === k ? 'var(--ink)' : 'var(--ink-muted)',
            fontWeight: mode === k ? 600 : 400,
            display: 'inline-flex', alignItems: 'center', justifyContent: 'center', gap: 6,
          }}>
            <Icon name={icon === 'edit' ? 'edit' : icon === 'link' ? 'git' : 'folder'} size={12}/> {l}
          </button>
        ))}
      </div>

      {mode === 'text' && <TextMode prompt={prompt} setPrompt={setPrompt} onGo={onGo}/>}
      {mode === 'files' && <FileMode files={droppedFiles} setFiles={setDroppedFiles}/>}
      {mode === 'url' && <UrlMode url={url} setUrl={setUrl} onGo={onGo}/>}

      <div style={{ display: 'flex', justifyContent: 'flex-end', gap: 8, marginTop: 20 }}>
        <Btn onClick={onClose} variant="ghost">取消</Btn>
        <Btn onClick={onGo} variant="accent" disabled={!canGo}>
          <Icon name="sparkle" size={12}/> 生成 · <kbd style={{ fontSize: 10, marginLeft: 4, fontFamily: 'var(--font-mono)', opacity: 0.7 }}>⌘↵</kbd>
        </Btn>
      </div>
    </div>
  );
}

function TextMode({ prompt, setPrompt, onGo }) {
  const ref = useRef(null);
  useEffect(() => { ref.current?.focus(); }, []);
  const examples = [
    '帮我审查 TypeScript PR 的类型安全问题',
    '从用户上传的发票 PDF 提取金额、日期、供应商',
    '把任意会议记录转成结构化纪要（决议/行动项/待办）',
    '按我们的品牌指南重写营销文案'
  ];
  return (
    <>
      <textarea ref={ref} value={prompt} onChange={e => setPrompt(e.target.value)}
        onKeyDown={e => { if (e.key === 'Enter' && (e.metaKey || e.ctrlKey)) onGo(); }}
        placeholder="例如：审查 TypeScript PR 的类型安全问题，并根据我们的 tsconfig 严格度给出修复建议…"
        rows={5}
        style={{ width: '100%', padding: '14px 16px', border: '1px solid var(--border)', borderRadius: 8,
          fontSize: 14, fontFamily: 'inherit', lineHeight: 1.6,
          background: 'var(--bg-sidebar)', outline: 'none', color: 'var(--ink)', resize: 'vertical', boxSizing: 'border-box' }}/>
      <div style={{ marginTop: 12 }}>
        <div style={{ fontSize: 10.5, color: 'var(--ink-faint)', textTransform: 'uppercase', letterSpacing: '0.08em', fontWeight: 600, marginBottom: 8 }}>示例</div>
        <div style={{ display: 'flex', flexWrap: 'wrap', gap: 6 }}>
          {examples.map(ex => (
            <button key={ex} onClick={() => setPrompt(ex)} style={{ padding: '6px 10px', background: 'var(--bg-sidebar)',
              border: '1px solid var(--border)', borderRadius: 99, fontSize: 11.5, color: 'var(--ink-muted)',
              cursor: 'pointer', fontFamily: 'inherit', textAlign: 'left' }}>{ex}</button>
          ))}
        </div>
      </div>
    </>
  );
}

function FileMode({ files, setFiles }) {
  const [drag, setDrag] = useState(false);
  const inputRef = useRef(null);
  const onDrop = (e) => {
    e.preventDefault(); setDrag(false);
    const list = [...(e.dataTransfer.files || [])].map(f => ({
      name: f.name, size: f.size, path: f.webkitRelativePath || f.name, type: f.type
    }));
    setFiles([...files, ...list]);
  };
  const onPick = (e) => {
    const list = [...(e.target.files || [])].map(f => ({
      name: f.name, size: f.size, path: f.webkitRelativePath || f.name, type: f.type
    }));
    setFiles([...files, ...list]);
  };
  return (
    <>
      <div
        onDragOver={e => { e.preventDefault(); setDrag(true); }}
        onDragLeave={() => setDrag(false)}
        onDrop={onDrop}
        onClick={() => inputRef.current?.click()}
        style={{
          padding: '32px 20px', borderRadius: 8, textAlign: 'center',
          border: '1.5px dashed ' + (drag ? 'var(--accent)' : 'var(--border)'),
          background: drag ? '#fef7f0' : 'var(--bg-sidebar)',
          cursor: 'pointer', transition: 'all 0.12s',
        }}>
        <Icon name="folder" size={26} style={{ color: drag ? 'var(--accent)' : 'var(--ink-faint)' }}/>
        <div style={{ marginTop: 10, fontSize: 13, color: 'var(--ink)', fontWeight: 500 }}>
          拖拽文件/目录到此处，或点击选择
        </div>
        <div style={{ marginTop: 4, fontSize: 11.5, color: 'var(--ink-faint)' }}>
          支持 .md / .pdf / .txt / 整个目录
        </div>
        <input ref={inputRef} type="file" multiple onChange={onPick}
          style={{ display: 'none' }} webkitdirectory="" directory=""/>
      </div>
      <input type="file" multiple onChange={onPick} style={{ display: 'none' }}/>
      <div style={{ marginTop: 10, display: 'flex', gap: 8 }}>
        <Btn onClick={() => { const el = document.createElement('input'); el.type='file'; el.multiple=true; el.onchange=onPick; el.click(); }} variant="secondary">
          <Icon name="file" size={12}/> 选择文件
        </Btn>
        <Btn onClick={() => inputRef.current?.click()} variant="secondary">
          <Icon name="folder" size={12}/> 选择目录
        </Btn>
        {files.length > 0 && <Btn onClick={() => setFiles([])} variant="ghost">清空</Btn>}
      </div>

      {files.length > 0 && (
        <div style={{ marginTop: 14, maxHeight: 180, overflow: 'auto', border: '1px solid var(--border)', borderRadius: 6, background: 'var(--bg)' }}>
          {files.map((f, i) => (
            <div key={i} style={{ display: 'flex', alignItems: 'center', gap: 8, padding: '7px 12px', borderBottom: i < files.length - 1 ? '1px solid var(--border-faint)' : 'none', fontSize: 12 }}>
              <Icon name="file" size={11} style={{ color: 'var(--ink-faint)' }}/>
              <code style={{ fontFamily: 'var(--font-mono)', fontSize: 11.5, color: 'var(--ink)', flex: 1, overflow: 'hidden', textOverflow: 'ellipsis', whiteSpace: 'nowrap' }}>{f.path || f.name}</code>
              <span style={{ fontSize: 10.5, color: 'var(--ink-faint)' }}>{formatSize(f.size)}</span>
              <button onClick={() => setFiles(files.filter((_, j) => j !== i))} style={{ background: 'none', border: 'none', cursor: 'pointer', color: 'var(--ink-faint)', padding: 2 }}>
                <Icon name="x" size={10}/>
              </button>
            </div>
          ))}
        </div>
      )}
    </>
  );
}

function UrlMode({ url, setUrl, onGo }) {
  const examples = [
    'https://docs.anthropic.com/claude/docs/building-with-claude',
    'https://example.com/style-guide',
    'https://github.com/anthropics/anthropic-cookbook'
  ];
  return (
    <>
      <div style={{ display: 'flex', gap: 6 }}>
        <input value={url} onChange={e => setUrl(e.target.value)}
          onKeyDown={e => { if (e.key === 'Enter' && (e.metaKey || e.ctrlKey)) onGo(); }}
          placeholder="https://…"
          style={{ flex: 1, padding: '11px 14px', border: '1px solid var(--border)', borderRadius: 8,
            fontSize: 13.5, fontFamily: 'var(--font-mono)', background: 'var(--bg-sidebar)',
            outline: 'none', color: 'var(--ink)' }}/>
      </div>
      <div style={{ fontSize: 11.5, color: 'var(--ink-muted)', marginTop: 10, lineHeight: 1.6 }}>
        AI 会抓取页面正文、提取结构化要点，生成对应的 SKILL.md 与 references/。<br/>
        <span style={{ color: 'var(--ink-faint)' }}>（演示环境：本机不会真的发起网络请求，而是模拟抓取结果）</span>
      </div>
      <div style={{ marginTop: 14 }}>
        <div style={{ fontSize: 10.5, color: 'var(--ink-faint)', textTransform: 'uppercase', letterSpacing: '0.08em', fontWeight: 600, marginBottom: 8 }}>示例</div>
        <div style={{ display: 'flex', flexDirection: 'column', gap: 5 }}>
          {examples.map(ex => (
            <button key={ex} onClick={() => setUrl(ex)} style={{ padding: '7px 10px', background: 'var(--bg-sidebar)',
              border: '1px solid var(--border)', borderRadius: 6, fontSize: 11.5, color: 'var(--ink-muted)',
              cursor: 'pointer', fontFamily: 'var(--font-mono)', textAlign: 'left' }}>{ex}</button>
          ))}
        </div>
      </div>
    </>
  );
}

// ── Step 2: generating ────────────────────────────────────

function GeneratingStep({ prompt }) {
  const [tick, setTick] = useState(0);
  useEffect(() => {
    const id = setInterval(() => setTick(t => t + 1), 220);
    return () => clearInterval(id);
  }, []);
  const steps = [
    '解析意图…',
    '推导 skill 名字…',
    '撰写 SKILL.md…',
    '生成 references/…',
    '生成 scripts/…',
    '校验结构…',
  ];
  return (
    <div style={{ padding: '20px 4px' }}>
      <div style={{ display: 'flex', alignItems: 'center', gap: 10, marginBottom: 22 }}>
        <div style={{ width: 26, height: 26, borderRadius: 7, background: 'var(--accent)', color: '#fff', display: 'grid', placeItems: 'center', animation: 'pulse 1.4s ease-in-out infinite' }}>
          <Icon name="sparkle" size={14}/>
        </div>
        <div>
          <div style={{ fontSize: 14, fontWeight: 600 }}>AI 正在生成</div>
          <div style={{ fontSize: 12, color: 'var(--ink-muted)', marginTop: 2 }}>"{prompt.slice(0, 80)}{prompt.length > 80 ? '…' : ''}"</div>
        </div>
      </div>
      <style>{`@keyframes pulse { 0%,100% { opacity: 0.8 } 50% { opacity: 0.4 } }`}</style>
      {steps.map((s, i) => (
        <div key={i} style={{ display: 'flex', alignItems: 'center', gap: 10, padding: '6px 0', fontSize: 12.5, color: i <= tick ? 'var(--ink)' : 'var(--ink-faint)', fontFamily: 'var(--font-mono)' }}>
          <Icon name={i < tick ? 'check' : 'dot'} size={11} style={{ color: i < tick ? '#15803d' : 'var(--ink-faint)' }}/>
          {s}
        </div>
      ))}
    </div>
  );
}

// ── Step 3: preview ───────────────────────────────────────

function PreviewStep({ skill, selectedPath, setSelectedPath, onRegen, onRename, onRegenDesc, onBack, onSave, onClose }) {
  const file = window.TreeUtil.findByPath(skill.tree, selectedPath);
  const validation = window.TreeUtil.validate(skill);
  const [renameOpen, setRenameOpen] = useState(false);

  return (
    <>
      <header style={{ padding: '18px 22px 12px', borderBottom: '1px solid var(--border)', display: 'flex', alignItems: 'center', gap: 10 }}>
        <Icon name="wand" size={14} style={{ color: 'var(--accent)' }}/>
        <div style={{ fontSize: 13, color: 'var(--ink-muted)' }}>已生成</div>
        {renameOpen ? (
          <input autoFocus defaultValue={skill.name}
            onBlur={(e) => { onRename(e.target.value); setRenameOpen(false); }}
            onKeyDown={e => { if (e.key === 'Enter') { onRename(e.target.value); setRenameOpen(false); } }}
            style={{ fontFamily: 'var(--font-mono)', fontSize: 14, fontWeight: 600,
              padding: '3px 8px', border: '1px solid var(--border)', borderRadius: 5, outline: 'none',
              background: 'var(--bg)', color: 'var(--ink)' }}/>
        ) : (
          <button onClick={() => setRenameOpen(true)} style={{ background: 'none', border: 'none', cursor: 'text', padding: 0 }}>
            <code style={{ fontFamily: 'var(--font-mono)', fontSize: 14, fontWeight: 600, color: 'var(--ink)' }}>{skill.name}/</code>
          </button>
        )}
        <ValidationBadge v={validation}/>
        <div style={{ flex: 1 }}/>
        <Btn onClick={onBack} variant="ghost">‹ 重新描述</Btn>
        <Btn onClick={onRegen} variant="secondary"><Icon name="sparkle" size={12}/> 重新生成</Btn>
        <Btn onClick={onSave} variant="accent">保存到库</Btn>
        <button onClick={onClose} style={{ background: 'none', border: 'none', cursor: 'pointer', color: 'var(--ink-faint)', padding: 2, marginLeft: 4 }}><Icon name="x" size={14}/></button>
      </header>

      <div style={{ display: 'flex', flex: 1, overflow: 'hidden' }}>
        {/* Tree */}
        <aside style={{ width: 240, borderRight: '1px solid var(--border)', background: 'var(--bg-sidebar)', overflow: 'auto', padding: '10px 8px' }}>
          <TreeOutline node={skill.tree} path={[]} selectedPath={selectedPath} onSelect={setSelectedPath} depth={0}/>
        </aside>

        {/* Content */}
        <main style={{ flex: 1, overflow: 'auto', background: 'var(--bg)' }}>
          {file && file.type === 'file' ? (
            <div>
              <div style={{ padding: '12px 20px', borderBottom: '1px solid var(--border)', fontSize: 12, display: 'flex', alignItems: 'center', gap: 10 }}>
                <code style={{ fontFamily: 'var(--font-mono)', color: 'var(--ink)' }}>{[skill.name, ...selectedPath].join('/')}</code>
                <span style={{ color: 'var(--ink-faint)' }}>· {window.TreeUtil.wordCount(file.content)} 词</span>
                {selectedPath.join('/') === 'SKILL.md' && (
                  <Btn onClick={onRegenDesc} variant="ghost" style={{ marginLeft: 'auto' }}><Icon name="sparkle" size={11}/> 重写 description</Btn>
                )}
              </div>
              <div style={{ padding: '22px 28px' }}>
                {window.TreeUtil.kind(file.name) === 'skill' || window.TreeUtil.kind(file.name) === 'markdown' || window.TreeUtil.kind(file.name) === 'changelog' ? (
                  <>
                    {file.name === 'SKILL.md' && <FrontmatterCard content={file.content}/>}
                    <Markdown source={file.content}/>
                  </>
                ) : (
                  <pre style={{ margin: 0, fontFamily: 'var(--font-mono)', fontSize: 12.5, lineHeight: 1.7, whiteSpace: 'pre-wrap', color: 'var(--ink)' }}>{file.content}</pre>
                )}
              </div>
            </div>
          ) : (
            <div style={{ padding: 40, color: 'var(--ink-faint)' }}>选择一个文件查看</div>
          )}
        </main>
      </div>
    </>
  );
}

function TreeOutline({ node, path, selectedPath, onSelect, depth }) {
  const active = path.join('/') === selectedPath.join('/');
  if (node.type === 'file') {
    return (
      <button onClick={() => onSelect(path)}
        style={{
          display: 'flex', alignItems: 'center', gap: 8,
          width: '100%', padding: '5px 8px', marginBottom: 1,
          marginLeft: depth * 12,
          background: active ? 'var(--bg-active)' : 'transparent',
          border: 'none', borderRadius: 5, cursor: 'pointer',
          fontSize: 12, color: 'var(--ink)', fontFamily: 'inherit',
          textAlign: 'left',
        }}
        onMouseEnter={e => { if (!active) e.currentTarget.style.background = 'var(--bg-hover)'; }}
        onMouseLeave={e => { if (!active) e.currentTarget.style.background = 'transparent'; }}>
        <Icon name={window.TreeUtil.iconFor(node)} size={12} style={{ color: node.name === 'SKILL.md' ? 'var(--accent)' : 'var(--ink-faint)' }}/>
        <code style={{ fontFamily: 'var(--font-mono)', fontWeight: node.name === 'SKILL.md' ? 600 : 400 }}>{node.name}</code>
      </button>
    );
  }
  return (
    <div>
      <div style={{ display: 'flex', alignItems: 'center', gap: 6, padding: '5px 8px', marginLeft: depth * 12, fontSize: 11.5, color: 'var(--ink-muted)' }}>
        <Icon name="folder" size={11} style={{ color: 'var(--accent)' }}/>
        <span style={{ fontFamily: 'var(--font-mono)', fontWeight: depth === 0 ? 600 : 500 }}>{node.name}{depth === 0 ? '/' : ''}</span>
      </div>
      {node.children?.map(c => (
        <TreeOutline key={c.name} node={c} path={[...path, c.name]} selectedPath={selectedPath} onSelect={onSelect} depth={depth + 1}/>
      ))}
    </div>
  );
}

Object.assign(window, { Generator });
