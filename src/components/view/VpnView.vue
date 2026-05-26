<script setup>
import { ref, computed, onMounted } from "vue";
import { invoke } from "@tauri-apps/api/core";

const subUrl = ref("");
const subscriptions = ref([]);
const activeNode = ref(null);
const proxyEnabled = ref(false);
const proxyAddr = ref("127.0.0.1:7890");
const loading = ref(false);
const msg = ref("");
const coreRunning = ref(false);
const showModal = ref(false);

async function loadState() {
  try {
    subscriptions.value = await invoke("get_subscriptions");
    activeNode.value = await invoke("get_active_node");
    const [enabled, addr] = await invoke("get_proxy_state");
    proxyEnabled.value = enabled;
    proxyAddr.value = addr;
    coreRunning.value = await invoke("core_status");
  } catch (e) {
    console.error(e);
  }
}

onMounted(loadState);

async function toggleVpn() {
  if (coreRunning.value) {
    await stopCore();
  } else {
    await startCore();
  }
}

async function startCore() {
  if (!activeNode.value) { msg.value = "请先选择一个节点"; return; }
  loading.value = true; msg.value = "";
  try {
    await invoke("start_core");
    await new Promise(r => setTimeout(r, 800));
    coreRunning.value = await invoke("core_status");
    proxyAlive.value = await invoke("test_proxy");
    if (coreRunning.value && proxyAlive.value) {
      proxyEnabled.value = true;
      msg.value = "已连接";
    } else if (coreRunning.value && !proxyAlive.value) {
      msg.value = "核心已启动但端口未响应";
    } else {
      msg.value = "启动异常，请重试";
    }
  } catch (e) {
    msg.value = `启动失败: ${e}`;
  } finally {
    loading.value = false;
  }
}

async function stopCore() {
  loading.value = true;
  try {
    await invoke("stop_core");
    coreRunning.value = false;
    proxyEnabled.value = false;
    proxyAlive.value = false;
    msg.value = "已断开";
  } catch (e) {
    msg.value = `停止失败: ${e}`;
  } finally {
    loading.value = false;
  }
}

async function addSub() {
  const url = subUrl.value.trim();
  if (!url) return;
  loading.value = true;
  msg.value = "";
  try {
    subscriptions.value = await invoke("add_subscription", { url });
    subUrl.value = "";
    msg.value = `添加成功`;
  } catch (e) {
    msg.value = `添加失败: ${e}`;
  } finally {
    loading.value = false;
  }
}

async function removeSub(index) {
  try {
    subscriptions.value = await invoke("remove_subscription", { index });
    activeNode.value = await invoke("get_active_node");
    msg.value = "已删除";
  } catch (e) {
    msg.value = `删除失败: ${e}`;
  }
}

async function selectNode(node) {
  try {
    await invoke("select_node", { nodeRaw: node.raw });
    activeNode.value = node;
  } catch (e) {
    msg.value = `选择失败: ${e}`;
  }
}

const proxyAlive = ref(false);

const pasteName = ref("");
const pasteContent = ref("");
const showPaste = ref(false);

async function importContent() {
  const content = pasteContent.value.trim();
  if (!content) return;
  loading.value = true;
  msg.value = "";
  try {
    const name = pasteName.value.trim() || `手动导入 ${new Date().toLocaleDateString()}`;
    subscriptions.value = await invoke("import_subscription_content", { name, content });
    pasteContent.value = "";
    pasteName.value = "";
    showPaste.value = false;
    msg.value = "导入成功";
  } catch (e) {
    msg.value = `导入失败: ${e}`;
  } finally {
    loading.value = false;
  }
}

const allNodes = computed(() => {
  const nodes = [];
  for (const sub of subscriptions.value) {
    for (const node of sub.nodes) {
      nodes.push({ ...node, _subName: sub.name, _subUrl: sub.url });
    }
  }
  return nodes;
});
</script>

