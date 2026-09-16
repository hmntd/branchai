<script setup lang="ts">
import { ref } from 'vue';
import { Terminal as TerminalIcon, X, Trash2, ChevronRight } from '@lucide/vue';

const props = defineProps<{
  repoPath: string;
}>();

const emit = defineEmits<{
  (e: 'close'): void;
}>();

const commandInput = ref('');
const logs = ref<Array<{ type: 'input' | 'output' | 'error'; text: string }>>([
  { type: 'output', text: 'BranchAI Embedded Terminal v0.1.0' },
  { type: 'output', text: `Active workspace: ${props.repoPath}` },
]);

function runTerminalCommand() {
  const cmd = commandInput.value.trim();
  if (!cmd) return;

  logs.value.push({ type: 'input', text: `$ ${cmd}` });
  commandInput.value = '';

  if (cmd === 'clear') {
    logs.value = [];
    return;
  }

  // Simulated CLI output
  if (cmd.startsWith('git status')) {
    logs.value.push({ type: 'output', text: 'On branch master\nChanges not staged for commit:\n  modified: src/App.vue\n  modified: src-tauri/Cargo.toml' });
  } else if (cmd.startsWith('git branch')) {
    logs.value.push({ type: 'output', text: '* master\n  feature/ai-commit' });
  } else if (cmd === 'pwd') {
    logs.value.push({ type: 'output', text: props.repoPath });
  } else {
    logs.value.push({ type: 'output', text: `Executing: ${cmd}... Done.` });
  }
}

function clearLogs() {
  logs.value = [];
}
</script>

<template>
  <div class="terminal-drawer gk-panel">
    <div class="terminal-header">
      <div class="header-left">
        <TerminalIcon :size="13" class="icon-primary" />
        <span class="title">Terminal - {{ repoPath.split('/').pop() }}</span>
      </div>
      <div class="header-right">
        <button class="icon-btn" @click="clearLogs" title="Clear Console">
          <Trash2 :size="13" />
        </button>
        <button class="icon-btn" @click="emit('close')" title="Close Terminal">
          <X :size="13" />
        </button>
      </div>
    </div>

    <div class="terminal-body">
      <div v-for="(log, idx) in logs" :key="idx" class="log-line" :class="log.type">
        <pre>{{ log.text }}</pre>
      </div>

      <form @submit.prevent="runTerminalCommand" class="command-form">
        <ChevronRight :size="12" class="icon-primary" />
        <input v-model="commandInput" placeholder="Type shell / git command..." class="terminal-input" />
      </form>
    </div>
  </div>
</template>

<style scoped src="../../styles/organisms/TerminalDrawer.css"></style>
