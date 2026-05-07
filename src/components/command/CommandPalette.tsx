import { useCallback, useEffect, useMemo, useRef, useState } from "react";
import { useEditorStore } from "../../stores/editor-store";
import { useProjectStore } from "../../stores/project-store";
import { useUiStore } from "../../stores/ui-store";

interface CommandItem {
  id: string;
  title: string;
  subtitle: string;
  keywords: string;
  run: () => void | Promise<void>;
}

function commandMatches(command: CommandItem, query: string) {
  if (!query.trim()) return true;
  const needle = query.trim().toLowerCase();
  return [command.title, command.subtitle, command.keywords].join(" ").toLowerCase().includes(needle);
}

export function CommandPalette() {
  const inputRef = useRef<HTMLInputElement | null>(null);
  const [query, setQuery] = useState("");
  const [notice, setNotice] = useState<string | null>(null);
  const isOpen = useUiStore((state) => state.isCommandPaletteOpen);
  const close = useUiStore((state) => state.closeCommandPalette);
  const setCurrentScreen = useUiStore((state) => state.setCurrentScreen);
  const openVersionPanel = useUiStore((state) => state.openVersionPanel);
  const bootstrap = useProjectStore((state) => state.bootstrap);
  const selectedPackageId = useProjectStore((state) => state.selectedPackageId);
  const selectPackage = useProjectStore((state) => state.selectPackage);
  const runEval = useProjectStore((state) => state.runEval);
  const runTest = useProjectStore((state) => state.runTest);
  const isDirty = useEditorStore((state) => state.isDirty);

  const selectedPackage = bootstrap?.packages.find((item) => item.id === selectedPackageId) ?? null;
  const handleClose = useCallback(() => {
    setQuery("");
    setNotice(null);
    close();
  }, [close]);
  const canLeaveDirtyEditor = useCallback(() => {
    if (!isDirty) return true;
    return window.confirm("当前文件有未保存修改。继续会放弃这次修改。");
  }, [isDirty]);

  useEffect(() => {
    if (!isOpen) return;

    const frame = window.requestAnimationFrame(() => inputRef.current?.focus());
    return () => window.cancelAnimationFrame(frame);
  }, [isOpen]);

  const commands = useMemo<CommandItem[]>(() => {
    const items: CommandItem[] = [];

    if (bootstrap) {
      items.push(
        ...bootstrap.packages.map((pkg) => ({
          id: `open-${pkg.id}`,
          title: pkg.name,
          subtitle: `${pkg.slug} · v${pkg.currentVersion}`,
          keywords: [pkg.description, pkg.status, ...pkg.tags].join(" "),
          run: () => {
            if (!canLeaveDirtyEditor()) return;
            selectPackage(pkg.id);
            setCurrentScreen("notebook");
            handleClose();
          },
        })),
      );
    }

    items.push(
      {
        id: "draft-import",
        title: "导入 / 新建草稿",
        subtitle: "导入候选 skill，或创建临时草稿工作区",
        keywords: "draft import new skill",
        run: () => {
          if (!canLeaveDirtyEditor()) return;
          setCurrentScreen("draft");
          handleClose();
        },
      },
      {
        id: "settings",
        title: "打开设置",
        subtitle: "切换项目根目录、查看运行配置",
        keywords: "settings project root",
        run: () => {
          if (!canLeaveDirtyEditor()) return;
          setCurrentScreen("settings");
          handleClose();
        },
      },
    );

    if (selectedPackage) {
      items.push(
        {
          id: "copy-package-path",
          title: "复制当前 Skill 路径",
          subtitle: selectedPackage.rootPath,
          keywords: "copy path package skill",
          run: async () => {
            await navigator.clipboard?.writeText(selectedPackage.rootPath);
            setNotice("已复制当前 Skill 路径");
          },
        },
        {
          id: "quick-reference",
          title: "打开快速引用",
          subtitle: selectedPackage.name,
          keywords: "reference use copy link export",
          run: () => {
            handleClose();
            window.dispatchEvent(new CustomEvent("skillnotebook:open-reference"));
          },
        },
        {
          id: "run-eval",
          title: "运行当前 Skill 评估",
          subtitle: selectedPackage.name,
          keywords: "eval evaluate quality",
          run: () => {
            void runEval(selectedPackage.id);
            handleClose();
          },
        },
        {
          id: "run-test",
          title: "运行当前 Skill 测试",
          subtitle: selectedPackage.name,
          keywords: "test smoke check",
          run: () => {
            void runTest(selectedPackage.id);
            handleClose();
          },
        },
        {
          id: "open-version-panel",
          title: "打开版本与质量门禁",
          subtitle: `${selectedPackage.slug} · v${selectedPackage.currentVersion}`,
          keywords: "version save commit snapshot restore diff quality gate",
          run: () => {
            handleClose();
            openVersionPanel();
          },
        },
      );
    }

    return items;
  }, [bootstrap, canLeaveDirtyEditor, handleClose, openVersionPanel, runEval, runTest, selectPackage, selectedPackage, setCurrentScreen]);

  const visibleCommands = commands.filter((command) => commandMatches(command, query)).slice(0, 12);

  if (!isOpen) return null;

  return (
    <div
      className="command-overlay"
      onClick={handleClose}
      onKeyDown={(event) => {
        if (event.key === "Escape") handleClose();
      }}
      role="presentation"
    >
      <div className="command-panel" onClick={(event) => event.stopPropagation()}>
        <div className="command-input-wrap">
          <svg width="17" height="17" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="1.8" strokeLinecap="round" strokeLinejoin="round">
            <circle cx="11" cy="11" r="8" />
            <path d="m21 21-4.35-4.35" />
          </svg>
          <input
            aria-label="命令搜索"
            onChange={(event) => setQuery(event.target.value)}
            placeholder="搜索 skill 或命令"
            ref={inputRef}
            value={query}
          />
          <span>ESC</span>
        </div>

        {notice ? <div className="command-notice">{notice}</div> : null}

        <div className="command-list">
          {visibleCommands.length === 0 ? (
            <div className="command-empty">没有匹配的命令</div>
          ) : (
            visibleCommands.map((command) => (
              <button
                className="command-item"
                key={command.id}
                onClick={() => { void command.run(); }}
                type="button"
              >
                <span>
                  <strong>{command.title}</strong>
                  <small>{command.subtitle}</small>
                </span>
                <kbd>↵</kbd>
              </button>
            ))
          )}
        </div>
      </div>
    </div>
  );
}
