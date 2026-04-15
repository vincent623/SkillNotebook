import { create } from "zustand";
import type { WorkspaceTab } from "../types/models";

interface PackageStore {
  activeTab: WorkspaceTab;
  setActiveTab: (tab: WorkspaceTab) => void;
}

export const usePackageStore = create<PackageStore>((set) => ({
  activeTab: "overview",
  setActiveTab: (tab) => {
    set({ activeTab: tab });
  },
}));
