<script setup lang="ts">
import { ref, onMounted } from 'vue';
import { invoke } from '@tauri-apps/api/core';
import { AlertOctagon, Sparkles, RefreshCw, CheckCircle, FileCode, X } from '@lucide/vue';

export interface ConflictFile {
  path: string;
  content: string;
}

const props = defineProps<{
  repoPath: string;
}>();

const emit = defineEmits<{
  (e: 'resolved'): void;
  (e: 'close'): void;
}>();

const conflicts = ref<ConflictFile[]>([]);
const selectedConflict = ref<ConflictFile | null>(null);
const aiResolution = ref<string>('');
const isSolving = ref(false);
const isApplying = ref(false);
const statusMessage = ref('');

async function loadConflicts() {
  try {
    const list: ConflictFile[] = await invoke('get_conflicts', { repoPath: props.repoPath });
    conflicts.value = list;
    if (list.length > 0) {
      selectedConflict.value = list[0];
    } else {
      selectedConflict.value = null;
    }
  } catch (e) {
    console.error(e);
  }
}

async function solveConflictWithAi() {
  if (!selectedConflict.value) return;
  isSolving.value = true;
  statusMessage.value = '';
  aiResolution.value = '';

  try {
    const resolution: string = await invoke('assist_conflict_resolution', {
      filePath: selectedConflict.value.path,
      conflictContent: selectedConflict.value.content,
    });
    aiResolution.value = resolution;
    statusMessage.value = 'AI resolution generated. Review below and click Apply.';
  } catch (err: any) {
    statusMessage.value = typeof err === 'string' ? err : err.message || 'AI resolution failed.';
  } finally {
    isSolving.value = false;
  }
}

async function applyResolution() {
  if (!selectedConflict.value || !aiResolution.value) return;
  isApplying.value = true;

  try {
    await invoke('resolve_conflict', {
      repoPath: props.repoPath,
      filePath: selectedConflict.value.path,
      content: aiResolution.value,
    });
    statusMessage.value = `Resolved ${selectedConflict.value.path} successfully!`;
    aiResolution.value = '';
    await loadConflicts();
    emit('resolved');
  } catch (err: any) {
    statusMessage.value = typeof err === 'string' ? err : err.message || 'Failed to save resolution.';
  } finally {
    isApplying.value = false;
  }
}

onMounted(() => {
  loadConflicts();
});
</script>

<template>
  <div class="conflict-container">
    <div class="conflict-sidebar glass-panel">
      <div class="sidebar-header">
        <AlertOctagon :size="18" class="icon-danger" />
        <h3 class="header-title">Conflict Assistant</h3>
        <button class="icon-btn" @click="emit('close')" title="Close & Back to Graph">
          <X :size="16" />
        </button>
      </div>

      <div class="conflict-list">
        <div
          v-for="c in conflicts"
          :key="c.path"
          class="conflict-item"
          :class="{ active: selectedConflict?.path === c.path }"
          @click="selectedConflict = c"
        >
          <FileCode :size="14" class="icon-warning" />
          <span class="file-path">{{ c.path }}</span>
        </div>
        <div v-if="conflicts.length === 0" class="no-conflicts">
          <CheckCircle :size="24" class="icon-success" />
          <p>No merge conflicts detected in repository!</p>
        </div>
      </div>
    </div>

    <div class="conflict-main glass-panel">
      <div class="main-header" v-if="selectedConflict">
        <span class="file-title">{{ selectedConflict.path }}</span>
        <div class="header-actions">
          <button @click="solveConflictWithAi" class="btn btn-ai btn-sm" :disabled="isSolving">
            <RefreshCw v-if="isSolving" :size="14" class="spinning" />
            <Sparkles v-else :size="14" />
            {{ isSolving ? 'Solving...' : 'Ask AI to Resolve' }}
          </button>
          <button class="btn btn-secondary btn-xs" @click="emit('close')">
            <X :size="14" /> Back to Graph
          </button>
        </div>
      </div>

      <div class="main-body" v-if="selectedConflict">
        <div class="split-view">
          <div class="view-box">
            <div class="box-title">Current Conflict Block (with markers)</div>
            <pre class="code-block">{{ selectedConflict.content }}</pre>
          </div>

          <div class="view-box">
            <div class="box-title">AI Proposed Resolution</div>
            <textarea
              v-model="aiResolution"
              placeholder="AI resolution will appear here, or edit manually..."
              class="resolution-textarea"
            ></textarea>
            <button
              v-if="aiResolution"
              @click="applyResolution"
              class="btn btn-primary btn-sm apply-btn"
              :disabled="isApplying"
            >
              <CheckCircle :size="14" />
              {{ isApplying ? 'Applying...' : 'Apply & Stage Resolution' }}
            </button>
          </div>
        </div>

        <div v-if="statusMessage" class="status-banner">
          {{ statusMessage }}
        </div>
      </div>

      <div class="empty-state" v-else>
        <AlertOctagon :size="48" class="icon-dim" />
        <p>No active merge conflict selected.</p>
        <button class="btn btn-secondary btn-sm" @click="emit('close')">
          <X :size="14" /> Back to Graph
        </button>
      </div>
    </div>
  </div>
