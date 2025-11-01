<template>
  <div
    v-if="modelValue && template"
    class="modal modal-blur fade show d-block"
    tabindex="-1"
    @click.self="close"
  >
    <div class="modal-dialog modal-lg modal-dialog-centered">
      <div class="modal-content">
        <div class="modal-header">
          <h5 class="modal-title">{{ $t('templates.useTemplate') }}: {{ template.name }}</h5>
          <button
            type="button"
            class="btn-close"
            @click="close"
          ></button>
        </div>

        <div class="modal-body">
          <form @submit.prevent="handleSubmit">
            <!-- Process ID -->
            <div class="mb-3">
              <label class="form-label required">{{ $t('templates.processId') }}</label>
              <input
                v-model="processId"
                type="text"
                class="form-control"
                :placeholder="$t('templates.processIdPlaceholder')"
                required
              />
              <small class="form-hint">{{ $t('templates.processIdHint') }}</small>
            </div>

            <!-- Template Variables -->
            <div v-if="template.variables && template.variables.length > 0">
              <h4 class="mb-3">{{ $t('templates.configureVariables') }}</h4>

              <div
                v-for="variable in template.variables"
                :key="variable.name"
                class="mb-3"
              >
                <label class="form-label" :class="{ required: variable.required }">
                  {{ variable.name }}
                  <small v-if="variable.description" class="text-muted">
                    ({{ variable.description }})
                  </small>
                </label>

                <!-- String input -->
                <input
                  v-if="variable.var_type === 'string' || variable.var_type === 'path'"
                  v-model="variableValues[variable.name]"
                  type="text"
                  class="form-control"
                  :placeholder="variable.default_value || ''"
                  :required="variable.required"
                />

                <!-- Number input -->
                <input
                  v-else-if="variable.var_type === 'number'"
                  v-model.number="variableValues[variable.name]"
                  type="number"
                  class="form-control"
                  :placeholder="variable.default_value?.toString() || ''"
                  :required="variable.required"
                  :min="variable.validation?.min"
                  :max="variable.validation?.max"
                />

                <!-- Boolean input -->
                <div v-else-if="variable.var_type === 'boolean'" class="form-check">
                  <input
                    v-model="variableValues[variable.name]"
                    type="checkbox"
                    class="form-check-input"
                    :id="`var-${variable.name}`"
                  />
                  <label class="form-check-label" :for="`var-${variable.name}`">
                    {{ $t('common.enabled') }}
                  </label>
                </div>

                <small v-if="variable.default_value !== undefined" class="form-hint">
                  {{ $t('templates.defaultValue') }}: {{ variable.default_value }}
                </small>
              </div>
            </div>

            <!-- Preview -->
            <div class="card bg-gray-50">
              <div class="card-body">
                <h4 class="card-title">{{ $t('templates.preview') }}</h4>
                <div class="mb-2">
                  <strong>{{ $t('templates.command') }}:</strong>
                  <code class="ms-2">{{ expandedCommand }}</code>
                </div>
                <div v-if="expandedArgs.length > 0" class="mb-2">
                  <strong>{{ $t('templates.arguments') }}:</strong>
                  <code class="ms-2">{{ expandedArgs.join(' ') }}</code>
                </div>
                <div v-if="expandedCwd" class="mb-2">
                  <strong>{{ $t('templates.workingDirectory') }}:</strong>
                  <code class="ms-2">{{ expandedCwd }}</code>
                </div>
                <div v-if="expandedEnv && Object.keys(expandedEnv).length > 0">
                  <strong>{{ $t('templates.environment') }}:</strong>
                  <pre class="mt-1 p-2 bg-white rounded">{{ JSON.stringify(expandedEnv, null, 2) }}</pre>
                </div>
              </div>
            </div>
          </form>
        </div>

        <div class="modal-footer">
          <button
            type="button"
            class="btn btn-secondary"
            @click="close"
          >
            {{ $t('common.cancel') }}
          </button>
          <button
            type="button"
            class="btn btn-primary"
            @click="handleSubmit"
            :disabled="!isFormValid || creating"
          >
            <span v-if="creating" class="spinner-border spinner-border-sm me-2"></span>
            {{ $t('templates.createProcess') }}
          </button>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, watch } from 'vue';
import type { ProcessTemplate, ProcessInfo } from '@/types';
import { useTemplateStore } from '@/stores/template';
import { useI18n } from 'vue-i18n';
import { useToast } from '@/composables/useToast';

const props = defineProps<{
  modelValue: boolean;
  template: ProcessTemplate | null;
}>();

const emit = defineEmits<{
  'update:modelValue': [value: boolean];
  'created': [process: ProcessInfo];
}>();

const { t } = useI18n();
const { showSuccess, showError } = useToast();
const templateStore = useTemplateStore();

const processId = ref('');
const variableValues = ref<Record<string, any>>({});
const creating = ref(false);

// Initialize variable values with defaults when template changes
watch(() => props.template, (newTemplate) => {
  if (newTemplate) {
    processId.value = '';
    variableValues.value = templateStore.getTemplateDefaults(newTemplate);
  }
});

const isFormValid = computed(() => {
  if (!processId.value) return false;
  if (!props.template) return false;

  const validation = templateStore.validateTemplateValues(props.template, variableValues.value);
  return validation.valid;
});

// Expand variables in strings
function expandVariables(text: string): string {
  let expanded = text;
  for (const [key, value] of Object.entries(variableValues.value)) {
    expanded = expanded.replace(new RegExp(`\\$\\{${key}\\}`, 'g'), String(value));
  }
  return expanded;
}

const expandedCommand = computed(() => {
  if (!props.template) return '';
  return expandVariables(props.template.command);
});

const expandedArgs = computed(() => {
  if (!props.template?.args) return [];
  return props.template.args.map((arg: string) => expandVariables(arg));
});

const expandedCwd = computed(() => {
  if (!props.template?.default_cwd) return '';
  return expandVariables(props.template.default_cwd);
});

const expandedEnv = computed(() => {
  if (!props.template?.env) return {};
  const env: Record<string, string> = {};
  for (const [key, value] of Object.entries(props.template.env)) {
    env[key] = expandVariables(String(value));
  }
  return env;
});

async function handleSubmit() {
  if (!isFormValid.value || !props.template) return;

  creating.value = true;
  try {
    const process = await templateStore.instantiateTemplate(
      props.template.template_id,
      processId.value,
      variableValues.value
    );

    showSuccess(t('templates.processCreated', { id: process.id }));
    emit('created', process);
    close();
  } catch (error: any) {
    showError(t('templates.createProcessError', { error: error.message }));
  } finally {
    creating.value = false;
  }
}

function close() {
  emit('update:modelValue', false);
  processId.value = '';
  variableValues.value = {};
}
</script>

<style scoped>
.modal.show {
  background-color: rgba(0, 0, 0, 0.5);
}
</style>