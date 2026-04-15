import { create } from "zustand";
import {
  createPackageFromNl,
  getAppBootstrap,
  runPackageEval,
  savePackageVersion,
} from "../services/tauri-api";
import type { AppBootstrap, CreatePackageFromNlResponse } from "../types/models";

type WorkspaceStatus = "idle" | "loading" | "ready" | "error";
type CreatePackageStatus = "idle" | "submitting" | "success" | "error";
type EvalRunStatus = "idle" | "submitting" | "success" | "error";
type VersionSaveStatus = "idle" | "submitting" | "success" | "error";

interface WorkspaceStore {
  status: WorkspaceStatus;
  bootstrap: AppBootstrap | null;
  selectedPackageId: string | null;
  errorMessage: string | null;
  createComposerOpen: boolean;
  createPrompt: string;
  createContext: string;
  createStatus: CreatePackageStatus;
  createError: string | null;
  lastCreateResult: CreatePackageFromNlResponse | null;
  evalStatus: EvalRunStatus;
  evalError: string | null;
  lastEvalPackageId: string | null;
  lastEvalCreatedAt: string | null;
  versionSaveStatus: VersionSaveStatus;
  versionSaveError: string | null;
  lastVersionSavedPackageId: string | null;
  lastVersionSavedAt: string | null;
  loadBootstrap: () => Promise<void>;
  selectPackage: (packageId: string) => void;
  toggleCreateComposer: (nextOpen?: boolean) => void;
  setCreatePrompt: (value: string) => void;
  setCreateContext: (value: string) => void;
  createPackage: () => Promise<CreatePackageFromNlResponse | null>;
  runEval: (packageId: string) => Promise<boolean>;
  saveVersion: (packageId: string, note?: string | null) => Promise<boolean>;
}

async function refreshBootstrap(
  preferredPackageId?: string | null,
): Promise<Pick<WorkspaceStore, "bootstrap" | "selectedPackageId" | "status" | "errorMessage">> {
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

export const useWorkspaceStore = create<WorkspaceStore>((set, get) => ({
  status: "idle",
  bootstrap: null,
  selectedPackageId: null,
  errorMessage: null,
  createComposerOpen: false,
  createPrompt: "",
  createContext: "",
  createStatus: "idle",
  createError: null,
  lastCreateResult: null,
  evalStatus: "idle",
  evalError: null,
  lastEvalPackageId: null,
  lastEvalCreatedAt: null,
  versionSaveStatus: "idle",
  versionSaveError: null,
  lastVersionSavedPackageId: null,
  lastVersionSavedAt: null,
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
  selectPackage: (packageId) => {
    set({ selectedPackageId: packageId, evalError: null });
  },
  toggleCreateComposer: (nextOpen) => {
    set((state) => {
      const open = nextOpen ?? !state.createComposerOpen;
      return {
        createComposerOpen: open,
        createError: null,
        createStatus: open ? state.createStatus : "idle",
      };
    });
  },
  setCreatePrompt: (value) => {
    set({ createPrompt: value });
  },
  setCreateContext: (value) => {
    set({ createContext: value });
  },
  createPackage: async () => {
    const bootstrap = get().bootstrap;
    const prompt = get().createPrompt.trim();
    const context = get().createContext.trim();

    if (!bootstrap) {
      set({
        createStatus: "error",
        createError: "Workspace is still loading. Try again in a moment.",
      });
      return null;
    }

    if (!prompt) {
      set({
        createStatus: "error",
        createError: "Describe the skill you want to create before generating a draft.",
      });
      return null;
    }

    set({
      createStatus: "submitting",
      createError: null,
      lastCreateResult: null,
    });

    try {
      const result = await createPackageFromNl({
        workspaceId: bootstrap.workspace.id,
        prompt,
        context: context || null,
      });
      const refreshedState = await refreshBootstrap(result.packageId);
      set({
        ...refreshedState,
        createComposerOpen: false,
        createPrompt: "",
        createContext: "",
        createStatus: "success",
        createError: null,
        lastCreateResult: result,
      });
      return result;
    } catch (error) {
      set({
        createStatus: "error",
        createError: error instanceof Error ? error.message : "Package creation failed.",
      });
      return null;
    }
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
}));