</template>

<style scoped>
.conflict-container {
  display: grid;
  grid-template-columns: 320px 1fr;
  gap: 16px;
  height: 100%;
  padding: 16px;
}

.conflict-sidebar {
  display: flex;
  flex-direction: column;
  gap: 12px;
  padding: 16px;
}

.sidebar-header {
  display: flex;
  align-items: center;
  gap: 8px;
  font-size: 14px;
  font-weight: 600;
}

.header-title {
  flex: 1;
}

.conflict-list {
  display: flex;
  flex-direction: column;
  gap: 6px;
  overflow-y: auto;
}

.conflict-item {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 8px 10px;
  border-radius: 6px;
  cursor: pointer;
  font-size: 12px;
  background: rgba(255, 255, 255, 0.03);
}

.conflict-item.active {
  background: rgba(248, 113, 113, 0.15);
  border: 1px solid rgba(248, 113, 113, 0.3);
  color: var(--danger);
}

.no-conflicts {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  padding: 40px 10px;
  gap: 10px;
  color: var(--text-muted);
  text-align: center;
  font-size: 13px;
}

.conflict-main {
  display: flex;
  flex-direction: column;
  overflow: hidden;
}

.main-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 12px 16px;
  border-bottom: 1px solid var(--border-color);
}

.header-actions {
  display: flex;
  align-items: center;
  gap: 8px;
}

.file-title {
  font-family: var(--font-mono);
  font-size: 13px;
  font-weight: 600;
}

.main-body {
  flex: 1;
  display: flex;
  flex-direction: column;
  gap: 12px;
  padding: 16px;
  overflow: hidden;
}

.split-view {
  display: grid;
  grid-template-columns: 1fr 1fr;
  gap: 16px;
  flex: 1;
}

.view-box {
  display: flex;
  flex-direction: column;
  gap: 8px;
  background: rgba(0, 0, 0, 0.4);
  padding: 12px;
  border-radius: 8px;
  border: 1px solid var(--border-color);
}

.box-title {
  font-size: 12px;
  font-weight: 600;
  color: var(--text-muted);
}

.code-block {
  font-family: var(--font-mono);
  font-size: 12px;
  line-height: 1.5;
  color: #f87171;
  overflow: auto;
  white-space: pre-wrap;
  flex: 1;
}

.resolution-textarea {
  flex: 1;
  font-family: var(--font-mono);
  font-size: 12px;
  line-height: 1.5;
  background: rgba(0, 0, 0, 0.3);
}

.apply-btn {
  align-self: flex-end;
}

.status-banner {
  padding: 8px 12px;
  border-radius: 6px;
  background: rgba(56, 189, 248, 0.15);
  color: var(--primary);
  font-size: 12px;
}

.empty-state {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  height: 100%;
  gap: 12px;
  color: var(--text-dim);
}

.icon-btn {
  background: transparent;
  border: none;
  color: var(--text-muted);
  cursor: pointer;
}
.icon-btn:hover {
  color: var(--text-main);
}

.btn-xs { padding: 3px 6px; font-size: 10px; }
.spinning { animation: spin 1s linear infinite; }

@keyframes spin { 100% { transform: rotate(360deg); } }
</style>
