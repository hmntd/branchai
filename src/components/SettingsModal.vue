<script setup lang="ts">
import { ref, onMounted } from 'vue';
import { invoke } from '@tauri-apps/api/core';
import { X, Key, Cpu, Sparkles, Check, AlertCircle, RefreshCw, MessageSquare, GripHorizontal, Eye, EyeOff } from '@lucide/vue';

export interface AppConfig {
  openai_key: string;
  claude_key: string;
  gemini_key: string;
  grok_key: string;
  active_provider: string;
  active_model: string;
  custom_prompt_commit: string;
  custom_prompt_review: string;
  custom_prompt_conflict: string;
  last_opened_repo?: string | null;
  recent_repos?: string[];
}

const emit = defineEmits<{
  (e: 'close'): void;
  (e: 'saved'): void;
}>();

const config = ref<AppConfig>({
  openai_key: '',
  claude_key: '',
  gemini_key: '',
  grok_key: '',
  active_provider: 'openai',
  active_model: 'gpt-4o',
  custom_prompt_commit: '',
  custom_prompt_review: '',
  custom_prompt_conflict: '',
});

const isTesting = ref(false);
const isSaving = ref(false);
const showKey = ref(false);
const testStatus = ref('');
const testSuccess = ref(false);

const activeTab = ref<'keys' | 'prompts'>('keys');

// Drag state for moving modal around viewport
const modalPos = ref({ x: 0, y: 0 });
const isDragging = ref(false);
let startMouseX = 0;
let startMouseY = 0;
let initialPosX = 0;
let initialPosY = 0;

function startDrag(e: MouseEvent) {
  // Prevent drag when clicking interactable controls inside header
  if ((e.target as HTMLElement).closest('button, input, select, textarea')) return;
  isDragging.value = true;
  startMouseX = e.clientX;
  startMouseY = e.clientY;
  initialPosX = modalPos.value.x;
  initialPosY = modalPos.value.y;

  window.addEventListener('mousemove', onDrag);
  window.addEventListener('mouseup', stopDrag);
}

function onDrag(e: MouseEvent) {
  if (!isDragging.value) return;
  modalPos.value = {
    x: initialPosX + (e.clientX - startMouseX),
    y: initialPosY + (e.clientY - startMouseY),
  };
}

function stopDrag() {
  isDragging.value = false;
  window.removeEventListener('mousemove', onDrag);
  window.removeEventListener('mouseup', stopDrag);
}

const PROVIDER_MODELS: Record<string, string[]> = {
  openai: ['gpt-4o', 'gpt-4o-mini', 'o3-mini'],
  claude: ['claude-3-5-sonnet-20241022', 'claude-3-5-haiku-20241022', 'claude-3-opus-20240229'],
  gemini: ['gemini-1.5-pro', 'gemini-1.5-flash', 'gemini-2.0-flash-exp'],
  grok: ['grok-2-latest', 'grok-beta'],
};

async function loadConfig() {
  try {
    const cfg: AppConfig = await invoke('get_config');
    config.value = cfg;
  } catch (e) {
    console.error(e);
  }
}

async function saveSettings() {
  isSaving.value = true;
  try {
    await invoke('save_config', { config: config.value });
    emit('saved');
    emit('close');
  } catch (e) {
    console.error(e);
  } finally {
    isSaving.value = false;
  }
}

async function testConnection() {
  isTesting.value = true;
  testStatus.value = '';
  testSuccess.value = false;

  let key = '';
  if (config.value.active_provider === 'openai') key = config.value.openai_key;
  if (config.value.active_provider === 'claude') key = config.value.claude_key;
  if (config.value.active_provider === 'gemini') key = config.value.gemini_key;
  if (config.value.active_provider === 'grok') key = config.value.grok_key;

  try {
    const res: string = await invoke('test_ai_connection', {
      provider: config.value.active_provider,
      apiKey: key,
      model: config.value.active_model,
    });
    testStatus.value = res;
    testSuccess.value = true;
  } catch (err: any) {
    testStatus.value = typeof err === 'string' ? err : err.message || 'Connection test failed.';
    testSuccess.value = false;
  } finally {
    isTesting.value = false;
  }
}

