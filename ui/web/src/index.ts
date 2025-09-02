/// <reference types="alpinejs" />

import { createMainStore } from './stores/MainStore';

console.log('Ichimi UI loaded');

// このファイルをモジュールとして扱うためのexport
export { };

// Alpine型を拡張
declare global {
  interface Window {
    Alpine: {
      store<T>(name: string, store?: T): T | void;
      data(name: string, component: () => any): void;
      plugin(plugin: any): void;
    };
  }
}

// Alpine.js初期化時の処理
document.addEventListener('alpine:init', () => {
  console.log('Alpine.js is initializing...');

  // Create unified state store
  const mainStore = createMainStore();

  // Register store with Alpine
  window.Alpine.store('state', mainStore);

  // Initialize store
  // mainStore.init();
});

console.log('Event listener registered');