<template>
  <span 
    class="badge"
    :class="badgeClass"
  >
    {{ label }}
  </span>
  <span v-if="pid" class="ms-2 text-muted">
    PID: {{ pid }}
  </span>
</template>

<script setup lang="ts">
import { computed } from 'vue';
import type { ProcessState } from '@/types';
import { getStateLabel, getStateColor, isRunning } from '@/types';

interface Props {
  state: ProcessState;
}

const props = defineProps<Props>();

const label = computed(() => getStateLabel(props.state));
const color = computed(() => getStateColor(props.state));

const badgeClass = computed(() => {
  const colorMap: Record<string, string> = {
    green: 'bg-green',
    yellow: 'bg-yellow',
    red: 'bg-red',
    secondary: 'bg-secondary',
    gray: 'bg-gray',
  };
  return colorMap[color.value] || 'bg-secondary';
});

const pid = computed(() => {
  if (isRunning(props.state) && typeof props.state === 'object' && 'Running' in props.state) {
    return props.state.Running.pid;
  }
  return null;
});
</script>