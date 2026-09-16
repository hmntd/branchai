<script setup lang="ts">
import { ref, onMounted } from 'vue';
import { invoke } from '@tauri-apps/api/core';
import { AlertOctagon, Sparkles, RefreshCw, CheckCircle, FileCode, X } from '@lucide/vue';
import BaseButton from '../atoms/BaseButton.vue';

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
        <div v-for="c in conflicts" :key="c.path" class="conflict-item"
          :class="{ active: selectedConflict?.path === c.path }" @click="selectedConflict = c">
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
          <BaseButton variant="ai" size="sm" @click="solveConflictWithAi" :disabled="isSolving">
            <RefreshCw v-if="isSolving" :size="14" class="spinning" />
            <Sparkles v-else :size="14" />
            {{ isSolving ? 'Solving...' : 'Ask AI to Resolve' }}
          </BaseButton>
          <BaseButton variant="secondary" size="xs" @click="emit('close')">
            <X :size="14" /> Back to Graph
          </BaseButton>
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
            <textarea v-model="aiResolution" placeholder="AI resolution will appear here, or edit manually..."
              class="resolution-textarea"></textarea>
            <BaseButton v-if="aiResolution" variant="primary" size="sm" class="apply-btn" @click="applyResolution"
              :disabled="isApplying">
              <CheckCircle :size="14" />
              {{ isApplying ? 'Applying...' : 'Apply & Stage Resolution' }}
            </BaseButton>
          </div>
        </div>

        <div v-if="statusMessage" class="status-banner">
          {{ statusMessage }}
        </div>
      </div>

      <div class="empty-state" v-else>
        <AlertOctagon :size="48" class="icon-dim" />
        <p>No active merge conflict selected.</p>
        <BaseButton variant="secondary" size="sm" @click="emit('close')">
          <X :size="14" /> Back to Graph
        </BaseButton>
      </div>
    </div>
  </div>
</template>

<style scoped src="../../styles/organisms/ConflictResolver.css"></style>