function onProviderChange() {
  const models = PROVIDER_MODELS[config.value.active_provider];
  if (models && models.length > 0) {
    config.value.active_model = models[0];
  }
}

onMounted(() => {
  loadConfig();
});
</script>

<template>
  <div class="modal-overlay" @click.self="emit('close')">
    <div
      class="modal-card glass-panel"
      :style="{ transform: `translate(${modalPos.x}px, ${modalPos.y}px)` }"
    >
      <!-- Header with drag handle -->
      <div class="modal-header" @mousedown="startDrag" :class="{ dragging: isDragging }">
        <div class="header-title">
          <GripHorizontal :size="18" class="drag-grip" title="Drag modal to move" />
          <Sparkles :size="20" class="icon-purple" />
          <span>BranchAI Settings</span>
        </div>
        <button class="close-btn" @click="emit('close')" title="Close Settings">
          <X :size="18" />
        </button>
      </div>

      <!-- Navigation Tabs -->
      <div class="modal-tabs">
        <button
          class="tab-btn"
          :class="{ active: activeTab === 'keys' }"
          @click="activeTab = 'keys'"
        >
          <Key :size="14" /> AI Keys & Providers
        </button>
        <button
          class="tab-btn"
          :class="{ active: activeTab === 'prompts' }"
          @click="activeTab = 'prompts'"
        >
          <MessageSquare :size="14" /> Prompt Templates
        </button>
      </div>

      <div class="modal-body">
        <!-- Tab 1: API Keys & Active Provider -->
        <div v-if="activeTab === 'keys'" class="tab-content">
          <div class="form-group">
            <label><Cpu :size="14" /> Active Provider</label>
            <select v-model="config.active_provider" @change="onProviderChange" class="select-input">
              <option value="openai">OpenAI</option>
              <option value="claude">Anthropic Claude</option>
              <option value="gemini">Google Gemini</option>
              <option value="grok">xAI Grok</option>
            </select>
          </div>

          <div class="form-group">
            <label>Model</label>
            <select v-model="config.active_model" class="select-input">
              <option
                v-for="m in (PROVIDER_MODELS[config.active_provider] || [])"
                :key="m"
                :value="m"
              >
                {{ m }}
              </option>
            </select>
          </div>

          <hr class="divider" />

          <!-- Provider API Keys: Only show selected active provider, retaining state when switching -->
          <div v-if="config.active_provider === 'openai'" class="form-group">
            <label><Key :size="14" /> OpenAI API Key</label>
            <div class="key-input-wrapper">
              <input :type="showKey ? 'text' : 'password'" v-model="config.openai_key" placeholder="sk-..." class="styled-input" />
              <button class="eye-btn" @click="showKey = !showKey" :title="showKey ? 'Hide key' : 'Show key'" type="button">
                <EyeOff v-if="showKey" :size="16" />
                <Eye v-else :size="16" />
              </button>
            </div>
          </div>

          <div v-if="config.active_provider === 'claude'" class="form-group">
            <label><Key :size="14" /> Anthropic Claude API Key</label>
            <div class="key-input-wrapper">
              <input :type="showKey ? 'text' : 'password'" v-model="config.claude_key" placeholder="sk-ant-..." class="styled-input" />
              <button class="eye-btn" @click="showKey = !showKey" :title="showKey ? 'Hide key' : 'Show key'" type="button">
                <EyeOff v-if="showKey" :size="16" />
                <Eye v-else :size="16" />
              </button>
            </div>
          </div>

          <div v-if="config.active_provider === 'gemini'" class="form-group">
            <label><Key :size="14" /> Google Gemini API Key</label>
            <div class="key-input-wrapper">
              <input :type="showKey ? 'text' : 'password'" v-model="config.gemini_key" placeholder="AIzaSy..." class="styled-input" />
              <button class="eye-btn" @click="showKey = !showKey" :title="showKey ? 'Hide key' : 'Show key'" type="button">
                <EyeOff v-if="showKey" :size="16" />
                <Eye v-else :size="16" />
              </button>
            </div>
          </div>

          <div v-if="config.active_provider === 'grok'" class="form-group">
            <label><Key :size="14" /> xAI Grok API Key</label>
            <div class="key-input-wrapper">
              <input :type="showKey ? 'text' : 'password'" v-model="config.grok_key" placeholder="xai-..." class="styled-input" />
              <button class="eye-btn" @click="showKey = !showKey" :title="showKey ? 'Hide key' : 'Show key'" type="button">
                <EyeOff v-if="showKey" :size="16" />
                <Eye v-else :size="16" />
              </button>
            </div>
          </div>

          <div class="test-row">
            <button @click="testConnection" class="btn btn-secondary btn-sm" :disabled="isTesting">
              <RefreshCw v-if="isTesting" :size="14" class="spinning" />
              <Sparkles v-else :size="14" />
              Test Active Connection
            </button>
            <div v-if="testStatus" class="test-result" :class="testSuccess ? 'text-success' : 'text-danger'">
              <Check v-if="testSuccess" :size="14" />
              <AlertCircle v-else :size="14" />
              <span>{{ testStatus }}</span>
            </div>
          </div>
        </div>

        <!-- Tab 2: Custom Prompt Templates -->
        <div v-if="activeTab === 'prompts'" class="tab-content">
          <div class="form-group">
            <label>Commit Message Prompt Template</label>
            <textarea v-model="config.custom_prompt_commit" rows="3" class="styled-textarea"></textarea>
          </div>

          <div class="form-group">
            <label>Code Review Prompt Template</label>
            <textarea v-model="config.custom_prompt_review" rows="3" class="styled-textarea"></textarea>
          </div>

          <div class="form-group">
            <label>Conflict Resolution Prompt Template</label>
            <textarea v-model="config.custom_prompt_conflict" rows="3" class="styled-textarea"></textarea>
          </div>
        </div>
      </div>

      <div class="modal-footer">
        <button class="btn btn-secondary" @click="emit('close')">Cancel</button>
        <button class="btn btn-primary" @click="saveSettings" :disabled="isSaving">
          <Check :size="14" /> Save Settings
        </button>
      </div>
    </div>
  </div>
