<template>
  <div
    class="vantage-process-card"
    :class="[`card-state--${stateClass}`, { 'card-state--running': isRunningState }]"
  >
    <div class="card-state-indicator" />

    <div class="card-content">
      <!-- Header -->
      <div class="card-header">
        <div class="card-title-group">
          <h3 class="card-title">{{ process.id }}</h3>
          <ProcessStatus :state="process.state" />
        </div>
        <div class="card-actions">
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

      <!-- Command -->
      <div class="card-command">
        <code>{{ commandLine }}</code>
      </div>

      <!-- Metadata -->
      <div class="card-meta" v-if="hasMetadata">
        <div v-if="process.cwd" class="meta-item">
          <IconFolder :size="16" :stroke-width="2" class="meta-icon" />
          <span class="meta-text">{{ process.cwd }}</span>
        </div>
        <div v-if="hasEnvVars" class="meta-item">
          <IconVariable :size="16" :stroke-width="2" class="meta-icon" />
          <span class="meta-text">{{ t('process.card.envVars', { count: envVarCount }) }}</span>
        </div>
        <div v-if="pid" class="meta-item">
          <IconHash :size="16" :stroke-width="2" class="meta-icon" />
          <span class="meta-text">{{ t('process.card.pid') }}: {{ pid }}</span>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed } from 'vue';
import { useI18n } from 'vue-i18n';
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
const { t } = useI18n();

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

const hasMetadata = computed(() => {
  return props.process.cwd || hasEnvVars.value || pid.value;
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

<style scoped lang="scss">
.vantage-process-card {
  position: relative;
  background: oklch(1 0 0);
  border-radius: 10px;
  border: 1px solid oklch(0.92 0 0);
  overflow: hidden;
  transition: all 0.2s ease;

  @media (prefers-color-scheme: dark) {
    background: oklch(0.2 0 0);
    border-color: oklch(0.28 0 0);
  }

  &:hover {
    transform: translateY(-2px);
    box-shadow: 0 8px 24px oklch(0 0 0 / 0.12);

    @media (prefers-color-scheme: dark) {
      box-shadow: 0 8px 24px oklch(0 0 0 / 0.4);
    }

    .card-state-indicator {
      width: 5px;
    }
  }
}

// State Indicator
.card-state-indicator {
  position: absolute;
  top: 0;
  left: 0;
  width: 4px;
  height: 100%;
  transition: width 0.2s ease;
}

.card-state--running .card-state-indicator {
  background: var(--vantage-btn-green-bg);
  animation: pulse 2s ease-in-out infinite;
}

.card-state--stopped .card-state-indicator {
  background: var(--vantage-btn-yellow-bg);
}

.card-state--failed .card-state-indicator {
  background: var(--vantage-btn-red-bg);
}

.card-state--notstarted .card-state-indicator {
  background: oklch(0.65 0.1 240);

  @media (prefers-color-scheme: dark) {
    background: oklch(0.7 0.12 240);
  }
}

@keyframes pulse {
  0%,
  100% {
    opacity: 0.6;
  }
  50% {
    opacity: 1;
  }
}

// Card Content
.card-content {
  padding: 1.25rem;
  padding-left: 1.5rem;
}

// Header
.card-header {
  display: flex;
  align-items: flex-start;
  justify-content: space-between;
  gap: 1rem;
  margin-bottom: 1rem;
}

.card-title-group {
  display: flex;
  align-items: center;
  gap: 0.75rem;
  flex-wrap: wrap;
  flex: 1;
  min-width: 0;
}

.card-title {
  margin: 0;
  font-size: 1.125rem;
  font-weight: 600;
  color: oklch(0.25 0 0);
  letter-spacing: -0.01em;
  word-break: break-word;

  @media (prefers-color-scheme: dark) {
    color: oklch(0.92 0 0);
  }
}

.card-actions {
  flex-shrink: 0;
}

// Command
.card-command {
  padding: 0.75rem 1rem;
  background: oklch(0.97 0 0);
  border-radius: 6px;
  border: 1px solid oklch(0.93 0 0);
  margin-bottom: 0.875rem;

  @media (prefers-color-scheme: dark) {
    background: oklch(0.16 0 0);
    border-color: oklch(0.25 0 0);
  }

  code {
    font-family: 'SF Mono', 'Monaco', 'Menlo', 'Consolas', monospace;
    font-size: 0.8125rem;
    line-height: 1.5;
    color: oklch(0.4 0 0);
    word-break: break-all;
    display: block;

    @media (prefers-color-scheme: dark) {
      color: oklch(0.75 0 0);
    }
  }
}

// Metadata
.card-meta {
  display: flex;
  flex-wrap: wrap;
  gap: 1rem;
}

.meta-item {
  display: flex;
  align-items: center;
  gap: 0.5rem;
  font-size: 0.8125rem;
}

.meta-icon {
  color: oklch(0.55 0 0);
  flex-shrink: 0;

  @media (prefers-color-scheme: dark) {
    color: oklch(0.6 0 0);
  }
}

.meta-text {
  color: oklch(0.5 0 0);
  line-height: 1.4;

  @media (prefers-color-scheme: dark) {
    color: oklch(0.65 0 0);
  }
}

// Responsive Design
@media (max-width: 768px) {
  .card-content {
    padding: 1rem;
    padding-left: 1.25rem;
  }

  .card-header {
    flex-direction: column;
    align-items: flex-start;
  }

  .card-title {
    font-size: 1rem;
  }

  .card-command code {
    font-size: 0.75rem;
  }

  .card-meta {
    gap: 0.75rem;
    font-size: 0.75rem;
  }
}

@media (max-width: 576px) {
  .card-state-indicator {
    width: 3px;
  }

  .vantage-process-card:hover .card-state-indicator {
    width: 4px;
  }

  .card-content {
    padding: 0.875rem;
    padding-left: 1rem;
  }

  .card-title {
    font-size: 0.9375rem;
  }
}
</style>