<script setup lang="ts">
import { ref, onMounted, watch } from 'vue';
import { GitCommit, Calendar, GitBranch, Copy, Check } from '@lucide/vue';
import BaseBadge from '../atoms/BaseBadge.vue';
import BaseButton from '../atoms/BaseButton.vue';

export interface GraphNode {
  id: string;
  short_id: string;
  author: string;
  message: string;
  time: number;
  parents: string[];
  branches: string[];
  is_head: boolean;
  column: number;
  commit_branch?: string;
}

const props = defineProps<{
  nodes: GraphNode[];
  hasChanges?: boolean;
  modifiedCount?: number;
  addedCount?: number;
  currentBranch?: string;
}>();

const emit = defineEmits<{
  (e: 'selectWip'): void;
  (e: 'selectCommit', node: GraphNode): void;
  (e: 'checkoutBranch', branchName: string): void;
}>();

const selectedNode = ref<GraphNode | null>(null);
const hoveredNode = ref<GraphNode | null>(null);
const isWipSelected = ref<boolean>(false);
const canvasRef = ref<HTMLCanvasElement | null>(null);
const copied = ref(false);

const COLORS = ['#00d2d3', '#9c88ff', '#2ed573', '#ffa502', '#ff4757'];

function getBranchName(node: GraphNode): string {
  if (node.branches && node.branches.length > 0) {
    return node.branches[0];
  }
  if (node.commit_branch && node.commit_branch !== 'HEAD') {
    return node.commit_branch;
  }
  return props.currentBranch || 'master';
}

function handleDblClickNode(node: GraphNode) {
  const branchName = getBranchName(node);
  if (branchName) {
    emit('checkoutBranch', branchName);
  }
}

function getAuthorColor(author: string) {
  let hash = 0;
  for (let i = 0; i < author.length; i++) {
    hash = author.charCodeAt(i) + ((hash << 5) - hash);
  }
  const colors = ['#e84118', '#00a8ff', '#9c88ff', '#4cd137', '#fbc531', '#487eb0', '#10ac84'];
  return colors[Math.abs(hash) % colors.length];
}

function getAuthorInitials(author: string) {
  if (!author) return '??';
  const parts = author.trim().split(' ');
  if (parts.length >= 2) {
    return (parts[0][0] + parts[1][0]).toUpperCase();
  }
  return author.substring(0, 2).toUpperCase();
}

