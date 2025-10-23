<template>
  <div
    class="card process-card"
    :class="[`process-card--${stateClass}`, { 'process-card--running': isRunningState }]"
  >
    <div class="card-body">
      <div class="d-flex align-items-start">
        <!-- Left: Process Info -->
        <div class="flex-grow-1">
          <div class="d-flex align-items-center mb-2">
            <h3 class="mb-0 me-3">{{ process.id }}</h3>
            <ProcessStatus :state="process.state" />
          </div>

          <div class="process-command mb-2">
            <code class="text-muted">{{ commandLine }}</code>
          </div>

          <div class="process-meta">
            <div v-if="process.cwd" class="meta-item">
              <IconFolder class="meta-icon" />
              <span class="text-muted">{{ process.cwd }}</span>
            </div>
            <div v-if="hasEnvVars" class="meta-item">
              <IconVariable class="meta-icon" />
              <span class="text-muted">{{ envVarCount }} env var{{ envVarCount !== 1 ? 's' : '' }}</span>
            </div>
            <div v-if="pid" class="meta-item">
              <IconHash class="meta-icon" />
              <span class="text-muted">PID: {{ pid }}</span>
            </div>
          </div>
        </div>

        <!-- Right: Actions -->
        <div class="ms-3 process-actions-wrapper">
          <ProcessActions
            :process="process"
            :show-output="true"
            @start="handleStart"
            @stop="handleStop"
            @remove="handleRemove"
            @show-output="handleShowOutput"
          />
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed } from 'vue';
import { IconFolder, IconVariable, IconHash } from '@tabler/icons-vue';
import type { ProcessInfo } from '@/types';
import { isRunning, isStopped, isFailed } from '@/types';
import ProcessStatus from './ProcessStatus.vue';
import ProcessActions from './ProcessActions.vue';
import { useProcessStore } from '@/stores/process';

interface Props {
  process: ProcessInfo;
}

const props = defineProps<Props>();

const processStore = useProcessStore();

const commandLine = computed(() => {
  const args = props.process.args.join(' ');
  return args ? `${props.process.command} ${args}` : props.process.command;
});

const hasEnvVars = computed(() => {
  return props.process.env && Object.keys(props.process.env).length > 0;
});

const envVarCount = computed(() => {
  return props.process.env ? Object.keys(props.process.env).length : 0;
});

const isRunningState = computed(() => isRunning(props.process.state));

const stateClass = computed(() => {
  if (isRunning(props.process.state)) return 'running';
  if (isStopped(props.process.state)) return 'stopped';
  if (isFailed(props.process.state)) return 'failed';
  return 'notstarted';
});

const pid = computed(() => {
  if (typeof props.process.state === 'object' && 'Running' in props.process.state) {
    return props.process.state.Running.pid;
  }
  return null;
});

async function handleStart(id: string) {
  try {
    await processStore.startProcess(id);
  } catch (error) {
    console.error('Failed to start process:', error);
  }
}

async function handleStop(id: string) {
  try {
    await processStore.stopProcess(id);
  } catch (error) {
    console.error('Failed to stop process:', error);
  }
}

async function handleRemove(id: string) {
  try {
    await processStore.removeProcess(id);
  } catch (error) {
    console.error('Failed to remove process:', error);
  }
}

function handleShowOutput(id: string) {
  // TODO: Implement output modal
  console.log('Show output for process:', id);
}
</script>

<style scoped>
.process-card {
  border-left: 4px solid;
  transition: all 0.3s ease;
  position: relative;
  overflow: hidden;
}

.process-card::before {
  content: '';
  position: absolute;
  top: 0;
  left: 0;
  width: 4px;
  height: 100%;
  transition: width 0.3s ease;
}

.process-card:hover {
  transform: translateY(-2px);
  box-shadow: 0 4px 12px rgba(0, 0, 0, 0.1);
}

.process-card:hover::before {
  width: 6px;
}

/* State-based border colors */
.process-card--running {
  border-left-color: var(--tblr-green);
}

.process-card--stopped {
  border-left-color: var(--tblr-secondary);
}

.process-card--failed {
  border-left-color: var(--tblr-red);
}

.process-card--notstarted {
  border-left-color: var(--tblr-azure);
}

/* Running animation */
.process-card--running::after {
  content: '';
  position: absolute;
  top: 0;
  left: 0;
  width: 4px;
  height: 100%;
  background: linear-gradient(
    to bottom,
    transparent 0%,
    var(--tblr-green) 50%,
    transparent 100%
  );
  animation: pulse-border 2s ease-in-out infinite;
}

@keyframes pulse-border {
  0%,
  100% {
    opacity: 0.3;
  }
  50% {
    opacity: 1;
  }
}

.process-command {
  padding: 0.5rem 0.75rem;
  background: var(--tblr-bg-surface-secondary);
  border-radius: 0.375rem;
  border: 1px solid var(--tblr-border-color-translucent);
}

.process-command code {
  font-size: 0.875rem;
  word-break: break-all;
}

.process-meta {
  display: flex;
  flex-wrap: wrap;
  gap: 1rem;
  font-size: 0.8125rem;
}

.meta-item {
  display: flex;
  align-items: center;
  gap: 0.375rem;
}

.meta-icon {
  width: 1rem;
  height: 1rem;
  opacity: 0.7;
}

.process-actions-wrapper {
  flex-shrink: 0;
}

@media (max-width: 768px) {
  .process-card {
    border-left-width: 3px;
  }

  .process-meta {
    gap: 0.75rem;
    font-size: 0.75rem;
  }
}
</style>