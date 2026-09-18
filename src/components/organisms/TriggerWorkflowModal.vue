<script setup lang="ts">
import { ref, watch } from 'vue';
import { invoke } from '@tauri-apps/api/core';
import { X, Play, Plus, Trash2, AlertCircle } from '@lucide/vue';
import BaseButton from '../atoms/BaseButton.vue';
import { GithubWorkflow } from '../../types/actions';

const props = defineProps<{
  repoPath: string;
  workflows: GithubWorkflow[];
  currentBranch?: string;
}>();

const emit = defineEmits<{
  (e: 'close'): void;
  (e: 'triggered'): void;
}>();

const selectedWorkflowId = ref<number | null>(
  props.workflows.length > 0 ? props.workflows[0].id : null
);
const refName = ref<string>(props.currentBranch || 'main');
const inputPairs = ref<Array<{ key: string; value: string }>>([
  { key: '', value: '' },
]);

const loading = ref(false);
const errorMsg = ref('');
const successMsg = ref('');

watch(
  () => props.workflows,
  (newVal) => {
    if (newVal.length > 0 && selectedWorkflowId.value === null) {
      selectedWorkflowId.value = newVal[0].id;
    }
  },
  { immediate: true }
);

function addInputPair() {
  inputPairs.value.push({ key: '', value: '' });
}

function removeInputPair(index: number) {
  inputPairs.value.splice(index, 1);
}

async function handleTrigger() {
  if (!selectedWorkflowId.value) {
    errorMsg.value = 'Please select a workflow to trigger.';
    return;
  }
  if (!refName.value.trim()) {
    errorMsg.value = 'Please enter a target branch or ref.';
    return;
  }

  loading.value = true;
  errorMsg.value = '';
  successMsg.value = '';

  const inputsObj: Record<string, string> = {};
  for (const pair of inputPairs.value) {
    if (pair.key.trim()) {
      inputsObj[pair.key.trim()] = pair.value.trim();
    }
  }

  const inputsParam = Object.keys(inputsObj).length > 0 ? inputsObj : null;

  try {
    const res: string = await invoke('trigger_workflow_dispatch', {
      repoPath: props.repoPath,
      workflowId: selectedWorkflowId.value,
      refName: refName.value.trim(),
      inputs: inputsParam,
    });
    successMsg.value = res;
    setTimeout(() => {
      emit('triggered');
      emit('close');
    }, 1200);
  } catch (err: any) {
    errorMsg.value = typeof err === 'string' ? err : err.message || 'Failed to trigger workflow';
  } finally {
    loading.value = false;
  }
}
</script>

<template>
  <div class="modal-backdrop" @click.self="emit('close')">
    <div class="modal-card">
      <div class="modal-header">
        <div class="header-title">
          <Play :size="16" class="icon-purple" />
          <span>Run Workflow Manually (`workflow_dispatch`)</span>
        </div>
        <button class="close-btn" @click="emit('close')">
          <X :size="16" />
        </button>
      </div>

      <div class="modal-body">
        <div v-if="errorMsg" class="alert alert-danger">
          <AlertCircle :size="14" />
          <span>{{ errorMsg }}</span>
        </div>

        <div v-if="successMsg" class="alert alert-success">
          <span>{{ successMsg }}</span>
        </div>

        <div class="form-group">
          <label class="form-label">Workflow</label>
          <select v-model="selectedWorkflowId" class="form-select">
            <option v-for="w in workflows" :key="w.id" :value="w.id">
              {{ w.name }} ({{ w.path }})
            </option>
          </select>
        </div>

        <div class="form-group">
          <label class="form-label">Target Branch / Ref</label>
          <input v-model="refName" type="text" placeholder="main, develop, or commit SHA..." class="form-input" />
        </div>

        <div class="form-group">
          <div class="flex-between">
            <label class="form-label">Custom Workflow Inputs (Optional)</label>
            <button type="button" class="btn-text" @click="addInputPair">
              <Plus :size="12" /> Add Parameter
            </button>
          </div>

          <div class="inputs-list">
            <div v-for="(pair, idx) in inputPairs" :key="idx" class="input-row">
              <input v-model="pair.key" placeholder="Key (e.g., environment)" class="form-input flex-1" />
              <input v-model="pair.value" placeholder="Value (e.g., production)" class="form-input flex-1" />
              <button class="icon-del-btn" title="Remove parameter" @click="removeInputPair(idx)">
                <Trash2 :size="13" />
              </button>
            </div>
          </div>
        </div>
      </div>

      <div class="modal-footer">
        <BaseButton variant="secondary" size="sm" @click="emit('close')">Cancel</BaseButton>
        <BaseButton variant="primary" size="sm" :disabled="loading || !selectedWorkflowId" @click="handleTrigger">
          <Play :size="13" v-if="!loading" />
          <span>{{ loading ? 'Dispatching...' : 'Run Workflow' }}</span>
        </BaseButton>
      </div>
    </div>
  </div>
</template>

<style scoped src="../../styles/organisms/TriggerWorkflowModal.css"></style>
