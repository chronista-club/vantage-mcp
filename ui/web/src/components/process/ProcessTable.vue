<template>
  <div class="table-responsive">
    <table class="table table-vcenter table-nowrap">
      <thead>
        <tr>
          <th>Process ID</th>
          <th>Command</th>
          <th>Status</th>
          <th>PID</th>
          <th>Working Directory</th>
          <th class="w-1">Actions</th>
        </tr>
      </thead>
      <tbody>
        <tr v-for="process in processes" :key="process.id">
          <td>
            <div class="fw-bold">{{ process.id }}</div>
          </td>
          <td>
            <code class="text-sm">{{ getCommandLine(process) }}</code>
          </td>
          <td>
            <ProcessStatus :state="process.state" />
          </td>
          <td>
            <span v-if="getPid(process.state)">{{ getPid(process.state) }}</span>
            <span v-else class="text-muted">-</span>
          </td>
          <td>
            <span v-if="process.cwd" class="text-muted small">{{ process.cwd }}</span>
            <span v-else class="text-muted">-</span>
          </td>
          <td>
            <ProcessActions
              :process="process"
              :small="true"
              :icon-only="true"
              @start="handleStart"
              @stop="handleStop"
              @remove="handleRemove"
            />
          </td>
        </tr>
      </tbody>
    </table>
  </div>
</template>

<script setup lang="ts">
import type { ProcessInfo, ProcessState } from '@/types';
import { isRunning } from '@/types';
import ProcessStatus from './ProcessStatus.vue';
import ProcessActions from './ProcessActions.vue';
import { useProcessStore } from '@/stores/process';

interface Props {
  processes: ProcessInfo[];
}

defineProps<Props>();

const processStore = useProcessStore();

function getCommandLine(process: ProcessInfo): string {
  const args = process.args.join(' ');
  return args ? `${process.command} ${args}` : process.command;
}

function getPid(state: ProcessState): number | null {
  if (isRunning(state) && typeof state === 'object' && 'Running' in state) {
    return state.Running.pid;
  }
  return null;
}

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
</script>