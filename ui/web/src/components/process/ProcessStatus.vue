<template>
  <span
    class="badge badge-status"
    :class="[badgeClass, { 'badge-running': isRunningState }]"
  >
    <component :is="statusIcon" class="badge-icon" />
    <span class="badge-label">{{ label }}</span>
  </span>
</template>

<script setup lang="ts">
import { computed } from 'vue';
import {
  IconPlayerPlay,
  IconPlayerPause,
  IconAlertCircle,
  IconCircle
} from '@tabler/icons-vue';
import type { ProcessState } from '@/types';
import { getStateLabel, getStateColor, isRunning, isStopped, isFailed } from '@/types';

interface Props {
  state: ProcessState;
}

const props = defineProps<Props>();

const label = computed(() => getStateLabel(props.state));
const color = computed(() => getStateColor(props.state));
const isRunningState = computed(() => isRunning(props.state));

const badgeClass = computed(() => {
  const colorMap: Record<string, string> = {
    green: 'badge-green',
    yellow: 'badge-yellow',
    red: 'badge-red',
    secondary: 'badge-secondary',
    gray: 'badge-gray',
  };
  return colorMap[color.value] || 'badge-secondary';
});

const statusIcon = computed(() => {
  if (isRunning(props.state)) return IconPlayerPlay;
  if (isFailed(props.state)) return IconAlertCircle;
  if (isStopped(props.state)) return IconPlayerPause;
  return IconCircle;
});
</script>

<style scoped>
.badge-status {
  display: inline-flex;
  align-items: center;
  gap: 0.375rem;
  padding: 0.375rem 0.75rem;
  font-size: 0.8125rem;
  font-weight: 600;
  border-radius: 0.375rem;
  transition: all 0.2s ease;
}

.badge-icon {
  width: 1rem;
  height: 1rem;
  flex-shrink: 0;
}

.badge-label {
  line-height: 1;
}

/* Color variants */
.badge-green {
  background-color: var(--tblr-green-lt);
  color: var(--tblr-green);
  border: 1px solid var(--tblr-green);
}

.badge-red {
  background-color: var(--tblr-red-lt);
  color: var(--tblr-red);
  border: 1px solid var(--tblr-red);
}

.badge-yellow {
  background-color: var(--tblr-yellow-lt);
  color: var(--tblr-yellow);
  border: 1px solid var(--tblr-yellow);
}

.badge-secondary,
.badge-gray {
  background-color: var(--tblr-secondary-lt);
  color: var(--tblr-secondary);
  border: 1px solid var(--tblr-secondary);
}

/* Running animation */
.badge-running {
  animation: pulse-badge 2s ease-in-out infinite;
}

@keyframes pulse-badge {
  0%, 100% {
    box-shadow: 0 0 0 0 rgba(var(--tblr-green-rgb), 0.4);
  }
  50% {
    box-shadow: 0 0 0 4px rgba(var(--tblr-green-rgb), 0);
  }
}

/* Hover effect */
.badge-status:hover {
  transform: translateY(-1px);
}
</style>