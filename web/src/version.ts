// Valeurs injectées à la compilation par vite.config.ts (jamais lues à l'exécution depuis le réseau).
declare const __APP_VERSION__: string;
declare const __APP_ENV__: string;

export const appVersion: string = __APP_VERSION__;
export const appEnvironment: string = __APP_ENV__;
