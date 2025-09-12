import { defineStore } from 'pinia';
import { ref, computed } from 'vue';
import type { ClipboardItem, ClipboardHistoryResponse } from '@/types';
import apiClient from '@/api/client';

export const useClipboardStore = defineStore('clipboard', () => {
  // State
  const currentItem = ref<ClipboardItem | null>(null);
  const history = ref<ClipboardItem[]>([]);
  const loading = ref(false);
  const error = ref<string | null>(null);
  const totalCount = ref(0);

  // Computed
  const hasContent = computed(() => currentItem.value !== null);
  const historyCount = computed(() => history.value.length);
  
  const latestItem = computed(() => 
    history.value.length > 0 ? history.value[0] : null
  );

  // Actions
  async function loadCurrent() {
    loading.value = true;
    error.value = null;
    try {
      currentItem.value = await apiClient.getClipboard();
    } catch (e: any) {
      error.value = e.message || 'Failed to load clipboard';
      console.error('Failed to load clipboard:', e);
    } finally {
      loading.value = false;
    }
  }

  async function loadHistory(limit = 50) {
    loading.value = true;
    error.value = null;
    try {
      const response = await apiClient.getClipboardHistory(limit);
      history.value = response.items;
      totalCount.value = response.total_count;
    } catch (e: any) {
      error.value = e.message || 'Failed to load clipboard history';
      console.error('Failed to load clipboard history:', e);
    } finally {
      loading.value = false;
    }
  }

  async function setTextContent(content: string, tags: string[] = []): Promise<ClipboardItem> {
    error.value = null;
    try {
      const item = await apiClient.setClipboardText(content, tags);
      currentItem.value = item;
      // Prepend to history
      history.value = [item, ...history.value.filter(h => h.id !== item.id)];
      return item;
    } catch (e: any) {
      error.value = e.message || 'Failed to set clipboard text';
      throw e;
    }
  }

  async function setFileContent(
    content: string, 
    filename: string, 
    tags: string[] = []
  ): Promise<ClipboardItem> {
    error.value = null;
    try {
      const item = await apiClient.setClipboardFile(content, filename, tags);
      currentItem.value = item;
      // Prepend to history
      history.value = [item, ...history.value.filter(h => h.id !== item.id)];
      return item;
    } catch (e: any) {
      error.value = e.message || 'Failed to set clipboard file';
      throw e;
    }
  }

  async function deleteItem(id: string) {
    error.value = null;
    try {
      await apiClient.deleteClipboardItem(id);
      history.value = history.value.filter(item => item.id !== id);
      if (currentItem.value?.id === id) {
        currentItem.value = null;
      }
    } catch (e: any) {
      error.value = e.message || `Failed to delete clipboard item ${id}`;
      throw e;
    }
  }

  async function searchClipboard(query: string, limit = 50): Promise<ClipboardHistoryResponse> {
    error.value = null;
    try {
      const response = await apiClient.searchClipboard(query, limit);
      return response;
    } catch (e: any) {
      error.value = e.message || 'Failed to search clipboard';
      throw e;
    }
  }

  async function clearAll() {
    error.value = null;
    try {
      await apiClient.clearClipboard();
      currentItem.value = null;
      history.value = [];
      totalCount.value = 0;
    } catch (e: any) {
      error.value = e.message || 'Failed to clear clipboard';
      throw e;
    }
  }

  // Helper to copy text to browser clipboard
  async function copyToSystemClipboard(text: string): Promise<boolean> {
    try {
      if (navigator.clipboard && window.isSecureContext) {
        await navigator.clipboard.writeText(text);
        return true;
      } else {
        // Fallback for older browsers
        const textArea = document.createElement('textarea');
        textArea.value = text;
        textArea.style.position = 'fixed';
        textArea.style.left = '-999999px';
        document.body.appendChild(textArea);
        textArea.focus();
        textArea.select();
        try {
          document.execCommand('copy');
          return true;
        } finally {
          document.body.removeChild(textArea);
        }
      }
    } catch (e) {
      console.error('Failed to copy to clipboard:', e);
      return false;
    }
  }

  // Helper to read from browser clipboard
  async function readFromSystemClipboard(): Promise<string | null> {
    try {
      if (navigator.clipboard && window.isSecureContext) {
        return await navigator.clipboard.readText();
      }
      return null;
    } catch (e) {
      console.error('Failed to read from clipboard:', e);
      return null;
    }
  }

  function clearError() {
    error.value = null;
  }

  return {
    // State
    currentItem,
    history,
    loading,
    error,
    totalCount,
    
    // Computed
    hasContent,
    historyCount,
    latestItem,
    
    // Actions
    loadCurrent,
    loadHistory,
    setTextContent,
    setFileContent,
    deleteItem,
    searchClipboard,
    clearAll,
    copyToSystemClipboard,
    readFromSystemClipboard,
    clearError,
  };
});