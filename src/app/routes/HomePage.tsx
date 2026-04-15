import { useUiStore } from "../../stores/ui-store";
import { useWorkspaceStore } from "../../stores/workspace-store";

export function HomePage() {
  const setCurrentScreen = useUiStore((state) => state.setCurrentScreen);
  const status = useWorkspaceStore((state) => state.status);
  const bootstrap = useWorkspaceStore((state) => state.bootstrap);

  return (
    <section className="home-page">
      <div className="hero-card">
        <p className="eyebrow">macOS local skill workbench</p>
        <h1>Skill Notebook</h1>
        <p className="hero-copy">
          A local-first workspace for editing, evaluating, and versioning skill packages like
          durable knowledge assets.
        </p>
        <div className="button-row">
          <button
            className="button-primary"
            onClick={() => {
              setCurrentScreen("workspace");
            }}
            type="button"
          >
            Open Workspace
          </button>
          <button
            className="button-secondary"
            onClick={() => {
              setCurrentScreen("settings");
            }}
            type="button"
          >
            Review Technical Direction
          </button>
        </div>
      </div>

      <div className="home-grid">
        <article className="content-card">
          <div className="section-head">
            <h3>Core loop</h3>
            <span className="section-meta">V1</span>
          </div>
          <p className="body-copy">Find → Create → Evaluate → Version</p>
        </article>

        <article className="content-card">
          <div className="section-head">
            <h3>Current bootstrap</h3>
            <span className="section-meta">{status}</span>
          </div>
          <p className="body-copy">
            {bootstrap
              ? `${bootstrap.packages.length} packages loaded from the current workspace bootstrap.`
              : "The app shell is ready to load a local workspace payload from Rust."}
          </p>
        </article>

        <article className="content-card">
          <div className="section-head">
            <h3>Source chat</h3>
            <span className="section-meta">Product seed</span>
          </div>
          <p className="mono-text">chats/202604131741.md</p>
          <p className="muted">
            The product and technical direction in this repository were distilled from that
            conversation.
          </p>
        </article>
      </div>
    </section>
  );
}