<template>
  <div class="vpn-panel">
    <div class="vpn-header">
      <h2>VPN</h2>
      <div class="header-right">
        <button class="add-btn" @click="showModal = true" title="管理订阅">
          <svg width="14" height="14" viewBox="0 0 14 14" fill="none" stroke="currentColor" stroke-width="1.6" stroke-linecap="round">
            <line x1="7" y1="2" x2="7" y2="12"/>
            <line x1="2" y1="7" x2="12" y2="7"/>
          </svg>
        </button>
        <button
          class="toggle-btn"
          :class="{ on: coreRunning, loading: loading }"
          :disabled="loading || !activeNode"
          @click="toggleVpn"
        >
          <span class="toggle-dot"></span>
        </button>
      </div>
    </div>

    <!-- Message -->
    <div class="vpn-msg" v-if="msg">{{ msg }}</div>

    <!-- Node list -->
    <div class="vpn-section nodes-section" v-if="allNodes.length">
      <div class="section-title">节点 ({{ allNodes.length }})</div>
      <div class="node-list">
        <div
          v-for="node in allNodes"
          :key="node.raw"
          class="node-item"
          :class="{ active: activeNode?.raw === node.raw }"
          @click="selectNode(node)"
        >
          <div class="node-left">
            <span class="node-type">{{ node.type.toUpperCase() }}</span>
            <span class="node-name">{{ node.name }}</span>
          </div>
          <span class="node-sub">{{ node._subName }}</span>
        </div>
      </div>
    </div>

    <!-- Modal overlay -->
    <div class="modal-overlay" v-if="showModal" @click.self="showModal = false">
      <div class="modal-box">
        <div class="modal-header">
          <span class="modal-title">订阅管理</span>
          <button class="modal-close" @click="showModal = false">&times;</button>
        </div>

        <div class="sub-form">
          <input
            v-model="subUrl"
            type="text"
            class="sub-input"
            placeholder="输入订阅链接..."
            @keyup.enter="addSub"
          />
          <button class="btn btn-purple" :disabled="loading || !subUrl.trim()" @click="addSub">
            {{ loading ? "..." : "添加" }}
          </button>
          <button class="btn btn-text" @click="showPaste = !showPaste">
            {{ showPaste ? '收起' : '手动导入' }}
          </button>
        </div>

        <div class="paste-area" v-if="showPaste">
          <input
            v-model="pasteName"
            type="text"
            class="sub-input"
            placeholder="订阅名称（可选）"
            style="margin-bottom: 6px;"
          />
          <textarea
            v-model="pasteContent"
            class="paste-input"
            placeholder="粘贴订阅内容..."
            rows="5"
          ></textarea>
          <button class="btn btn-purple" :disabled="loading || !pasteContent.trim()" @click="importContent" style="margin-top: 6px;">
            导入
          </button>
        </div>

        <div class="sub-list" v-if="subscriptions.length">
          <div
            v-for="(sub, i) in subscriptions"
            :key="sub.url"
            class="sub-row"
          >
            <span class="sub-name" :title="sub.url">{{ sub.name }}</span>
            <span class="sub-count">{{ sub.nodes.length }} 节点</span>
            <button class="btn-delete" @click="removeSub(i)">×</button>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>

<style>
.vpn-panel {
  width: 100%;
  height: 100%;
  display: flex;
  flex-direction: column;
  overflow: hidden;
}

.vpn-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  margin-bottom: 16px;
}
.vpn-header h2 {
  font-size: 20px;
  font-weight: 600;
}
.header-right {
  display: flex;
  align-items: center;
  gap: 10px;
  flex-shrink: 0;
}

/* Add button */
.add-btn {
  width: 32px;
  height: 32px;
  display: flex;
  align-items: center;
  justify-content: center;
  border: 1px solid rgba(255, 255, 255, 0.12);
  border-radius: 50%;
  background: rgba(255, 255, 255, 0.04);
  color: rgba(255, 255, 255, 0.45);
  cursor: pointer;
  transition: all 0.2s;
  flex-shrink: 0;
}
.add-btn:hover {
  background: rgba(255, 255, 255, 0.1);
  color: rgba(255, 255, 255, 0.9);
  border-color: rgba(255, 255, 255, 0.25);
}

/* Toggle switch */
.toggle-btn {
  width: 40px;
  height: 22px;
  border-radius: 11px;
  border: none;
  cursor: pointer;
  background: rgba(255, 255, 255, 0.12);
  position: relative;
  transition: background 0.2s;
  flex-shrink: 0;
  padding: 0;
}
.toggle-btn.on {
  background: rgb(140, 94, 212);
}
.toggle-btn:disabled {
  opacity: 0.4;
  cursor: default;
}
.toggle-btn.loading {
  animation: pulse 0.8s ease-in-out infinite;
}
.toggle-dot {
  position: absolute;
  top: 3px;
  left: 3px;
  width: 16px;
  height: 16px;
  border-radius: 50%;
  background: #fff;
  transition: transform 0.2s;
}
.toggle-btn.on .toggle-dot {
  transform: translateX(18px);
}
@keyframes pulse {
  0%, 100% { opacity: 0.4; }
  50% { opacity: 0.8; }
}

/* Modal */
.modal-overlay {
  position: fixed;
  inset: 0;
  background: rgba(0, 0, 0, 0.6);
  display: flex;
  align-items: center;
  justify-content: center;
  z-index: 100;
}
.modal-box {
  width: 420px;
  max-height: 80vh;
  background: rgb(30, 30, 30);
  border: 1px solid rgba(255, 255, 255, 0.08);
  border-radius: 12px;
  padding: 20px;
  display: flex;
  flex-direction: column;
  overflow: hidden;
}
.modal-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  margin-bottom: 16px;
}
.modal-title {
  font-size: 15px;
  font-weight: 600;
}
.modal-close {
  width: 28px;
  height: 28px;
  border: none;
  border-radius: 6px;
  background: transparent;
  color: rgba(255, 255, 255, 0.4);
  font-size: 18px;
  cursor: pointer;
  display: flex;
  align-items: center;
  justify-content: center;
}
.modal-close:hover {
  background: rgba(255, 255, 255, 0.08);
  color: #fff;
}

