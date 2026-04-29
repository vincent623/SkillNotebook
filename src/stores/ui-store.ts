import { create } from "zustand";
import type { AppScreen } from "../types/models";

interface UiStore {
  currentScreen: AppScreen;
  isCommandPaletteOpen: boolean;
  setCurrentScreen: (screen: AppScreen) => void;
  openCommandPalette: () => void;
  closeCommandPalette: () => void;
  toggleCommandPalette: () => void;
}

export const useUiStore = create<UiStore>((set) => ({
  currentScreen: "explorer",
  isCommandPaletteOpen: false,
  setCurrentScreen: (screen) => {
    set({ currentScreen: screen });
  },
  openCommandPalette: () => {
    set({ isCommandPaletteOpen: true });
  },
  closeCommandPalette: () => {
    set({ isCommandPaletteOpen: false });
  },
  toggleCommandPalette: () => {
    set((state) => ({ isCommandPaletteOpen: !state.isCommandPaletteOpen }));
  },
}));
