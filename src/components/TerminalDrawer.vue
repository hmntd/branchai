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
      <div
        v-for="(log, idx) in logs"
        :key="idx"
        class="log-line"
        :class="log.type"
      >
        <pre>{{ log.text }}</pre>
      </div>

      <form @submit.prevent="runTerminalCommand" class="command-form">
        <ChevronRight :size="12" class="icon-primary" />
        <input
          v-model="commandInput"
          placeholder="Type shell / git command..."
          class="terminal-input"
        />
      </form>
    </div>
  </div>
</template>

<style scoped>
.terminal-drawer {
  position: absolute;
  bottom: 0;
  left: 0;
  right: 0;
  height: 200px;
  background: #0e1117;
  border-top: 1px solid var(--border-color);
  border-radius: 0;
  z-index: 100;
  display: flex;
  flex-direction: column;
}

.terminal-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 6px 12px;
  background: #181a1f;
  border-bottom: 1px solid var(--border-color);
  font-size: 11px;
  font-weight: 600;
}

.header-left, .header-right {
  display: flex;
  align-items: center;
  gap: 8px;
}

.terminal-body {
  flex: 1;
  padding: 10px;
  overflow-y: auto;
  font-family: var(--font-mono);
  font-size: 11px;
  display: flex;
  flex-direction: column;
  gap: 4px;
}

.log-line pre {
  margin: 0;
  white-space: pre-wrap;
  word-break: break-all;
}

.log-line.input {
  color: var(--primary);
  font-weight: 600;
}

.log-line.output {
  color: var(--text-muted);
}

.command-form {
  display: flex;
  align-items: center;
  gap: 6px;
  margin-top: 4px;
}

.terminal-input {
  flex: 1;
  background: transparent;
  border: none;
  font-family: var(--font-mono);
  font-size: 11px;
  color: var(--text-main);
}
.terminal-input:focus {
  box-shadow: none;
}

.icon-btn {
  background: transparent;
  border: none;
  color: var(--text-muted);
  cursor: pointer;
  padding: 2px;
}
.icon-btn:hover {
  color: var(--text-main);
}
</style>