.vpn-section {
  margin-bottom: 16px;
  flex-shrink: 0;
}
.nodes-section {
  flex: 1;
  min-height: 0;
  display: flex;
  flex-direction: column;
}

.section-title {
  font-size: 12px;
  font-weight: 600;
  text-transform: uppercase;
  color: rgba(255, 255, 255, 0.4);
  margin-bottom: 8px;
  letter-spacing: 0.5px;
}

.sub-form {
  display: flex;
  gap: 8px;
}
.sub-input {
  flex: 1;
  padding: 8px 12px;
  border: 1px solid rgba(255, 255, 255, 0.1);
  border-radius: 6px;
  background: rgba(255, 255, 255, 0.06);
  color: #fff;
  font-size: 13px;
  outline: none;
}
.sub-input::placeholder {
  color: rgba(255, 255, 255, 0.3);
}

.btn {
  padding: 8px 16px;
  border: none;
  border-radius: 6px;
  font-size: 13px;
  cursor: pointer;
  transition: background 0.15s;
  white-space: nowrap;
}
.btn:disabled {
  opacity: 0.4;
  cursor: default;
}
.btn-purple {
  background: rgb(140, 94, 212);
  color: #fff;
}
.btn-purple:hover:not(:disabled) {
  background: rgb(120, 80, 190);
}
.btn-text {
  background: rgba(255, 255, 255, 0.06);
  color: rgba(255, 255, 255, 0.6);
}
.btn-text:hover {
  background: rgba(255, 255, 255, 0.12);
  color: #fff;
}

.paste-area {
  margin-top: 8px;
}
.paste-input {
  width: 100%;
  padding: 8px 12px;
  border: 1px solid rgba(255, 255, 255, 0.1);
  border-radius: 6px;
  background: rgba(255, 255, 255, 0.06);
  color: #fff;
  font-size: 12px;
  outline: none;
  resize: vertical;
  font-family: monospace;
  line-height: 1.4;
}
.paste-input::placeholder {
  color: rgba(255, 255, 255, 0.25);
}
.sub-list {
  margin-top: 12px;
  display: flex;
  flex-wrap: wrap;
  gap: 6px;
  overflow-y: auto;
}
.sub-row {
  display: flex;
  align-items: center;
  gap: 6px;
  padding: 4px 10px;
  border-radius: 6px;
  background: rgba(255, 255, 255, 0.06);
  font-size: 12px;
}
.sub-name {
  max-width: 120px;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
  color: rgba(255, 255, 255, 0.7);
}
.sub-count {
  color: rgba(255, 255, 255, 0.35);
}
.btn-delete {
  width: 18px;
  height: 18px;
  border: none;
  border-radius: 4px;
  background: transparent;
  color: rgba(255, 255, 255, 0.35);
  cursor: pointer;
  font-size: 14px;
  line-height: 1;
  display: flex;
  align-items: center;
  justify-content: center;
}
.btn-delete:hover {
  background: rgba(255, 80, 80, 0.3);
  color: #ff6b6b;
}

.node-list {
  flex: 1;
  min-height: 0;
  overflow-y: auto;
  display: flex;
  flex-direction: column;
  gap: 4px;
}
.node-item {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 10px 12px;
  border-radius: 6px;
  cursor: pointer;
  transition: background 0.12s;
  gap: 8px;
}
.node-item:hover {
  background: rgba(255, 255, 255, 0.06);
}
.node-item.active {
  background: rgb(140, 94, 212);
}
.node-left {
  display: flex;
  align-items: center;
  gap: 8px;
  min-width: 0;
  flex: 1;
}
.node-type {
  font-size: 10px;
  font-weight: 700;
  padding: 2px 5px;
  border-radius: 4px;
  background: rgba(255, 255, 255, 0.08);
  color: rgba(255, 255, 255, 0.5);
  flex-shrink: 0;
}
.node-item.active .node-type {
  background: rgb(140, 94, 212);
  color: rgb(180, 140, 240);
}
.node-name {
  font-size: 13px;
  color: rgba(255, 255, 255, 0.75);
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}
.node-sub {
  font-size: 11px;
  color: rgba(255, 255, 255, 0.3);
  flex-shrink: 0;
}

.vpn-msg {
  margin-bottom: 12px;
  padding: 6px 12px;
  border-radius: 6px;
  background: rgb(140, 94, 212);
  color: rgba(255, 255, 255, 0.6);
  font-size: 12px;
}

.node-list::-webkit-scrollbar {
  width: 4px;
}
.node-list::-webkit-scrollbar-thumb {
  background: rgba(255, 255, 255, 0.1);
  border-radius: 2px;
}
</style>
