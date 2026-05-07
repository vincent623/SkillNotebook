import { create } from "zustand";
import {
  getAppBootstrap,
  restorePackageVersion,
  runPackageEval,
  runPackageTest,
  savePackageVersion,
} from "../services/tauri-api";
import type { AppBootstrap, PackageTestReport } from "../types/models";

type ProjectStatus = "idle" | "loading" | "ready" | "error";
type EvalRunStatus = "idle" | "submitting" | "success" | "error";
type TestRunStatus = "idle" | "submitting" | "success" | "error";
type VersionSaveStatus = "idle" | "submitting" | "success" | "error";
type VersionRestoreStatus = "idle" | "submitting" | "success" | "error";

interface ProjectStore {
  status: ProjectStatus;
  bootstrap: AppBootstrap | null;
  selectedPackageId: string | null;
  errorMessage: string | null;
  evalStatus: EvalRunStatus;
  evalError: string | null;
  lastEvalPackageId: string | null;
  lastEvalCreatedAt: string | null;
  testStatus: TestRunStatus;
  testError: string | null;
  lastTestPackageId: string | null;
  lastTestReport: PackageTestReport | null;
  versionSaveStatus: VersionSaveStatus;
  versionSaveError: string | null;
  lastVersionSavedPackageId: string | null;
  lastVersionSavedAt: string | null;
  versionRestoreStatus: VersionRestoreStatus;
  versionRestoreError: string | null;
  lastVersionRestoredPackageId: string | null;
  lastVersionRestoredVersionId: string | null;
  lastVersionRestoredVersionNumber: number | null;
  lastVersionRestoredAt: string | null;
  loadBootstrap: () => Promise<void>;
  refreshBootstrap: (preferredPackageId?: string | null) => Promise<void>;
  selectPackage: (packageId: string) => void;
  runEval: (packageId: string) => Promise<boolean>;
  runTest: (packageId: string) => Promise<boolean>;
  saveVersion: (packageId: string, note?: string | null) => Promise<boolean>;
  restoreVersion: (versionId: string, packageId: string) => Promise<boolean>;
}

async function refreshBootstrap(
  preferredPackageId?: string | null,
): Promise<Pick<ProjectStore, "bootstrap" | "selectedPackageId" | "status" | "errorMessage">> {
  const bootstrap = await getAppBootstrap();
  const selectedPackageId =
    (preferredPackageId &&
      bootstrap.packages.some((item) => item.id === preferredPackageId) &&
      preferredPackageId) ||
    bootstrap.selectedPackageId ||
    bootstrap.packages[0]?.id ||
    null;

  return {
    status: "ready",
    bootstrap,
    selectedPackageId,
    errorMessage: null,
  };
}

export const useProjectStore = create<ProjectStore>((set, get) => ({
  status: "idle",
  bootstrap: null,
  selectedPackageId: null,
  errorMessage: null,
  evalStatus: "idle",
  evalError: null,
  lastEvalPackageId: null,
  lastEvalCreatedAt: null,
  testStatus: "idle",
  testError: null,
  lastTestPackageId: null,
  lastTestReport: null,
  versionSaveStatus: "idle",
  versionSaveError: null,
  lastVersionSavedPackageId: null,
  lastVersionSavedAt: null,
  versionRestoreStatus: "idle",
  versionRestoreError: null,
  lastVersionRestoredPackageId: null,
  lastVersionRestoredVersionId: null,
  lastVersionRestoredVersionNumber: null,
  lastVersionRestoredAt: null,
  loadBootstrap: async () => {
    set({ status: "loading", errorMessage: null });

    try {
      set(await refreshBootstrap(get().selectedPackageId));
    } catch (error) {
      set({
        status: "error",
        errorMessage: error instanceof Error ? error.message : "Unknown bootstrap error",
      });
    }
  },
  refreshBootstrap: async (preferredPackageId) => {
    try {
      set(await refreshBootstrap(preferredPackageId ?? get().selectedPackageId));
    } catch (error) {
      set({
        status: "error",
        errorMessage: error instanceof Error ? error.message : "Unknown refresh error",
      });
    }
  },
  selectPackage: (packageId) => {
    set({
      selectedPackageId: packageId,
      evalError: null,
      testError: null,
      versionRestoreError: null,
    });
  },
  runEval: async (packageId) => {
    set({
      evalStatus: "submitting",
      evalError: null,
      lastEvalPackageId: packageId,
      lastEvalCreatedAt: null,
    });

    try {
      const report = await runPackageEval(packageId);
      const refreshedState = await refreshBootstrap(packageId);
      set({
        ...refreshedState,
        evalStatus: "success",
        evalError: null,
        lastEvalPackageId: packageId,
        lastEvalCreatedAt: report.createdAt,
      });
      return true;
    } catch (error) {
      set({
        evalStatus: "error",
        evalError: error instanceof Error ? error.message : "Eval run failed.",
        lastEvalPackageId: packageId,
        lastEvalCreatedAt: null,
      });
      return false;
    }
  },
  runTest: async (packageId) => {
    set({
      testStatus: "submitting",
      testError: null,
      lastTestPackageId: packageId,
      lastTestReport: null,
    });

    try {
      const report = await runPackageTest(packageId);
      set({
        testStatus: "success",
        testError: null,
        lastTestPackageId: packageId,
        lastTestReport: report,
      });
      return report.status === "passed";
    } catch (error) {
      set({
        testStatus: "error",
        testError: error instanceof Error ? error.message : "Package test run failed.",
        lastTestPackageId: packageId,
        lastTestReport: null,
      });
      return false;
    }
  },
  saveVersion: async (packageId, note) => {
    set({
      versionSaveStatus: "submitting",
      versionSaveError: null,
      lastVersionSavedPackageId: packageId,
      lastVersionSavedAt: null,
    });

    try {
      const version = await savePackageVersion(packageId, note ?? null);
      const refreshedState = await refreshBootstrap(packageId);
      set({
        ...refreshedState,
        versionSaveStatus: "success",
        versionSaveError: null,
        lastVersionSavedPackageId: packageId,
        lastVersionSavedAt: version.createdAt,
      });
      return true;
    } catch (error) {
      set({
        versionSaveStatus: "error",
        versionSaveError: error instanceof Error ? error.message : "Version save failed.",
        lastVersionSavedPackageId: packageId,
        lastVersionSavedAt: null,
      });
      return false;
    }
  },
  restoreVersion: async (versionId, packageId) => {
    set({
      versionRestoreStatus: "submitting",
      versionRestoreError: null,
      lastVersionRestoredPackageId: packageId,
      lastVersionRestoredVersionId: versionId,
      lastVersionRestoredVersionNumber: null,
      lastVersionRestoredAt: null,
    });

    try {
      const restoredPackage = await restorePackageVersion(versionId);
      const refreshedState = await refreshBootstrap(restoredPackage.id);
      set({
        ...refreshedState,
        versionRestoreStatus: "success",
        versionRestoreError: null,
        lastVersionRestoredPackageId: restoredPackage.id,
        lastVersionRestoredVersionId: versionId,
        lastVersionRestoredVersionNumber: restoredPackage.currentVersion,
        lastVersionRestoredAt: restoredPackage.updatedAt,
      });
      return true;
    } catch (error) {
      set({
        versionRestoreStatus: "error",
        versionRestoreError: error instanceof Error ? error.message : "Version restore failed.",
        lastVersionRestoredPackageId: packageId,
        lastVersionRestoredVersionId: versionId,
        lastVersionRestoredVersionNumber: null,
        lastVersionRestoredAt: null,
      });
      return false;
    }
  },
}));