</template>

<style scoped>
.modal-overlay {
  position: fixed;
  top: 0;
  left: 0;
  width: 100vw;
  height: 100vh;
  background: rgba(0, 0, 0, 0.7);
  backdrop-filter: blur(8px);
  z-index: 100;
  display: flex;
  align-items: center;
  justify-content: center;
}

.modal-card {
  width: 540px;
  min-width: 440px;
  max-width: 90vw;
  min-height: 480px;
  max-height: 85vh;
  display: flex;
  flex-direction: column;
  background: rgba(19, 24, 37, 0.96);
  border: 1px solid var(--border-color);
  box-shadow: 0 16px 50px rgba(0, 0, 0, 0.7);
  border-radius: 16px;
  overflow: auto;
  resize: both;
  position: relative;
  transition: box-shadow 0.2s ease;
}

.modal-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 16px 20px;
  border-bottom: 1px solid var(--border-color);
  cursor: grab;
  user-select: none;
  background: rgba(255, 255, 255, 0.02);
}

.modal-header.dragging {
  cursor: grabbing;
}

.header-title {
  display: flex;
  align-items: center;
  gap: 10px;
  font-weight: 700;
  font-size: 16px;
  color: var(--text-main);
}

.drag-grip {
  color: var(--text-muted);
  opacity: 0.6;
}

.close-btn {
  background: transparent !important;
  border: none !important;
  color: var(--text-muted);
  border-radius: 8px;
  padding: 6px;
  display: flex;
  align-items: center;
  justify-content: center;
  cursor: pointer;
  transition: all 0.2s ease;
}

.close-btn:hover {
  background: rgba(255, 255, 255, 0.1) !important;
  color: #ffffff;
}

.modal-tabs {
  display: flex;
  border-bottom: 1px solid var(--border-color);
  background: rgba(0, 0, 0, 0.2);
}

.tab-btn {
  flex: 1;
  display: flex;
  align-items: center;
  justify-content: center;
  gap: 8px;
  padding: 12px;
  background: transparent;
  border: none;
  color: var(--text-muted);
  font-size: 13px;
  font-weight: 500;
  cursor: pointer;
  border-bottom: 2px solid transparent;
  transition: all 0.2s;
}

