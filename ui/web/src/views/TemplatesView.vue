<template>
  <div>
    <div class="page-header d-print-none">
      <div class="container-xl">
        <div class="row g-2 align-items-center">
          <div class="col">
            <h2 class="page-title">Templates</h2>
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
          <p class="empty-title">No templates available</p>
          <p class="empty-subtitle text-muted">Templates will appear here when configured</p>
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
                <button 
                  @click="useTemplate(template)"
                  class="btn btn-primary w-100"
                >
                  Use Template
                </button>
              </div>
            </div>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { onMounted } from 'vue';
import type { ProcessTemplate } from '@/types';
import { useTemplateStore } from '@/stores/template';

const templateStore = useTemplateStore();

onMounted(async () => {
  await templateStore.loadTemplates();
});

function useTemplate(template: ProcessTemplate) {
  // TODO: Implement template instantiation modal
  templateStore.selectTemplate(template);
  console.log('Use template:', template);
}
</script>