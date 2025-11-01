<template>
  <div>
    <div class="page-header d-print-none">
      <div class="container-xl">
        <div class="row g-2 align-items-center">
          <div class="col">
            <h2 class="page-title">{{ $t('templates.title') }}</h2>
          </div>
          <div class="col-auto">
            <button
              @click="showCreateModal = true"
              class="btn btn-primary"
            >
              <i class="ti ti-plus"></i> {{ $t('templates.createTemplate') }}
            </button>
          </div>
        </div>
      </div>
    </div>

    <div class="page-body">
      <div class="container-xl">
        <!-- Loading State -->
        <div v-if="templateStore.loading" class="text-center">
          <div class="spinner-border" role="status"></div>
        </div>

        <!-- Error State -->
        <div v-else-if="templateStore.error" class="alert alert-danger">
          {{ templateStore.error }}
        </div>

        <!-- Empty State -->
        <div
          v-else-if="templateStore.templateCount === 0"
          class="empty"
        >
          <p class="empty-title">{{ $t('templates.noTemplates') }}</p>
          <p class="empty-subtitle text-muted">{{ $t('templates.noTemplatesDescription') }}</p>
          <div class="empty-action">
            <button @click="showCreateModal = true" class="btn btn-primary">
              <i class="ti ti-plus"></i> {{ $t('templates.createFirstTemplate') }}
            </button>
          </div>
        </div>

        <!-- Template Grid -->
        <div v-else class="row row-cards">
          <div
            v-for="template in templateStore.templates"
            :key="template.template_id"
            class="col-md-6 col-lg-4"
          >
            <div class="card">
              <div class="card-body">
                <h3 class="card-title">{{ template.name }}</h3>
                <p class="text-muted">{{ template.description }}</p>
                <div v-if="template.tags && template.tags.length > 0" class="mb-3">
                  <span
                    v-for="tag in template.tags"
                    :key="tag"
                    class="badge bg-secondary me-1"
                  >
                    {{ tag }}
                  </span>
                </div>
                <div class="d-flex gap-2">
                  <button
                    @click="useTemplate(template)"
                    class="btn btn-primary flex-fill"
                  >
                    {{ $t('templates.useTemplate') }}
                  </button>
                  <button
                    @click="deleteTemplate(template)"
                    class="btn btn-ghost-danger"
                    :title="$t('common.delete')"
                  >
                    <i class="ti ti-trash"></i>
                  </button>
                </div>
              </div>
            </div>
          </div>
        </div>
      </div>
    </div>

    <!-- Create Template Modal -->
    <CreateTemplateModal
      v-model="showCreateModal"
      @created="onTemplateCreated"
    />

    <!-- Use Template Modal -->
    <UseTemplateModal
      v-model="showUseModal"
      :template="selectedTemplate"
      @created="onProcessCreated"
    />

    <!-- Confirm Delete Modal -->
    <ConfirmModal
      v-model="showDeleteConfirm"
      :title="$t('common.delete')"
      :message="deleteConfirmMessage"
      :confirm-text="$t('common.delete')"
      danger-mode
      @confirm="confirmDelete"
    />
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted } from 'vue';
import type { ProcessTemplate, ProcessInfo } from '@/types';
import { useTemplateStore } from '@/stores/template';
import { useRouter } from 'vue-router';
import { useI18n } from 'vue-i18n';
import { useToast } from '@/composables/useToast';
import CreateTemplateModal from '@/components/CreateTemplateModal.vue';
import UseTemplateModal from '@/components/UseTemplateModal.vue';
import ConfirmModal from '@/components/ConfirmModal.vue';
import apiClient from '@/api/client';

const { t } = useI18n();
const router = useRouter();
const { showSuccess, showError } = useToast();
const templateStore = useTemplateStore();

const showCreateModal = ref(false);
const showUseModal = ref(false);
const showDeleteConfirm = ref(false);
const selectedTemplate = ref<ProcessTemplate | null>(null);
const templateToDelete = ref<ProcessTemplate | null>(null);
const deleteConfirmMessage = ref('');

onMounted(async () => {
  await templateStore.loadTemplates();
});

function useTemplate(template: ProcessTemplate) {
  selectedTemplate.value = template;
  showUseModal.value = true;
}

function deleteTemplate(template: ProcessTemplate) {
  templateToDelete.value = template;
  deleteConfirmMessage.value = t('templates.confirmDelete', { name: template.name });
  showDeleteConfirm.value = true;
}

async function confirmDelete() {
  if (!templateToDelete.value) return;

  try {
    await apiClient.deleteTemplate(templateToDelete.value.template_id);
    showSuccess(t('templates.deleteSuccess'));
    await templateStore.loadTemplates();
  } catch (error: any) {
    showError(t('templates.deleteError', { error: error.message }));
  } finally {
    templateToDelete.value = null;
  }
}

async function onTemplateCreated() {
  await templateStore.loadTemplates();
}

function onProcessCreated(process: ProcessInfo) {
  showSuccess(t('templates.processCreated', { id: process.id }));
  router.push('/processes');
}
</script>