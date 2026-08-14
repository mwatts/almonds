import { defineStore } from "pinia";

export const useAuthStore = defineStore("auth_store", {
  state: () => ({
    accessToken: "",
    refreshToken: "",
    tokenExpiry: 0,
    /** Transient token used mid-flow for account verification / password reset. */
    pendingToken: "",
  }),

  getters: {
    isAuthenticated: (state) => !!state.accessToken,
    hasPendingToken: (state) => !!state.pendingToken,
  },

  actions: {
    setSession(
      accessToken: string,
      refreshToken: string,
      tokenExpiry = 0,
    ) {
      this.accessToken = accessToken;
      this.refreshToken = refreshToken;
      this.tokenExpiry = tokenExpiry;
    },

    setPendingToken(token: string) {
      this.pendingToken = token;
    },

    clearPendingToken() {
      this.pendingToken = "";
    },

    clearSession() {
      this.accessToken = "";
      this.refreshToken = "";
      this.tokenExpiry = 0;
      this.pendingToken = "";
    },
  },

  persist: true,
});
