<template>
  <div class="btn-list">
    <button 
      @click="handleStart"
      class="btn btn-green"
      :class="{ 'btn-sm': small }"
      :disabled="!canStart || starting"
      :title="startTooltip"
    >
      <IconPlayerPlay v-if="!small || !iconOnly" />
      <span v-if="!iconOnly" class="ms-1">Start</span>
    </button>
    
    <button 
      @click="handleStop"
      class="btn btn-yellow"
      :class="{ 'btn-sm': small }"
      :disabled="!canStop || stopping"
      :title="stopTooltip"
    >
      <IconPlayerStop v-if="!small || !iconOnly" />
      <span v-if="!iconOnly" class="ms-1">Stop</span>
    </button>
    
    <button 
      @click="handleRemove"
      class="btn btn-red"
      :class="{ 'btn-sm': small }"
      :disabled="!canRemove || removing"
      :title="removeTooltip"
    >
      <IconTrash v-if="!small || !iconOnly" />
      <span v-if="!iconOnly" class="ms-1">Remove</span>
    </button>
    
    <button 
      v-if="showOutput"
      @click="handleShowOutput"
      class="btn btn-ghost-secondary"
      :class="{ 'btn-sm': small }"
      title="Show Output"
    >
      <IconTerminal />
      <span v-if="!iconOnly" class="ms-1">Output</span>
    </button>
  </div>
</template>

<script setup lang="ts">
import { ref, computed } from 'vue';
import { IconPlayerPlay, IconPlayerStop, IconTrash, IconTerminal } from '@tabler/icons-vue';
import type { ProcessInfo } from '@/types';
import { isRunning, isNotStarted } from '@/types';

interface Props {
  process: ProcessInfo;
  small?: boolean;
  iconOnly?: boolean;
  showOutput?: boolean;
}

const props = withDefaults(defineProps<Props>(), {
  small: false,
  iconOnly: false,
  showOutput: false,
});

const emit = defineEmits<{
  start: [id: string];
  stop: [id: string];
  remove: [id: string];
  showOutput: [id: string];
}>();

const starting = ref(false);
const stopping = ref(false);
const removing = ref(false);

const canStart = computed(() => isNotStarted(props.process.state) || !isRunning(props.process.state));
const canStop = computed(() => isRunning(props.process.state));
const canRemove = computed(() => !isRunning(props.process.state));

const startTooltip = computed(() => {
  if (starting.value) return 'Starting...';
  if (!canStart.value) return 'Process is already running';
  return 'Start Process';
});

const stopTooltip = computed(() => {
  if (stopping.value) return 'Stopping...';
  if (!canStop.value) return 'Process is not running';
  return 'Stop Process';
});

const removeTooltip = computed(() => {
  if (removing.value) return 'Removing...';
  if (!canRemove.value) return 'Cannot remove running process';
  return 'Remove Process';
});

async function handleStart() {
  if (!canStart.value || starting.value) return;
  
  starting.value = true;
  try {
    emit('start', props.process.id);
  } finally {
    setTimeout(() => {
      starting.value = false;
    }, 1000);
  }
}

async function handleStop() {
  if (!canStop.value || stopping.value) return;
  
  stopping.value = true;
  try {
    emit('stop', props.process.id);
  } finally {
    setTimeout(() => {
      stopping.value = false;
    }, 1000);
  }
}

async function handleRemove() {
  if (!canRemove.value || removing.value) return;
  
  if (!confirm(`Are you sure you want to remove process "${props.process.id}"?`)) {
    return;
  }
  
  removing.value = true;
  try {
    emit('remove', props.process.id);
  } finally {
    setTimeout(() => {
      removing.value = false;
    }, 1000);
  }
}

function handleShowOutput() {
  emit('showOutput', props.process.id);
}
</script>