import { PackageRow } from "../../components/explorer/PackageRow";
import { useUiStore } from "../../stores/ui-store";
import { useProjectStore } from "../../stores/project-store";

export function ExplorerView() {
  const bootstrap = useProjectStore((state) => state.bootstrap);
  const selectPackage = useProjectStore((state) => state.selectPackage);
  const setCurrentScreen = useUiStore((state) => state.setCurrentScreen);

  if (!bootstrap) {
    return (
      <section className="explorer-loading">
        <p>正在加载技能仓库...</p>
      </section>
    );
  }

  const packages = [...bootstrap.packages].sort(
    (a, b) => new Date(b.updatedAt).getTime() - new Date(a.updatedAt).getTime(),
  );

  return (
    <section className="explorer-view">
      <div className="explorer-path-bar">
        <span className="explorer-path">{bootstrap.projectRoot.rootPath}</span>
        <button
          className="button-primary"
          onClick={() => setCurrentScreen("draft")}
          type="button"
        >
          + 导入
        </button>
      </div>
      <div className="explorer-list">
        {packages.length === 0 ? (
          <div className="explorer-empty">
            <p className="muted">当前根目录下还没有 `.skills/` 内容。点击「+ 导入」收纳第一个 skill。</p>
          </div>
        ) : (
          packages.map((item) => {
            const evalReport = bootstrap.evalReports.find((r) => r.packageId === item.id);
            return (
              <PackageRow
                key={item.id}
                item={item}
                evalReport={evalReport}
                onClick={() => {
                  selectPackage(item.id);
                  setCurrentScreen("notebook");
                }}
              />
            );
          })
        )}
      </div>
    </section>
  );
}