function drawGraph() {
  const canvas = canvasRef.value;
  if (!canvas) return;
  const ctx = canvas.getContext('2d');
  if (!ctx) return;

  const nodeHeight = 36;
  const colWidth = 18;
  const startX = 16;
  const startY = 18;

  canvas.width = 120;
  canvas.height = Math.max(props.nodes.length * nodeHeight + 40, 300);

  ctx.clearRect(0, 0, canvas.width, canvas.height);

  // 1. Draw continuous vertical branch lane lines
  const columnTails: Record<number, number> = {};
  props.nodes.forEach((node, i) => {
    const col = node.column;
    const y = startY + (i + (props.hasChanges ? 1 : 0)) * nodeHeight;
    if (columnTails[col] !== undefined) {
      const prevY = columnTails[col];
      ctx.beginPath();
      ctx.moveTo(startX + col * colWidth, prevY);
      ctx.lineTo(startX + col * colWidth, y);
      ctx.strokeStyle = COLORS[col % COLORS.length];
      ctx.lineWidth = 2;
      ctx.stroke();
    }
    columnTails[col] = y;
  });

  // 2. Draw curved connections between parent dots and child dots across columns
  props.nodes.forEach((node, i) => {
    const x1 = startX + node.column * colWidth;
    const y1 = startY + (i + (props.hasChanges ? 1 : 0)) * nodeHeight;

    node.parents.forEach((parentId) => {
      const parentIdx = props.nodes.findIndex((n) => n.id === parentId);
      if (parentIdx !== -1) {
        const parentNode = props.nodes[parentIdx];
        if (parentNode.column !== node.column) {
          const x2 = startX + parentNode.column * colWidth;
          const y2 = startY + (parentIdx + (props.hasChanges ? 1 : 0)) * nodeHeight;

          ctx.beginPath();
          ctx.moveTo(x1, y1);
          ctx.bezierCurveTo(x1, y1 + 18, x2, y2 - 18, x2, y2);
          ctx.strokeStyle = COLORS[node.column % COLORS.length];
          ctx.lineWidth = 2.5;
          ctx.stroke();
        }
      }
    });
  });

  // 3. Draw WIP connecting line & node if changes exist
  if (props.hasChanges && props.nodes.length > 0) {
    const headNode = props.nodes[0];
    const x1 = startX + headNode.column * colWidth;
    const y1 = startY;
    const y2 = startY + nodeHeight;

    ctx.beginPath();
    ctx.moveTo(x1, y1);
    ctx.lineTo(x1, y2);
    ctx.strokeStyle = '#00d2d3';
    ctx.lineWidth = 2;
    ctx.setLineDash([3, 3]);
    ctx.stroke();
    ctx.setLineDash([]);

    // Draw WIP node dashed ring
    ctx.beginPath();
    ctx.arc(x1, y1, 7, 0, 2 * Math.PI);
    ctx.strokeStyle = '#00d2d3';
    ctx.lineWidth = 2;
    ctx.setLineDash([3, 3]);
    ctx.stroke();
    ctx.setLineDash([]);

    // Inner WIP dot
    ctx.beginPath();
    ctx.arc(x1, y1, 3.5, 0, 2 * Math.PI);
    ctx.fillStyle = '#00d2d3';
    ctx.fill();
  }

  // 4. Draw graph node circles
  props.nodes.forEach((node, i) => {
    const x = startX + node.column * colWidth;
    const y = startY + (i + (props.hasChanges ? 1 : 0)) * nodeHeight;

    const isSelected = selectedNode.value?.id === node.id;
    const isHovered = hoveredNode.value?.id === node.id;

    ctx.beginPath();
    ctx.arc(x, y, isSelected || isHovered ? 6 : 4.5, 0, 2 * Math.PI);
    ctx.fillStyle = COLORS[node.column % COLORS.length];
    ctx.fill();

    ctx.lineWidth = node.is_head || isSelected ? 2 : 1;
    ctx.strokeStyle = node.is_head ? '#00d2d3' : isSelected ? '#ffffff' : '#181a1f';
    ctx.stroke();
  });
}

function selectCommit(node: GraphNode) {
  isWipSelected.value = false;
  selectedNode.value = node;
  emit('selectCommit', node);
}

function selectWip() {
  isWipSelected.value = true;
  selectedNode.value = null;
  emit('selectWip');
}

function formatDate(ts: number) {
  if (!ts) return '';
  return new Date(ts * 1000).toLocaleString();
}

function copyHash(hash: string) {
  navigator.clipboard.writeText(hash);
  copied.value = true;
  setTimeout(() => (copied.value = false), 2000);
}

onMounted(() => {
  drawGraph();
  if (props.hasChanges) {
    selectWip();
  } else if (props.nodes.length > 0) {
    selectCommit(props.nodes[0]);
  }
});

watch([() => props.nodes, () => props.hasChanges], () => {
  drawGraph();
  if (props.hasChanges && isWipSelected.value) {
    selectWip();
  } else if (props.nodes.length > 0 && !selectedNode.value) {
    selectCommit(props.nodes[0]);
  }
});
</script>

