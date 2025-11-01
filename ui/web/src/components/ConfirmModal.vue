<template>
  <div
    v-if="modelValue"
    class="modal modal-blur fade show d-block"
    tabindex="-1"
    @click.self="onCancel"
  >
    <div class="modal-dialog modal-sm modal-dialog-centered">
      <div class="modal-content">
        <div class="modal-header">
          <h5 class="modal-title">{{ title }}</h5>
          <button
            type="button"
            class="btn-close"
            @click="onCancel"
          ></button>
        </div>

        <div class="modal-body">
          <p>{{ message }}</p>
        </div>

        <div class="modal-footer">
          <button
            type="button"
            class="btn btn-secondary"
            @click="onCancel"
          >
            {{ cancelText || $t('common.cancel') }}
          </button>
          <button
            type="button"
            :class="['btn', dangerMode ? 'btn-danger' : 'btn-primary']"
            @click="onConfirm"
          >
            {{ confirmText || $t('common.confirm') }}
          </button>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
defineProps<{
  modelValue: boolean;
  title: string;
  message: string;
  confirmText?: string;
  cancelText?: string;
  dangerMode?: boolean;
}>();

const emit = defineEmits<{
  'update:modelValue': [value: boolean];
  'confirm': [];
  'cancel': [];
}>();

function onConfirm() {
  emit('confirm');
  emit('update:modelValue', false);
}

function onCancel() {
  emit('cancel');
  emit('update:modelValue', false);
}
</script>

<style scoped>
.modal.show {
  background-color: rgba(0, 0, 0, 0.5);
}
</style>
