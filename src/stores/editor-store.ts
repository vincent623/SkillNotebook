import { create } from "zustand";
import type { FileEntry } from "../types/models";
import { getPackageFileTree, readPackageFile, writePackageFile } from "../services/tauri-api";

type EditorMode = "preview" | "edit";

interface EditorStore {
  fileTree: FileEntry[];
  currentFilePath: string | null;
  fileContent: string;
  originalContent: string;
  mode: EditorMode;
  isTreeLoading: boolean;
  isFileLoading: boolean;
  isSaving: boolean;
  isDirty: boolean;
  viewingVersionId: string | null;
  treeError: string | null;
  fileError: string | null;
  saveError: string | null;
  saveNotice: string | null;

  loadFileTree: (packageId: string) => Promise<void>;
  openFile: (packageId: string, path: string) => Promise<void>;
  setMode: (mode: EditorMode) => void;
  setFileContent: (content: string) => void;
  saveFile: (packageId: string) => Promise<boolean>;
  refreshOpenFile: (packageId: string) => Promise<void>;
  setViewingVersionId: (versionId: string | null) => void;
  reset: () => void;
}

export const useEditorStore = create<EditorStore>((set, get) => ({
  fileTree: [],
  currentFilePath: null,
  fileContent: "",
  originalContent: "",
  mode: "preview",
  isTreeLoading: false,
  isFileLoading: false,
  isSaving: false,
  isDirty: false,
  viewingVersionId: null,
  treeError: null,
  fileError: null,
  saveError: null,
  saveNotice: null,

  loadFileTree: async (packageId) => {
    set({ isTreeLoading: true, treeError: null });
    try {
      const tree = await getPackageFileTree(packageId);
      set({ fileTree: tree, isTreeLoading: false, treeError: null });
    } catch (error) {
      set({
        fileTree: [],
        isTreeLoading: false,
        treeError: error instanceof Error ? error.message : "文件树加载失败。",
      });
    }
  },

  openFile: async (packageId, path) => {
    set({
      isFileLoading: true,
      currentFilePath: path,
      mode: "preview",
      fileError: null,
      saveError: null,
      saveNotice: null,
    });
    try {
      const file = await readPackageFile(packageId, path);
      set({
        fileContent: file.content,
        originalContent: file.content,
        isDirty: false,
        isFileLoading: false,
        fileError: null,
      });
    } catch (error) {
      set({
        currentFilePath: path,
        fileContent: "",
        originalContent: "",
        isDirty: false,
        isFileLoading: false,
        fileError: error instanceof Error ? error.message : "文件读取失败。",
      });
    }
  },

  setMode: (mode) => set({ mode }),

  setFileContent: (content) => {
    set((state) => ({
      fileContent: content,
      isDirty: content !== state.originalContent,
      saveError: null,
      saveNotice: null,
    }));
  },

  saveFile: async (packageId) => {
    const { currentFilePath, fileContent } = get();
    if (!currentFilePath) return false;
    set({ isSaving: true, saveError: null, saveNotice: null });
    try {
      await writePackageFile(packageId, currentFilePath, fileContent);
      set({
        isSaving: false,
        isDirty: false,
        originalContent: fileContent,
        saveNotice: `已保存 ${currentFilePath}`,
      });
      return true;
    } catch (error) {
      set({
        isSaving: false,
        saveError: error instanceof Error ? error.message : "保存失败。",
      });
      return false;
    }
  },

  refreshOpenFile: async (packageId) => {
    const { currentFilePath, isDirty, isFileLoading, isSaving } = get();
    if (!currentFilePath || isDirty || isFileLoading || isSaving) return;

    try {
      const file = await readPackageFile(packageId, currentFilePath);
      set((state) => {
        if (state.currentFilePath !== currentFilePath || state.isDirty) {
          return {};
        }
        return {
          fileContent: file.content,
          originalContent: file.content,
          fileError: null,
        };
      });
    } catch (error) {
      set({
        fileError: error instanceof Error ? error.message : "文件刷新失败。",
      });
    }
  },

  setViewingVersionId: (versionId) => set({ viewingVersionId: versionId }),

  reset: () =>
    set({
      fileTree: [],
      currentFilePath: null,
      fileContent: "",
      originalContent: "",
      mode: "preview",
      isTreeLoading: false,
      isFileLoading: false,
      isSaving: false,
      isDirty: false,
      viewingVersionId: null,
      treeError: null,
      fileError: null,
      saveError: null,
      saveNotice: null,
    }),
}));
