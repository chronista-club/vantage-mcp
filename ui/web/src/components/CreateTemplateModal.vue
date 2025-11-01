<template>
  <div
    v-if="modelValue"
    class="modal modal-blur fade show d-block"
    tabindex="-1"
    @click.self="close"
  >
    <div class="modal-dialog modal-lg modal-dialog-centered">
      <div class="modal-content">
        <div class="modal-header">
          <h5 class="modal-title">{{ $t('templates.createTemplate') }}</h5>
          <button
            type="button"
            class="btn-close"
            @click="close"
          ></button>
        </div>

        <div class="modal-body">
          <form @submit.prevent="handleSubmit">
            <!-- Template ID -->
            <div class="mb-3">
              <label class="form-label required">{{ $t('templates.templateId') }}</label>
              <input
                v-model="form.template_id"
                type="text"
                class="form-control"
                :placeholder="$t('templates.templateIdPlaceholder')"
                required
              />
              <small class="form-hint">{{ $t('templates.templateIdHint') }}</small>
            </div>

            <!-- Name -->
            <div class="mb-3">
              <label class="form-label required">{{ $t('templates.name') }}</label>
              <input
                v-model="form.name"
                type="text"
                class="form-control"
                :placeholder="$t('templates.namePlaceholder')"
                required
              />
            </div>

            <!-- Description -->
            <div class="mb-3">
              <label class="form-label">{{ $t('templates.description') }}</label>
              <textarea
                v-model="form.description"
                class="form-control"
                rows="2"
                :placeholder="$t('templates.descriptionPlaceholder')"
              ></textarea>
            </div>

            <!-- Category -->
            <div class="mb-3">
              <label class="form-label">{{ $t('templates.category') }}</label>
              <input
                v-model="form.category"
                type="text"
                class="form-control"
                :placeholder="$t('templates.categoryPlaceholder')"
              />
            </div>

            <!-- Command -->
            <div class="mb-3">
              <label class="form-label required">{{ $t('templates.command') }}</label>
              <input
                v-model="form.command"
                type="text"
                class="form-control"
                :placeholder="$t('templates.commandPlaceholder')"
                required
              />
            </div>

            <!-- Arguments -->
            <div class="mb-3">
              <label class="form-label">{{ $t('templates.arguments') }}</label>
              <input
                v-model="argsString"
                type="text"
                class="form-control"
                :placeholder="$t('templates.argumentsPlaceholder')"
              />
              <small class="form-hint">{{ $t('templates.argumentsHint') }}</small>
            </div>

            <!-- Working Directory -->
            <div class="mb-3">
              <label class="form-label">{{ $t('templates.workingDirectory') }}</label>
              <input
                v-model="form.default_cwd"
                type="text"
                class="form-control"
                :placeholder="$t('templates.workingDirectoryPlaceholder')"
              />
            </div>

            <!-- Tags -->
            <div class="mb-3">
              <label class="form-label">{{ $t('templates.tags') }}</label>
              <input
                v-model="tagsString"
                type="text"
                class="form-control"
                :placeholder="$t('templates.tagsPlaceholder')"
              />
              <small class="form-hint">{{ $t('templates.tagsHint') }}</small>
            </div>

            <!-- Variables Section -->
            <div class="mb-3">
              <label class="form-label">{{ $t('templates.variables') }}</label>
              <div class="card">
                <div class="card-body">
                  <div v-if="form.variables.length === 0" class="text-muted text-center py-3">
                    {{ $t('templates.noVariables') }}
                  </div>
                  <div v-else>
                    <div
                      v-for="(variable, index) in form.variables"
                      :key="index"
                      class="mb-3 p-3 bg-gray-50 rounded"
                    >
                      <div class="row g-2">
                        <div class="col-md-6">
                          <input
                            v-model="variable.name"
                            type="text"
                            class="form-control form-control-sm"
                            :placeholder="$t('templates.variableName')"
                          />
                        </div>
                        <div class="col-md-4">
                          <select
                            v-model="variable.var_type"
                            class="form-select form-select-sm"
                          >
                            <option value="string">String</option>
                            <option value="number">Number</option>
                            <option value="boolean">Boolean</option>
                            <option value="path">Path</option>
                          </select>
                        </div>
                        <div class="col-md-2">
                          <button
                            type="button"
                            class="btn btn-sm btn-ghost-danger w-100"
                            @click="removeVariable(index)"
                          >
                            <i class="ti ti-trash"></i>
                          </button>
                        </div>
                      </div>
                      <div class="row g-2 mt-1">
                        <div class="col-md-6">
                          <input
                            v-model="variable.description"
                            type="text"
                            class="form-control form-control-sm"
                            :placeholder="$t('templates.variableDescription')"
                          />
                        </div>
                        <div class="col-md-4">
                          <input
                            v-model="variable.default_value"
                            :type="variable.var_type === 'number' ? 'number' : 'text'"
                            class="form-control form-control-sm"
                            :placeholder="$t('templates.defaultValue')"
                          />
                        </div>
                        <div class="col-md-2">
                          <label class="form-check">
                            <input
                              v-model="variable.required"
                              type="checkbox"
                              class="form-check-input"
                            />
                            <span class="form-check-label">{{ $t('templates.required') }}</span>
                          </label>
                        </div>
                      </div>
                    </div>
                  </div>
                  <button
                    type="button"
                    class="btn btn-sm btn-primary"
                    @click="addVariable"
                  >
                    <i class="ti ti-plus"></i> {{ $t('templates.addVariable') }}
                  </button>
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
            :disabled="!isFormValid"
          >
            {{ $t('templates.create') }}
          </button>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, watch } from 'vue';
