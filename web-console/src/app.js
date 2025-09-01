console.log('app.js loaded');

document.addEventListener('alpine:init', () => {
  console.log('Alpine.js is initializing...');
  // シンプルなデータ構造で開始
  Alpine.store('state', {
    mode: 'dark',
    toggleMode() {
      this.mode = this.mode === 'dark' ? 'light' : 'dark';
    }
  });
});

console.log('Event listener registered');