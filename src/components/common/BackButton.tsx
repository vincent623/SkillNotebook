import { useUiStore } from "../../stores/ui-store";

export function BackButton() {
  const setCurrentScreen = useUiStore((state) => state.setCurrentScreen);

  return (
    <button
      className="back-button"
      onClick={() => setCurrentScreen("explorer")}
      type="button"
    >
      <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round">
        <path d="M19 12H5" />
        <path d="M12 19l-7-7 7-7" />
      </svg>
      <span>返回</span>
    </button>
  );
}