import type { ProcessTemplate, TemplateVariable } from '@/types';
import apiClient from '@/api/client';
import { useToast } from '@/composables/useToast';
import { useI18n } from 'vue-i18n';

defineProps<{
  modelValue: boolean;
}>();

const emit = defineEmits<{
  'update:modelValue': [value: boolean];
  'created': [template: ProcessTemplate];
}>();

const { t } = useI18n();
const { showSuccess, showError } = useToast();

const form = ref({
  template_id: '',
  name: '',
  description: '',
  category: '',
  command: '',
  args: [] as string[],
  env: {} as Record<string, string>,
  default_cwd: '',
  tags: [] as string[],
  variables: [] as TemplateVariable[],
  default_auto_start: false,
});

const argsString = ref('');
const tagsString = ref('');

// Parse args string to array
watch(argsString, (value) => {
  form.value.args = value.split(' ').filter(arg => arg.trim());
});

// Parse tags string to array
watch(tagsString, (value) => {
  form.value.tags = value.split(',').map(tag => tag.trim()).filter(tag => tag);
});

const isFormValid = computed(() => {
  return form.value.template_id && form.value.name && form.value.command;
});

function addVariable() {
  form.value.variables.push({
    name: '',
    var_type: 'string',
    description: '',
    default_value: '',
    required: false,
  });
}

function removeVariable(index: number) {
  form.value.variables.splice(index, 1);
}

async function handleSubmit() {
  if (!isFormValid.value) return;

  try {
    // Filter out empty variables
    const validVariables = form.value.variables.filter(v => v.name);

    const requestData = {
      id: form.value.template_id,
      name: form.value.name,
      description: form.value.description || undefined,
      category: form.value.category || undefined,
      command: form.value.command,
      args: form.value.args.length > 0 ? form.value.args : [],
      env: Object.keys(form.value.env).length > 0 ? form.value.env : {},
      default_cwd: form.value.default_cwd || undefined,
      tags: form.value.tags.length > 0 ? form.value.tags : undefined,
      variables: validVariables.length > 0 ? validVariables : [],
      default_auto_start: form.value.default_auto_start,
    };

    await apiClient.createTemplate(requestData);
    showSuccess(t('templates.createSuccess'));

    // Create ProcessTemplate object for emit
    const template: ProcessTemplate = {
      template_id: form.value.template_id,
      name: form.value.name,
      description: form.value.description || undefined,
      category: form.value.category || undefined,
      command: form.value.command,
      args: form.value.args.length > 0 ? form.value.args : undefined,
      env: Object.keys(form.value.env).length > 0 ? form.value.env : undefined,
      default_cwd: form.value.default_cwd || undefined,
      tags: form.value.tags.length > 0 ? form.value.tags : undefined,
      variables: validVariables.length > 0 ? validVariables : undefined,
      default_auto_start: form.value.default_auto_start,
    };

    emit('created', template);
    close();
  } catch (error: any) {
    showError(t('templates.createError', { error: error.message }));
  }
}

function close() {
  emit('update:modelValue', false);
  // Reset form
  form.value = {
    template_id: '',
    name: '',
    description: '',
    category: '',
    command: '',
    args: [],
    env: {},
    default_cwd: '',
    tags: [],
    variables: [],
    default_auto_start: false,
  };
  argsString.value = '';
  tagsString.value = '';
}
</script>

<style scoped>
.modal.show {
  background-color: rgba(0, 0, 0, 0.5);
}
</style>