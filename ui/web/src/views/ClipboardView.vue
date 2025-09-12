<template>
  <div>
    <div class="page-header d-print-none">
      <div class="container-xl">
        <div class="row g-2 align-items-center">
          <div class="col">
            <h2 class="page-title">Clipboard</h2>
            <div class="page-subtitle">共有クリップボード - Claude とユーザー間でテキストやファイルを共有</div>
          </div>
          <div class="col-auto ms-auto d-print-none">
            <button 
              @click="clearAll"
              class="btn btn-ghost-secondary me-2"
              :disabled="clipboardStore.historyCount === 0"
            >
              <IconTrash /> Clear All
            </button>
            <button 
              @click="refresh"
              class="btn btn-primary"
            >
              <IconRefresh /> Refresh
            </button>
          </div>
        </div>
      </div>
    </div>

    <div class="page-body">
      <div class="container-xl">
        <div class="row">
          <!-- Input Section -->
          <div class="col-md-6">
            <div class="card">
              <div class="card-header">
                <h3 class="card-title">
                  <IconEdit class="me-2" />
                  新しいコンテンツを追加
                </h3>
              </div>
              <div class="card-body">
                <!-- Text Input -->
                <div class="mb-3">
                  <label class="form-label">テキスト内容</label>
                  <textarea 
                    v-model="inputText"
                    class="form-control" 
                    rows="6"
                    placeholder="テキストを入力またはペーストしてください..."
                  ></textarea>
                </div>

                <!-- Tags Input -->
                <div class="mb-3">
                  <label class="form-label">タグ（オプション）</label>
                  <input 
                    v-model="inputTags"
                    type="text" 
                    class="form-control"
                    placeholder="タグをカンマ区切りで入力（例: urgent, code, memo）"
                  >
                </div>

                <!-- Action Buttons -->
                <div class="btn-list">
                  <button 
                    @click="saveText"
                    class="btn btn-primary"
                    :disabled="!inputText.trim()"
                  >
                    <IconDeviceFloppy /> テキストを保存
                  </button>
                  <button 
                    @click="clearInput"
                    class="btn btn-ghost-secondary"
                  >
                    <IconX /> クリア
                  </button>
                </div>
              </div>
            </div>
          </div>

          <!-- Content Display Section -->
          <div class="col-md-6">
            <div class="card">
              <div class="card-header">
                <h3 class="card-title">
                  <IconList class="me-2" />
                  保存されたコンテンツ
                </h3>
                <div class="card-actions">
                  <small class="text-muted">
                    {{ clipboardStore.historyCount }} アイテム
                  </small>
                </div>
              </div>
              <div class="card-body">
                <!-- Loading State -->
                <div v-if="clipboardStore.loading" class="text-center py-4">
                  <div class="spinner-border" role="status"></div>
                  <div class="mt-2 text-muted">読み込み中...</div>
                </div>

                <!-- Error State -->
                <div v-else-if="clipboardStore.error" class="alert alert-danger">
                  {{ clipboardStore.error }}
                </div>

                <!-- Empty State -->
                <div 
                  v-else-if="clipboardStore.historyCount === 0"
                  class="text-center py-4 text-muted"
                >
                  <IconClipboardOff class="fs-1 mb-3" />
                  <div>まだコンテンツが保存されていません</div>
                  <div class="small">左側のフォームから追加してください</div>
                </div>

                <!-- Clipboard Items -->
                <div v-else>
                  <div 
                    v-for="(item, index) in clipboardStore.history" 
                    :key="item.id"
                    class="card mb-3 clipboard-item"
                    :class="{ 'clipboard-item-latest': index === 0 }"
                  >
                    <div class="card-body">
                      <!-- Header -->
                      <div class="d-flex justify-content-between align-items-start mb-2">
                        <div class="d-flex align-items-center">
                          <IconFileText v-if="!item.filename" class="text-muted me-2" />
                          <IconFile v-else class="text-muted me-2" />
                          <div>
                            <span v-if="item.filename" class="fw-bold">{{ item.filename }}</span>
                            <span v-else class="text-muted">テキスト</span>
                            <div class="small text-muted">
                              {{ new Date(item.created_at).toLocaleString() }}
                            </div>
                          </div>
                        </div>
                        <div class="btn-list">
                          <button 
                            @click="copyToClipboard(item.content)"
                            class="btn btn-sm btn-ghost-secondary"
                            title="クリップボードにコピー"
                          >
                            <IconCopy />
                          </button>
                          <button 
                            @click="deleteItem(item.id)"
                            class="btn btn-sm btn-ghost-secondary text-red"
                            title="削除"
                          >
                            <IconTrash />
                          </button>
                        </div>
                      </div>

                      <!-- Content Preview -->
                      <pre>{{ truncateContent(item.content) }}</pre>

                      <!-- Tags -->
                      <div v-if="item.tags && item.tags.length > 0" class="mt-2">
                        <span 
                          v-for="tag in item.tags" 
                          :key="tag"
                          class="badge bg-secondary me-1"
                        >
                          {{ tag }}
                        </span>
                      </div>
                    </div>
                  </div>
                </div>
              </div>
            </div>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted } from 'vue';
import { 
  IconRefresh, 
  IconTrash, 
  IconEdit, 
  IconList,
  IconClipboardOff,
  IconFileText,
  IconFile,
  IconCopy,
  IconDeviceFloppy,
  IconX
} from '@tabler/icons-vue';
import { useClipboardStore } from '@/stores/clipboard';

const clipboardStore = useClipboardStore();

const inputText = ref('');
const inputTags = ref('');

onMounted(async () => {
  await clipboardStore.loadHistory();
});

async function refresh() {
  await clipboardStore.loadHistory();
}

async function saveText() {
  if (!inputText.value.trim()) return;
  
  const tags = inputTags.value
    .split(',')
    .map(t => t.trim())
    .filter(t => t);
  
  try {
    await clipboardStore.setTextContent(inputText.value, tags);
    clearInput();
  } catch (error) {
    console.error('Failed to save text:', error);
  }
}

function clearInput() {
  inputText.value = '';
  inputTags.value = '';
}

async function clearAll() {
  if (!confirm('すべてのクリップボードアイテムを削除しますか？')) {
    return;
  }
  
  try {
    await clipboardStore.clearAll();
  } catch (error) {
    console.error('Failed to clear clipboard:', error);
  }
}

async function copyToClipboard(content: string) {
  const success = await clipboardStore.copyToSystemClipboard(content);
  if (success) {
    // TODO: Show success toast
    console.log('Copied to clipboard');
  }
}

async function deleteItem(id: string) {
  try {
    await clipboardStore.deleteItem(id);
  } catch (error) {
    console.error('Failed to delete item:', error);
  }
}

function truncateContent(content: string, maxLength = 300): string {
  if (content.length <= maxLength) return content;
  return content.substring(0, maxLength) + '...';
}
</script>