.tab-btn:hover {
  color: var(--text-main);
}

.tab-btn.active {
  color: var(--primary);
  border-bottom-color: var(--primary);
  background: rgba(0, 210, 211, 0.08);
}

.modal-body {
  padding: 22px 24px;
  flex: 1;
  overflow-y: auto;
}

.tab-content {
  display: flex;
  flex-direction: column;
  gap: 16px;
}

.form-group {
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.form-group label {
  font-size: 13px;
  font-weight: 600;
  color: var(--text-muted);
  display: flex;
  align-items: center;
  gap: 6px;
}

/* Customized Dropdown Select controls matching modal palette */
.select-input {
  appearance: none;
  -webkit-appearance: none;
  -moz-appearance: none;
  width: 100%;
  background-color: rgba(255, 255, 255, 0.05);
  background-image: url("data:image/svg+xml,%3Csvg xmlns='http://www.w3.org/2000/svg' width='16' height='16' viewBox='0 0 24 24' fill='none' stroke='%239da5b4' stroke-width='2' stroke-linecap='round' stroke-linejoin='round'%3E%3Cpolyline points='6 9 12 15 18 9'%3E%3C/polyline%3E%3C/svg%3E");
  background-repeat: no-repeat;
  background-position: right 14px center;
  background-size: 16px;
  border: 1px solid rgba(255, 255, 255, 0.12);
  color: var(--text-main);
  border-radius: 8px;
  padding: 10px 38px 10px 14px;
  font-size: 13px;
  font-weight: 500;
  cursor: pointer;
  transition: all 0.2s ease;
  box-shadow: inset 0 1px 2px rgba(0, 0, 0, 0.2);
}

.select-input:hover {
  border-color: var(--primary);
  background-color: rgba(255, 255, 255, 0.08);
}

.select-input:focus {
  border-color: var(--primary);
  box-shadow: 0 0 0 3px var(--primary-glow);
  outline: none;
}

.select-input option {
  background-color: #192231;
  color: #e6e6e6;
  padding: 10px;
}

.styled-input, .styled-textarea {
  width: 100%;
  background: rgba(255, 255, 255, 0.04);
  border: 1px solid rgba(255, 255, 255, 0.12);
  color: var(--text-main);
  border-radius: 8px;
  padding: 10px 14px;
  font-size: 13px;
  outline: none;
  transition: all 0.2s ease;
}

.styled-input:focus, .styled-textarea:focus {
  border-color: var(--primary);
  box-shadow: 0 0 0 3px var(--primary-glow);
  background: rgba(255, 255, 255, 0.07);
}

/* API Key input with eye toggle */
.key-input-wrapper {
  position: relative;
  display: flex;
  align-items: center;
}

.key-input-wrapper .styled-input {
  padding-right: 44px;
}

.eye-btn {
  position: absolute;
  right: 10px;
  background: transparent;
  border: none;
  color: var(--text-muted);
  display: flex;
  align-items: center;
  justify-content: center;
  cursor: pointer;
  padding: 4px;
  border-radius: 4px;
  transition: color 0.2s ease, background 0.2s ease;
  flex-shrink: 0;
}

.eye-btn:hover {
  color: var(--primary);
  background: rgba(0, 210, 211, 0.12);
}

.divider {
  border: none;
  border-top: 1px solid var(--border-color);
  margin: 4px 0;
}

.test-row {
  display: flex;
  align-items: center;
  gap: 12px;
  margin-top: 6px;
}

.test-result {
  display: flex;
  align-items: center;
  gap: 6px;
  font-size: 12px;
}

.text-success {
  color: var(--success);
}
.text-danger {
  color: var(--danger);
}

.modal-footer {
  display: flex;
  justify-content: flex-end;
  gap: 10px;
  padding: 16px 24px;
  border-top: 1px solid var(--border-color);
  background: rgba(0, 0, 0, 0.2);
}

.icon-purple {
  color: #c084fc;
}

.spinning {
  animation: spin 1s linear infinite;
}

@keyframes spin {
  100% {
    transform: rotate(360deg);
  }
}
</style>
