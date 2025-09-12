<template>
  <div class="card process-card">
    <div class="card-body">
      <div class="row align-items-center">
        <div class="col">
          <h3 class="mb-1">{{ process.id }}</h3>
          <div class="text-muted">
            <code>{{ commandLine }}</code>
          </div>
          <div class="mt-2">
            <ProcessStatus :state="process.state" />
          </div>
          <div v-if="process.cwd" class="mt-1 text-muted small">
            <IconFolder class="icon-sm" /> {{ process.cwd }}
          </div>
          <div v-if="hasEnvVars" class="mt-1 text-muted small">
            <IconVariable class="icon-sm" /> {{ envVarCount }} environment variable{{ envVarCount !== 1 ? 's' : '' }}
          </div>
        </div>
        <div class="col-auto">
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
import { IconFolder, IconVariable } from '@tabler/icons-vue';
import type { ProcessInfo } from '@/types';
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
.icon-sm {
  width: 1rem;
  height: 1rem;
  vertical-align: text-bottom;
}
</style>