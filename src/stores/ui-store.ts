import { create } from "zustand";
import type { AppScreen } from "../types/models";

interface UiStore {
  currentScreen: AppScreen;
  setCurrentScreen: (screen: AppScreen) => void;
}

export const useUiStore = create<UiStore>((set) => ({
  currentScreen: "home",
  setCurrentScreen: (screen) => {
    set({ currentScreen: screen });
  },
}));
