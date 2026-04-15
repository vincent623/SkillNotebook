import type { EvalOverallStatus, PackageStatus } from "../../types/models";

type BadgeTone = PackageStatus | EvalOverallStatus;

interface StatusBadgeProps {
  status: BadgeTone;
}

const toneLabel: Record<BadgeTone, string> = {
  draft: "Draft",
  evaluating: "Evaluating",
  validated: "Validated",
  needs_eval: "Needs Eval",
  archived: "Archived",
  usable: "Usable",
  needs_improvement: "Needs Improvement",
  problematic: "Problematic",
};

export function StatusBadge({ status }: StatusBadgeProps) {
  return <span className={`status-badge status-${status}`}>{toneLabel[status]}</span>;
}
