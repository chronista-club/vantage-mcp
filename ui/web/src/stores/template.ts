import { defineStore } from 'pinia';
import { ref, computed } from 'vue';
import type { ProcessTemplate, ProcessInfo } from '@/types';
import apiClient from '@/api/client';
import { useProcessStore } from './process';

export const useTemplateStore = defineStore('template', () => {
  // State
  const templates = ref<ProcessTemplate[]>([]);
  const loading = ref(false);
  const error = ref<string | null>(null);
  const selectedTemplate = ref<ProcessTemplate | null>(null);
  const instantiating = ref(false);

  // Computed
  const templateCount = computed(() => templates.value.length);
  
  const templatesByCategory = computed(() => {
    const categories = new Map<string, ProcessTemplate[]>();
    templates.value.forEach(template => {
      const category = template.category || 'Uncategorized';
      if (!categories.has(category)) {
        categories.set(category, []);
      }
      categories.get(category)!.push(template);
    });
    return categories;
  });

  // Actions
  async function loadTemplates() {
    loading.value = true;
    error.value = null;
    try {
      templates.value = await apiClient.getTemplates();
    } catch (e: any) {
      error.value = e.message || 'Failed to load templates';
      console.error('Failed to load templates:', e);
    } finally {
      loading.value = false;
    }
  }

  async function instantiateTemplate(
    templateId: string,
    processId: string,
    values: Record<string, any>
  ): Promise<ProcessInfo> {
    instantiating.value = true;
    error.value = null;
    try {
      const newProcess = await apiClient.instantiateTemplate(templateId, processId, values);
      
      // Add to process store
      const processStore = useProcessStore();
      await processStore.loadProcesses();
      
      return newProcess;
    } catch (e: any) {
      error.value = e.message || 'Failed to instantiate template';
      throw e;
    } finally {
      instantiating.value = false;
    }
  }

  function selectTemplate(template: ProcessTemplate | null) {
    selectedTemplate.value = template;
  }

  function clearError() {
    error.value = null;
  }

  // Helper to validate template variables
  function validateTemplateValues(
    template: ProcessTemplate,
    values: Record<string, any>
  ): { valid: boolean; errors: string[] } {
    const errors: string[] = [];
    
    if (!template.variables) {
      return { valid: true, errors: [] };
    }

    for (const variable of template.variables) {
      const value = values[variable.name];
      
      // Check required
      if (variable.required && (value === undefined || value === null || value === '')) {
        errors.push(`${variable.name} is required`);
        continue;
      }
      
      // Skip validation if not required and empty
      if (!variable.required && (value === undefined || value === null || value === '')) {
        continue;
      }
      
      // Type validation
      if (variable.var_type) {
        switch (variable.var_type) {
          case 'number':
            if (typeof value !== 'number' && isNaN(Number(value))) {
              errors.push(`${variable.name} must be a number`);
            } else {
              const numValue = Number(value);
              if (variable.validation?.min !== undefined && numValue < variable.validation.min) {
                errors.push(`${variable.name} must be at least ${variable.validation.min}`);
              }
              if (variable.validation?.max !== undefined && numValue > variable.validation.max) {
                errors.push(`${variable.name} must be at most ${variable.validation.max}`);
              }
            }
            break;
          case 'boolean':
            if (typeof value !== 'boolean') {
              errors.push(`${variable.name} must be a boolean`);
            }
            break;
          case 'path':
            if (typeof value !== 'string' || value.trim() === '') {
              errors.push(`${variable.name} must be a valid path`);
            }
            break;
          // 'string' type doesn't need special validation
        }
      }
    }
    
    return { valid: errors.length === 0, errors };
  }

  // Get default values for a template
  function getTemplateDefaults(template: ProcessTemplate): Record<string, any> {
    const defaults: Record<string, any> = {};
    
    if (template.variables) {
      for (const variable of template.variables) {
        if (variable.default_value !== undefined) {
          defaults[variable.name] = variable.default_value;
        } else {
          // Set sensible defaults based on type
          switch (variable.var_type) {
            case 'number':
              defaults[variable.name] = 0;
              break;
            case 'boolean':
              defaults[variable.name] = false;
              break;
            default:
              defaults[variable.name] = '';
          }
        }
      }
    }
    
    return defaults;
  }

  return {
    // State
    templates,
    loading,
    error,
    selectedTemplate,
    instantiating,
    
    // Computed
    templateCount,
    templatesByCategory,
    
    // Actions
    loadTemplates,
    instantiateTemplate,
    selectTemplate,
    clearError,
    validateTemplateValues,
    getTemplateDefaults,
  };
});