<template>
  <div class="graph-table-layout">
    <!-- Center Main Graph Table -->
    <div class="graph-table-container gk-panel">
      <!-- Table Header Bar -->
      <div class="table-header">
        <div class="col col-branch">BRANCH / TAG</div>
        <div class="col col-graph">GRAPH</div>
        <div class="col col-message">COMMIT MESSAGE</div>
        <div class="col col-author">AUTHOR</div>
        <div class="col col-hash">HASH</div>
      </div>

      <div class="table-body">
        <div class="canvas-layer">
          <canvas ref="canvasRef"></canvas>
        </div>

        <div class="rows-layer">
          <!-- Clickable WIP Row -->
          <div v-if="hasChanges" class="graph-row wip-row" :class="{ active: isWipSelected }" @click="selectWip">
            <div class="col col-branch">
              <span class="badge badge-wip">// WIP</span>
            </div>
            <div class="col col-graph"></div>
            <div class="col col-message wip-msg-cell">
              <span class="wip-title">// Work In Progress</span>
              <span class="wip-stats-pill">
                ✎ {{ modifiedCount || 9 }} + {{ addedCount || 13 }}
              </span>
            </div>
            <div class="col col-author">
              <div class="author-badge" style="background: #00d2d3">
                <span class="initials">YOU</span>
              </div>
              <span class="author-name">You</span>
            </div>
            <div class="col col-hash">---</div>
          </div>

          <!-- Commit Rows with Hover & Selection Branch Display near Avatar -->
          <div v-for="node in nodes" :key="node.id" class="graph-row"
            :class="{ active: selectedNode?.id === node.id && !isWipSelected, 'is-head': node.is_head }"
            @click="selectCommit(node)" @dblclick="handleDblClickNode(node)" @mouseenter="hoveredNode = node"
            @mouseleave="hoveredNode = null">
            <div class="col col-branch">
              <BaseBadge v-for="b in node.branches" :key="b" variant="branch" class="clickable"
                @dblclick.stop="emit('checkoutBranch', b)" title="Double click to switch to branch">
                <GitBranch :size="10" /> {{ b }}
              </BaseBadge>
            </div>
            <div class="col col-graph"></div>
            <div class="col col-message">
              <span class="msg">{{ node.message }}</span>
              <!-- Interactive Hover Tooltip displaying Branch Name -->
              <div v-if="hoveredNode?.id === node.id" class="hover-tooltip">
                <div class="tooltip-branch">
                  <GitBranch :size="11" />
                  <span>Branch: <strong>{{ getBranchName(node) }}</strong></span>
                </div>
                <div class="tooltip-hash">Commit {{ node.short_id }} by {{ node.author }} (double click to checkout)
                </div>
              </div>
            </div>
            <div class="col col-author">
              <div class="author-badge" :style="{ background: getAuthorColor(node.author) }">
                <span class="initials">{{ getAuthorInitials(node.author) }}</span>
              </div>
              <span class="author-name" :title="node.author">{{ node.author }}</span>
              <span class="avatar-branch-badge clickable"
                :class="{ 'is-active': hoveredNode?.id === node.id || selectedNode?.id === node.id }"
                @dblclick.stop="handleDblClickNode(node)" title="Double click to switch to branch">
                <GitBranch :size="9" />
                <span>{{ getBranchName(node) }}</span>
              </span>
            </div>
            <div class="col col-hash">{{ node.short_id }}</div>
          </div>
        </div>
      </div>
    </div>

    <!-- Commit / WIP Inspector Drawer -->
    <div class="commit-inspector gk-panel" v-if="isWipSelected || selectedNode">
      <div class="inspector-header" v-if="isWipSelected">
        <GitCommit :size="16" class="icon-warning" />
        <span class="title">Work In Progress (WIP)</span>
      </div>
      <div class="inspector-header" v-else-if="selectedNode">
        <GitCommit :size="16" class="icon-primary" />
        <span class="title">Commit {{ selectedNode.short_id }}</span>
        <BaseButton variant="secondary" size="xs" @click="copyHash(selectedNode.id)">
          <Check v-if="copied" :size="12" class="icon-success" />
          <Copy v-else :size="12" />
        </BaseButton>
      </div>

      <div class="inspector-body" v-if="isWipSelected">
        <div class="msg-box wip-box">
          <strong>Uncommitted Changes</strong>
          <p>Clicking // WIP focuses your modified and staged files in the right panel.</p>
        </div>
      </div>

      <div class="inspector-body" v-else-if="selectedNode">
        <!-- Displayed Branch Name Badge -->
        <div class="inspector-branch-badge">
          <GitBranch :size="12" class="icon-primary" />
          <span>Branch: <strong>{{ selectedNode.commit_branch || selectedNode.branches[0] || 'master' }}</strong></span>
        </div>

        <div class="msg-box">{{ selectedNode.message }}</div>

        <div class="meta-row">
          <div class="author-badge" :style="{ background: getAuthorColor(selectedNode.author) }">
            <span class="initials">{{ getAuthorInitials(selectedNode.author) }}</span>
          </div>
          <span>{{ selectedNode.author }}</span>
        </div>

        <div class="meta-row">
          <Calendar :size="13" class="icon-muted" />
          <span>{{ formatDate(selectedNode.time) }}</span>
        </div>

        <div class="parents-row" v-if="selectedNode.parents.length">
          <span class="label">Parents:</span>
          <span v-for="p in selectedNode.parents" :key="p" class="parent-tag">
            {{ p.substring(0, 7) }}
          </span>
        </div>
      </div>
    </div>
  </div>
</template>

<style scoped src="../../styles/organisms/BranchGraph.css"></style>
