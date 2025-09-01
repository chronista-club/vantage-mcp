console.log('app.ts loaded');

// Alpine.jsの型定義
interface AlpineStore {
  state: {
    mode: 'dark' | 'light';
    toggleMode(): void;
  };
}

declare global {
  interface Window {
    Alpine: {
      store: <T>(name: string, store: T) => void;
      data: (name: string, component: () => any) => void;
    };
  }
}

document.addEventListener('alpine:init', () => {
  console.log('Alpine.js is initializing...');
  
  // グローバルステートストア
  window.Alpine.store('state', {
    mode: 'dark' as 'dark' | 'light',
    toggleMode() {
      this.mode = this.mode === 'dark' ? 'light' : 'dark';
    }
  });
});

console.log('Event listener registered');