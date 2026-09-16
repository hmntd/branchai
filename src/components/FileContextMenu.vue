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

// ── State ──────────────────────────────────────────────────
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

// ── Position adjustment: runs every time the menu becomes visible ───────────
// We watch `visible` so that by the time we measure, the v-if div is in the DOM.
watch(
  () => props.visible,
  async (isVisible) => {
    if (!isVisible) return;
    // Set raw position first so the element renders somewhere before measuring
    adjustedX.value = props.x;
    adjustedY.value = props.y;
    await nextTick(); // wait for v-if to put the div in DOM
    if (!menuEl.value) return;
    const rect = menuEl.value.getBoundingClientRect();
    const vw = window.innerWidth;
    const vh = window.innerHeight;
    adjustedX.value = props.x + rect.width  > vw ? vw - rect.width  - 8 : props.x;
    adjustedY.value = props.y + rect.height > vh ? vh - rect.height - 8 : props.y;
  }
);

// ── Click-outside to close ─────────────────────────────────
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

// ── Helpers ────────────────────────────────────────────────
function close() {
  emit('close');
}

function fullPath(): string {
  return `${props.repoPath}/${props.file?.path}`;
}

// ── Actions ────────────────────────────────────────────────

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

// ── Modals ─────────────────────────────────────────────────
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
  <!-- ── Toast (global, shown even after menu closes) ── -->
  <Teleport to="body">
    <div v-if="toast" class="ctx-toast" :class="toastError ? 'ctx-toast--error' : 'ctx-toast--ok'">
      {{ toast }}
    </div>
  </Teleport>

  <!-- ── Context Menu ─────────────────────────────────── -->
  <Teleport to="body">
    <div
      v-if="visible && file"
      ref="menuEl"
      class="ctx-menu"
      :style="{ left: adjustedX + 'px', top: adjustedY + 'px' }"
      @contextmenu.prevent
    >
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

  <!-- ── File History Modal ───────────────────────────── -->
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

  <!-- ── File Blame Modal ─────────────────────────────── -->
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

  <!-- ── Patch Modal ──────────────────────────────────── -->
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

<style scoped>
/* ── Context Menu ──────────────────────────────────────── */
.ctx-menu {
  position: fixed;
  z-index: 9999;
  min-width: 240px;
  background: #1a1d24;
  border: 1px solid rgba(255, 255, 255, 0.1);
  border-radius: 8px;
  box-shadow: 0 8px 32px rgba(0, 0, 0, 0.7), 0 0 0 1px rgba(99, 102, 241, 0.15);
  padding: 6px 0;
  backdrop-filter: blur(12px);
  animation: ctx-fade-in 0.12s ease;
}

@keyframes ctx-fade-in {
  from { opacity: 0; transform: scale(0.96) translateY(-4px); }
  to   { opacity: 1; transform: scale(1) translateY(0); }
}

.ctx-file-name {
  font-size: 10px;
  font-family: var(--font-mono, monospace);
  color: rgba(255, 255, 255, 0.35);
  padding: 4px 14px 6px;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
  border-bottom: 1px solid rgba(255, 255, 255, 0.07);
  margin-bottom: 4px;
}

.ctx-item {
  display: flex;
  align-items: center;
  gap: 9px;
  width: 100%;
  padding: 6px 14px;
  background: transparent;
  border: none;
  color: rgba(255, 255, 255, 0.82);
  font-size: 12px;
  text-align: left;
  cursor: pointer;
  transition: background 0.1s, color 0.1s;
}

.ctx-item:hover {
  background: rgba(99, 102, 241, 0.18);
  color: #fff;
}

.ctx-item--danger {
  color: rgba(255, 85, 85, 0.85);
}

.ctx-item--danger:hover {
  background: rgba(255, 71, 87, 0.15);
  color: #ff5555;
}

.ctx-icon {
  flex-shrink: 0;
  color: rgba(255, 255, 255, 0.45);
}

.ctx-icon--green {
  color: #2ed573 !important;
}

.ctx-item--danger .ctx-icon {
  color: rgba(255, 85, 85, 0.7);
}

.ctx-sep {
  height: 1px;
  background: rgba(255, 255, 255, 0.07);
  margin: 4px 0;
}

/* ── Toast ─────────────────────────────────────────────── */
.ctx-toast {
  position: fixed;
  bottom: 24px;
  left: 50%;
  transform: translateX(-50%);
  z-index: 99999;
  padding: 8px 18px;
  border-radius: 6px;
  font-size: 12px;
  font-weight: 600;
  pointer-events: none;
  animation: ctx-toast-in 0.2s ease;
}

