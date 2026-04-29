import type { EvalOverallStatus, PackageStatus } from "../../types/models";

type BadgeTone = PackageStatus | EvalOverallStatus;

interface StatusBadgeProps {
  status: BadgeTone;
}

const toneLabel: Record<BadgeTone, string> = {
  draft: "草稿",
  evaluating: "评估中",
  validated: "已验证",
  needs_eval: "待评估",
  archived: "已归档",
  usable: "可用",
  needs_improvement: "待改进",
  problematic: "有问题",
};

export function StatusBadge({ status }: StatusBadgeProps) {
  return <span className={`status-badge status-${status}`}>{toneLabel[status]}</span>;
}
