<script setup lang="ts">
import { ChevronDown, ChevronRight, Folder, FolderOpen, Plus, Minus } from '@lucide/vue';
import TreeNodes from './TreeNodes.vue';

interface TreeNode {
  name: string;
  fullPath: string;
  isDir: boolean;
  children: TreeNode[];
  file?: {
    path: string;
    status: string;
    staged: boolean;
  };
}

const props = defineProps<{
  nodes: TreeNode[];
  staged: boolean;
  readOnly?: boolean;
  depth?: number;
  collapsedFolders: Record<string, boolean>;
  getIconClass: (status: string) => string;
  getPrefix: (status: string) => string;
}>();

const emit = defineEmits<{
  (e: 'toggleFolder', path: string): void;
  (e: 'inspect', file: { path: string; status: string; staged: boolean }): void;
  (e: 'stage', file: { path: string; status: string; staged: boolean }): void;
  (e: 'unstage', file: { path: string; status: string; staged: boolean }): void;
  (e: 'contextMenu', event: MouseEvent, file: { path: string; status: string; staged: boolean }): void;
}>();

const depth = props.depth ?? 0;

function isOpen(fullPath: string) {
  return !props.collapsedFolders[fullPath];
}

function indent(d: number) {
  return { paddingLeft: `${d * 12 + 4}px` };
}
</script>

<template>
  <template v-for="node in nodes" :key="node.fullPath">
    <!-- Folder row -->
    <div v-if="node.isDir" class="tree-folder-row" :style="indent(depth)" @click="emit('toggleFolder', node.fullPath)">
      <component :is="isOpen(node.fullPath) ? ChevronDown : ChevronRight" :size="10" class="tree-chevron" />
      <component :is="isOpen(node.fullPath) ? FolderOpen : Folder" :size="12" class="tree-folder-icon" />
      <span class="tree-name">{{ node.name }}</span>
    </div>

    <!-- Recursion into children -->
    <TreeNodes v-if="node.isDir && isOpen(node.fullPath)" :nodes="node.children" :staged="staged" :read-only="readOnly"
      :depth="depth + 1" :collapsed-folders="collapsedFolders" :get-icon-class="getIconClass" :get-prefix="getPrefix"
      @toggle-folder="(p) => emit('toggleFolder', p)" @inspect="(f) => emit('inspect', f)"
      @stage="(f) => emit('stage', f)" @unstage="(f) => emit('unstage', f)"
      @context-menu="(ev, f) => emit('contextMenu', ev, f)" />

    <!-- File row -->
    <div v-if="!node.isDir && node.file" class="file-row tree-file-row" :class="{ staged }" :style="indent(depth)"
      @click="emit('inspect', node.file)" @contextmenu.prevent="emit('contextMenu', $event, node.file)">
      <span class="status-prefix" :class="staged ? 'text-success' : getIconClass(node.file.status)">
        {{ staged ? '✓' : getPrefix(node.file.status) }}
      </span>
      <span class="file-path tree-file-name">{{ node.name }}</span>
      <button v-if="!readOnly && !staged" class="stage-btn" @click.stop="emit('stage', node.file)" title="Stage File">
        <Plus :size="11" />
      </button>
      <button v-else-if="!readOnly && staged" class="stage-btn" @click.stop="emit('unstage', node.file)"
        title="Unstage File">
        <Minus :size="11" />
      </button>
    </div>
  </template>
</template>

<style scoped src="../../styles/organisms/TreeNodes.css"></style>
