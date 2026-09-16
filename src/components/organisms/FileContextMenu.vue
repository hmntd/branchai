<script setup lang="ts">
import { ref, watch, onMounted, onUnmounted, nextTick } from 'vue';
import { invoke } from '@tauri-apps/api/core';
import {
  Plus,
  Trash2,
  Archive,
  History,
  GitBranch,
  GitCompare,
  Code2,
  ExternalLink,
  FolderOpen,
  Copy,
  FileCode2,
  Pencil,
  X,
} from '@lucide/vue';

export interface FileStatus {
  path: string;
  status: string;
  staged: boolean;
}

const props = defineProps<{
  visible: boolean;
  x: number;
  y: number;
  file: FileStatus | null;
  repoPath: string;
}>();

const emit = defineEmits<{
  (e: 'close'): void;
  (e: 'refresh'): void;
}>();

const menuEl = ref<HTMLElement | null>(null);
const adjustedX = ref(props.x);
const adjustedY = ref(props.y);

const toast = ref('');
const toastError = ref(false);
let toastTimer: ReturnType<typeof setTimeout> | null = null;

function showToast(msg: string, isError = false) {
  toast.value = msg;
  toastError.value = isError;
  if (toastTimer) clearTimeout(toastTimer);
  toastTimer = setTimeout(() => (toast.value = ''), 3000);
}

watch(
  () => props.visible,
  async (isVisible) => {
    if (!isVisible) return;
    adjustedX.value = props.x;
    adjustedY.value = props.y;
    await nextTick();
    if (!menuEl.value) return;
    const rect = menuEl.value.getBoundingClientRect();
    const vw = window.innerWidth;
    const vh = window.innerHeight;
    adjustedX.value = props.x + rect.width > vw ? vw - rect.width - 8 : props.x;
    adjustedY.value = props.y + rect.height > vh ? vh - rect.height - 8 : props.y;
  }
);

function onOutsideClick(e: MouseEvent) {
  if (menuEl.value && !menuEl.value.contains(e.target as Node)) {
    emit('close');
  }
}
onMounted(() => window.addEventListener('mousedown', onOutsideClick, true));
onUnmounted(() => {
  window.removeEventListener('mousedown', onOutsideClick, true);
  if (toastTimer) clearTimeout(toastTimer);
});

function close() {
  emit('close');
}

function fullPath(): string {
  return `${props.repoPath}/${props.file?.path}`;
}

async function stageFile() {
  if (!props.file) return;
  try {
    await invoke('stage_file', { repoPath: props.repoPath, filePath: props.file.path });
    emit('refresh');
    showToast('File staged');
  } catch (e: any) {
    showToast(String(e), true);
  }
  close();
}

async function discardChanges() {
  if (!props.file) return;
  try {
    await invoke('discard_file_changes', { repoPath: props.repoPath, filePath: props.file.path });
    emit('refresh');
    showToast('Changes discarded');
  } catch (e: any) {
    showToast(String(e), true);
  }
  close();
}

async function stashFile() {
  if (!props.file) return;
  try {
    await invoke('stash_file', { repoPath: props.repoPath, filePath: props.file.path });
    emit('refresh');
    showToast('File stashed');
  } catch (e: any) {
    showToast(String(e), true);
  }
  close();
}

const showHistoryModal = ref(false);
const historyEntries = ref<{ id: string; author: string; message: string; time: number }[]>([]);
const historyLoading = ref(false);

async function fileHistory() {
  if (!props.file) return;
  close();
  showHistoryModal.value = true;
  historyLoading.value = true;
  try {
    const data: { id: string; author: string; message: string; time: number }[] = await invoke(
      'get_file_history',
      { repoPath: props.repoPath, filePath: props.file.path }
    );
    historyEntries.value = data;
  } catch (e: any) {
    showToast(String(e), true);
  } finally {
    historyLoading.value = false;
  }
}

const showBlameModal = ref(false);
const blameLines = ref<{ line_no: number; author: string; time: number; content: string }[]>([]);
const blameLoading = ref(false);

async function fileBlame() {
  if (!props.file) return;
  close();
  showBlameModal.value = true;
  blameLoading.value = true;
  try {
    const data: { line_no: number; author: string; time: number; content: string }[] =
      await invoke('get_file_blame', { repoPath: props.repoPath, filePath: props.file.path });
    blameLines.value = data;
  } catch (e: any) {
    showToast(String(e), true);
  } finally {
    blameLoading.value = false;
  }
}