@keyframes ctx-toast-in {
  from { opacity: 0; transform: translateX(-50%) translateY(8px); }
  to   { opacity: 1; transform: translateX(-50%) translateY(0); }
}

.ctx-toast--ok {
  background: rgba(46, 213, 115, 0.9);
  color: #0d1117;
}

.ctx-toast--error {
  background: rgba(255, 71, 87, 0.9);
  color: #fff;
}

/* ── Modals ─────────────────────────────────────────────── */
.ctx-modal-overlay {
  position: fixed;
  inset: 0;
  background: rgba(0, 0, 0, 0.72);
  backdrop-filter: blur(4px);
  z-index: 10000;
  display: flex;
  align-items: center;
  justify-content: center;
}

.ctx-modal {
  width: 560px;
  max-height: 70vh;
  background: #1a1d24;
  border: 1px solid rgba(255, 255, 255, 0.1);
  border-radius: 10px;
  box-shadow: 0 16px 48px rgba(0, 0, 0, 0.8);
  display: flex;
  flex-direction: column;
  overflow: hidden;
}

.ctx-modal--wide {
  width: 820px;
  max-height: 75vh;
}

.ctx-modal-header {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 10px 14px;
  border-bottom: 1px solid rgba(255, 255, 255, 0.08);
  background: rgba(255, 255, 255, 0.03);
}

.ctx-modal-title {
  font-size: 12px;
  font-weight: 600;
  font-family: var(--font-mono, monospace);
  color: rgba(255, 255, 255, 0.85);
  flex: 1;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.ctx-modal-close {
  background: transparent;
  border: none;
  color: rgba(255, 255, 255, 0.45);
  font-size: 14px;
  cursor: pointer;
  padding: 2px 6px;
  border-radius: 4px;
  transition: background 0.15s, color 0.15s;
}

.ctx-modal-close:hover {
  background: rgba(255, 71, 87, 0.15);
  color: #ff5555;
}

.ctx-modal-action {
  background: rgba(99, 102, 241, 0.2);
  border: 1px solid rgba(99, 102, 241, 0.35);
  color: rgba(255, 255, 255, 0.8);
  font-size: 11px;
  font-weight: 600;
  border-radius: 4px;
  padding: 3px 10px;
  cursor: pointer;
  transition: background 0.15s;
}

.ctx-modal-action:hover {
  background: rgba(99, 102, 241, 0.4);
}

.ctx-modal-body {
  flex: 1;
  overflow: auto;
  padding: 12px 0;
}

.ctx-loading,
.ctx-empty {
  font-size: 12px;
  color: rgba(255, 255, 255, 0.35);
  padding: 20px;
  text-align: center;
  font-style: italic;
}

/* History */
.ctx-history-list {
  display: flex;
  flex-direction: column;
}

.ctx-history-row {
  display: flex;
  flex-direction: column;
  gap: 2px;
  padding: 8px 16px;
  border-bottom: 1px solid rgba(255, 255, 255, 0.05);
  transition: background 0.1s;
}

.ctx-history-row:hover {
  background: rgba(99, 102, 241, 0.08);
}

.ctx-commit-id {
  font-family: var(--font-mono, monospace);
  font-size: 11px;
  color: #6366f1;
}

.ctx-commit-msg {
  font-size: 12px;
  color: rgba(255, 255, 255, 0.85);
}

.ctx-commit-meta {
  font-size: 10px;
  color: rgba(255, 255, 255, 0.35);
}

/* Blame */
.ctx-blame-list {
  display: flex;
  flex-direction: column;
  font-family: var(--font-mono, monospace);
  font-size: 11px;
}

.ctx-blame-row {
  display: grid;
  grid-template-columns: 40px 120px 140px 1fr;
  gap: 8px;
  padding: 3px 16px;
  border-bottom: 1px solid rgba(255, 255, 255, 0.03);
  transition: background 0.1s;
  align-items: center;
}

.ctx-blame-row:hover {
  background: rgba(99, 102, 241, 0.08);
}

.ctx-blame-no {
  color: rgba(255, 255, 255, 0.25);
  text-align: right;
}

.ctx-blame-author {
  color: #6366f1;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.ctx-blame-date {
  color: rgba(255, 255, 255, 0.3);
  font-size: 10px;
}

.ctx-blame-content {
  color: rgba(255, 255, 255, 0.7);
  white-space: pre;
  overflow: hidden;
  text-overflow: ellipsis;
}

/* Patch */
.ctx-patch-pre {
  font-family: var(--font-mono, monospace);
  font-size: 11px;
  color: rgba(255, 255, 255, 0.75);
  padding: 0 16px;
  white-space: pre;
  overflow: auto;
  margin: 0;
  line-height: 1.6;
}
</style>