async function openExternalDiff() {
  if (!props.file) return;
  try {
    await invoke('open_in_external_diff', { repoPath: props.repoPath, filePath: props.file.path });
  } catch (e: any) {
    showToast(String(e), true);
  }
  close();
}

async function openInVscode() {
  if (!props.file) return;
  try {
    await invoke('open_in_vscode', { filePath: fullPath() });
  } catch (e: any) {
    showToast(String(e), true);
  }
  close();
}

async function openInDefaultProgram() {
  if (!props.file) return;
  try {
    await invoke('open_file_default', { filePath: fullPath() });
  } catch (e: any) {
    showToast(String(e), true);
  }
  close();
}

async function showInFolder() {
  if (!props.file) return;
  try {
    await invoke('show_in_folder', { filePath: fullPath() });
  } catch (e: any) {
    showToast(String(e), true);
  }
  close();
}

async function copyFilePath() {
  if (!props.file) return;
  try {
    await navigator.clipboard.writeText(fullPath());
    showToast('Path copied to clipboard');
  } catch {
    showToast('Failed to copy path', true);
  }
  close();
}

const showPatchModal = ref(false);
const patchContent = ref('');
const patchLoading = ref(false);

async function createPatch() {
  if (!props.file) return;
  close();
  showPatchModal.value = true;
  patchLoading.value = true;
  try {
    const content: string = await invoke('create_patch_from_file', {
      repoPath: props.repoPath,
      filePath: props.file.path,
    });
    patchContent.value = content;
  } catch (e: any) {
    showToast(String(e), true);
  } finally {
    patchLoading.value = false;
  }
}

async function editFile() {
  if (!props.file) return;
  try {
    await invoke('open_in_vscode', { filePath: fullPath() });
  } catch (e: any) {
    showToast(String(e), true);
  }
  close();
}

async function deleteFile() {
  if (!props.file) return;
  if (!confirm(`Delete "${props.file.path}"? This cannot be undone.`)) return;
  try {
    await invoke('delete_file', { filePath: fullPath() });
    emit('refresh');
    showToast('File deleted');
  } catch (e: any) {
    showToast(String(e), true);
  }
  close();
}

function formatDate(ts: number) {
  return new Date(ts * 1000).toLocaleString();
}

async function savePatchToFile() {
  if (!props.file) return;
  const blob = new Blob([patchContent.value], { type: 'text/plain' });
  const url = URL.createObjectURL(blob);
  const a = document.createElement('a');
  a.href = url;
  a.download = `${props.file.path.replace(/\//g, '_')}.patch`;
  a.click();
  URL.revokeObjectURL(url);
}
</script>

<template>
  <!-- Toast -->
  <Teleport to="body">
    <div v-if="toast" class="ctx-toast" :class="toastError ? 'ctx-toast--error' : 'ctx-toast--ok'">
      {{ toast }}
    </div>
  </Teleport>

  <!-- Context Menu -->
  <Teleport to="body">
    <div v-if="visible && file" ref="menuEl" class="ctx-menu" :style="{ left: adjustedX + 'px', top: adjustedY + 'px' }"
      @contextmenu.prevent>
      <div class="ctx-file-name">{{ file.path }}</div>

      <!-- Group 1: Stage / Discard / Stash -->
      <button class="ctx-item" @click="stageFile">
        <Plus :size="13" class="ctx-icon ctx-icon--green" />
        Stage
      </button>
      <button class="ctx-item ctx-item--danger" @click="discardChanges">
        <Trash2 :size="13" class="ctx-icon" />
        Discard Changes
      </button>
      <button class="ctx-item" @click="stashFile">
        <Archive :size="13" class="ctx-icon" />
        Stash File
      </button>

      <div class="ctx-sep" />

      <!-- Group 2: History / Blame -->
      <button class="ctx-item" @click="fileHistory">
        <History :size="13" class="ctx-icon" />
        File History
      </button>
      <button class="ctx-item" @click="fileBlame">
        <GitBranch :size="13" class="ctx-icon" />
        File Blame
      </button>

      <div class="ctx-sep" />

      <!-- Group 3: Open actions -->
      <button class="ctx-item" @click="openExternalDiff">
        <GitCompare :size="13" class="ctx-icon" />
        Open in External Diff Tool
      </button>
      <button class="ctx-item" @click="openInVscode">
        <Code2 :size="13" class="ctx-icon" />
        Open in VS Code
      </button>
      <button class="ctx-item" @click="openInDefaultProgram">
        <ExternalLink :size="13" class="ctx-icon" />
        Open File in Default Program
      </button>
      <button class="ctx-item" @click="showInFolder">
        <FolderOpen :size="13" class="ctx-icon" />
        Show in Folder
      </button>

      <div class="ctx-sep" />

      <!-- Group 4: Copy / Patch -->
      <button class="ctx-item" @click="copyFilePath">
        <Copy :size="13" class="ctx-icon" />
        Copy File Path
      </button>
      <button class="ctx-item" @click="createPatch">
        <FileCode2 :size="13" class="ctx-icon" />
        Create Patch from File Changes
      </button>

      <div class="ctx-sep" />

      <!-- Group 5: Edit / Delete -->
      <button class="ctx-item" @click="editFile">
        <Pencil :size="13" class="ctx-icon" />
        Edit File
      </button>
      <button class="ctx-item ctx-item--danger" @click="deleteFile">
        <X :size="13" class="ctx-icon" />
        Delete File
      </button>
    </div>
  </Teleport>

  <!-- File History Modal -->
  <Teleport to="body">
    <div v-if="showHistoryModal" class="ctx-modal-overlay" @click.self="showHistoryModal = false">
      <div class="ctx-modal">
        <div class="ctx-modal-header">
          <History :size="15" class="ctx-icon" />
          <span class="ctx-modal-title">File History — {{ file?.path }}</span>
          <button class="ctx-modal-close" @click="showHistoryModal = false">✕</button>
        </div>
        <div class="ctx-modal-body">
          <div v-if="historyLoading" class="ctx-loading">Loading history…</div>
          <div v-else-if="historyEntries.length === 0" class="ctx-empty">No history found.</div>
          <div v-else class="ctx-history-list">
            <div v-for="entry in historyEntries" :key="entry.id" class="ctx-history-row">
              <span class="ctx-commit-id">{{ entry.id.slice(0, 7) }}</span>
              <span class="ctx-commit-msg">{{ entry.message }}</span>
              <span class="ctx-commit-meta">{{ entry.author }} · {{ formatDate(entry.time) }}</span>
            </div>
          </div>
        </div>
      </div>
    </div>
  </Teleport>

  <!-- File Blame Modal -->
  <Teleport to="body">
    <div v-if="showBlameModal" class="ctx-modal-overlay" @click.self="showBlameModal = false">
      <div class="ctx-modal ctx-modal--wide">
        <div class="ctx-modal-header">
          <GitBranch :size="15" class="ctx-icon" />
          <span class="ctx-modal-title">File Blame — {{ file?.path }}</span>
          <button class="ctx-modal-close" @click="showBlameModal = false">✕</button>
        </div>
        <div class="ctx-modal-body">
          <div v-if="blameLoading" class="ctx-loading">Loading blame…</div>
          <div v-else-if="blameLines.length === 0" class="ctx-empty">No blame data found.</div>
          <div v-else class="ctx-blame-list">
            <div v-for="line in blameLines" :key="line.line_no" class="ctx-blame-row">
              <span class="ctx-blame-no">{{ line.line_no }}</span>
              <span class="ctx-blame-author">{{ line.author }}</span>
              <span class="ctx-blame-date">{{ formatDate(line.time) }}</span>
              <span class="ctx-blame-content">{{ line.content }}</span>
            </div>
          </div>
        </div>
      </div>
    </div>
  </Teleport>

  <!-- Patch Modal -->
  <Teleport to="body">
    <div v-if="showPatchModal" class="ctx-modal-overlay" @click.self="showPatchModal = false">
      <div class="ctx-modal ctx-modal--wide">
        <div class="ctx-modal-header">
          <FileCode2 :size="15" class="ctx-icon" />
          <span class="ctx-modal-title">Patch — {{ file?.path }}</span>
          <div style="display:flex;gap:6px;margin-left:auto">
            <button class="ctx-modal-action" @click="savePatchToFile">Save .patch</button>
            <button class="ctx-modal-close" @click="showPatchModal = false">✕</button>
          </div>
        </div>
        <div class="ctx-modal-body">
          <div v-if="patchLoading" class="ctx-loading">Generating patch…</div>
          <pre v-else class="ctx-patch-pre">{{ patchContent || 'No changes to patch.' }}</pre>
        </div>
      </div>
    </div>
  </Teleport>
</template>

<style scoped src="../../styles/organisms/FileContextMenu.css"></style>
