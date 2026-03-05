<!--
  广播客户端主界面

  功能：
  - 按住说话模式（空格键或按钮）
  - 编码格式选择（PCM/Opus）
  - 音量调节
  - 连接状态显示
  - 配置管理
  - 自动更新检测
-->
<template>
  <div class="app-container">
    <!-- Header -->
    <header class="app-header">
      <div class="header-left">
        <span class="app-logo">🎙️</span>
        <h1 class="app-name">语音广播</h1>
      </div>
      <div class="header-right">
        <el-button
          size="small"
          @click="showSettingsDialog"
        >
          <template #icon>
            <span>⚙️</span>
          </template>
          设置
        </el-button>
        <el-button
          v-if="updateAvailable"
          type="primary"
          size="small"
          @click="showUpdateDialog"
          :loading="isDownloading"
        >
          <template #icon>
            <span v-if="!isDownloading">🔄</span>
            <span v-else>⏳</span>
          </template>
          {{ isDownloading ? '下载中...' : `v${updateInfo.version}` }}
        </el-button>
        <el-button
          v-else
          size="small"
          @click="checkForUpdates"
        >
          <template #icon>
            <span>🔍</span>
          </template>
          检查更新
        </el-button>
      </div>
    </header>

    <!-- Main Content -->
    <main class="app-main">
      <div class="control-card">
        <!-- Status Bar -->
        <div class="status-bar" :class="`status-${connectionStatus}`">
          <div class="status-icon">
            <span v-if="connectionStatus === 'connected'">✅</span>
            <span v-else-if="connectionStatus === 'disconnected'">❌</span>
            <span v-else-if="connectionStatus === 'connecting'">⏳</span>
            <span v-else>⚪</span>
          </div>
          <div class="status-text">
            <div class="status-label">{{ getStatusLabel }}</div>
            <div class="status-message">{{ status }}</div>
          </div>
          <div v-if="connectionStatus === 'busy'" class="busy-info">
            <span class="busy-icon">🚫</span>
            <span class="busy-text">{{ formatTime(busyDuration) }}</span>
          </div>
        </div>

        <!-- Config Grid -->
        <div class="config-grid">
          <div class="config-item">
            <label class="config-label">编码格式</label>
            <el-select
              v-model="codec"
              size="default"
              :disabled="isBroadcasting || isConnecting"
              @change="saveCodecPreference"
            >
              <el-option label="PCM（最低延迟）" value="pcm" />
              <el-option label="Opus（省流量）" value="opus" />
            </el-select>
          </div>
          <div class="config-item">
            <label class="config-label">音量 {{ (volume * 100).toFixed(0) }}%</label>
            <el-slider
              v-model="volume"
              :min="0"
              :max="APP_CONFIG.MAX_VOLUME"
              :step="0.05"
              :disabled="isBroadcasting || isConnecting"
              @change="saveVolumePreference"
            />
          </div>
        </div>

        <!-- Broadcast Button -->
        <button
          class="broadcast-btn"
          :class="{ broadcasting: isBroadcasting }"
          :disabled="(connectionStatus === 'busy' || connectionStatus === 'connecting') && !isBroadcasting"
          @mousedown="handleMouseDown"
          @mouseup="handleMouseUp"
          @mouseleave="handleMouseUp"
          @touchstart.prevent="handleTouchStart"
          @touchend.prevent="handleTouchEnd"
        >
          <template v-if="connectionStatus === 'busy' && !isBroadcasting">
            <span class="btn-icon">🚫</span>
            <span>广播被占用</span>
          </template>
          <template v-else-if="connectionStatus === 'disconnected' && !isBroadcasting">
            <span class="btn-icon">🔄</span>
            <span>重试连接</span>
          </template>
          <template v-else-if="!isBroadcasting">
            <span class="btn-icon">🎙️</span>
            <span>按住说话</span>
          </template>
          <template v-else>
            <span class="btn-icon">🔴</span>
            <span>松开停止</span>
          </template>
        </button>

        <!-- Timer & Hints -->
        <div v-if="isBroadcasting" class="timer-section">
          <span class="timer-icon">⏱️</span>
          <span class="timer-text">{{ formatTime(broadcastDuration) }}</span>
        </div>
        <div v-else class="hints-section">
          按住按钮或空格键开始广播
        </div>
      </div>
    </main>

    <!-- Footer -->
    <footer class="app-footer">
      <span class="version">v{{ appVersion }}</span>
    </footer>

    <!-- Update Dialog -->
    <el-dialog
      v-model="updateDialogVisible"
      title="软件更新"
      width="500px"
      :close-on-click-modal="false"
    >
      <div v-if="updateAvailable">
        <div class="update-header">
          <div class="update-icon">🎉</div>
          <h3>发现新版本 {{ updateInfo.version }}</h3>
        </div>
        <div class="update-content">
          <div class="update-info">
            <div class="info-item">
              <span class="info-label">当前版本:</span>
              <span class="info-value">{{ appVersion }}</span>
            </div>
            <div class="info-item">
              <span class="info-label">新版本:</span>
              <span class="info-value">{{ updateInfo.version }}</span>
            </div>
            <div v-if="updateInfo.date" class="info-item">
              <span class="info-label">发布日期:</span>
              <span class="info-value">{{ formatDate(updateInfo.date) }}</span>
            </div>
          </div>
          <div class="update-notes">
            <h4>更新内容:</h4>
            <div class="notes-content" v-html="formatReleaseNotes(updateInfo.body)"></div>
          </div>
        </div>
      </div>
      <div v-else>
        <div class="no-update">
          <div class="no-update-icon">✅</div>
          <h3>已是最新版本</h3>
          <p>当前版本已经是最新版本，无需更新。</p>
        </div>
      </div>
      <template #footer>
        <span class="dialog-footer">
          <el-button @click="updateDialogVisible = false">取消</el-button>
          <el-button
            v-if="updateAvailable && !isInstalling"
            type="primary"
            @click="downloadAndUpdate"
            :loading="isDownloading"
          >
            {{ isDownloading ? '下载中...' : '立即更新' }}
          </el-button>
          <el-button
            v-else-if="isInstalling"
            type="success"
            disabled
          >
            已下载，请关闭应用以应用更新
          </el-button>
        </span>
      </template>
    </el-dialog>

    <!-- Settings Dialog -->
    <el-dialog
      v-model="settingsDialogVisible"
      title="设置"
      width="450px"
    >
      <el-form
        ref="settingsFormRef"
        :model="settingsForm"
        :rules="settingsFormRules"
        label-width="120px"
      >
        <el-form-item label="广播服务器地址" prop="serverUrl">
          <el-input
            v-model="settingsForm.serverUrl"
            placeholder="ws://localhost:8081/ws"
            clearable
          />
        </el-form-item>
        <el-form-item label="更新服务器地址" prop="updateServerBaseUrl">
          <el-input
            v-model="settingsForm.updateServerBaseUrl"
            placeholder="https://example.com/updates（留空使用默认）"
            clearable
          />
          <div class="form-item-hint" v-if="settingsForm.updateServerBaseUrl">
            程序会自动在地址后拼接 /平台/版本号，如: /windows-x86_64/0.1.0
          </div>
        </el-form-item>
        <el-form-item label="默认编码" prop="defaultCodec">
          <el-select v-model="settingsForm.defaultCodec" style="width: 100%">
            <el-option label="PCM（最低延迟）" value="pcm" />
            <el-option label="Opus（省流量）" value="opus" />
          </el-select>
        </el-form-item>
        <el-form-item label="默认音量" prop="defaultVolume">
          <el-slider
            v-model="settingsForm.defaultVolume"
            :min="0"
            :max="1.5"
            :step="0.05"
            :format-tooltip="(v: number) => `${(v * 100).toFixed(0)}%`"
          />
        </el-form-item>
      </el-form>
      <div class="config-path-info">
        <span class="info-label">配置文件路径:</span>
        <span class="info-value">{{ configFilePath || '加载中...' }}</span>
      </div>
      <template #footer>
        <span class="dialog-footer">
          <el-button @click="settingsDialogVisible = false">取消</el-button>
          <el-button type="primary" @click="saveSettings">保存</el-button>
        </span>
      </template>
    </el-dialog>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onBeforeUnmount, onMounted, watch, reactive } from 'vue';
import { ElMessage, ElNotification } from 'element-plus';
import { invoke } from '@tauri-apps/api/core';
import { listen } from '@tauri-apps/api/event';
import { getVersion } from '@tauri-apps/api/app';
import { useAppUpdate } from '@/utils/useAppUpdate';
import { APP_CONFIG } from '@/config';
import { readConfig, writeConfig, getConfigPath, type AppConfig } from '@/composables';
import type { CodecType } from '@/types/index';
import { createLogger } from '@/utils/logger';

const logger = createLogger('App');
const configLogger = createLogger('Config');
const broadcastLogger = createLogger('Broadcast');
const audioLogger = createLogger('AudioProcessing');
const eventLogger = createLogger('Event');

const codec = ref<CodecType>(APP_CONFIG.DEFAULT_CODEC);
const volume = ref(APP_CONFIG.DEFAULT_VOLUME);
const serverUrl = ref(APP_CONFIG.WS_URL);
const updateDialogVisible = ref(false);
const settingsDialogVisible = ref(false);
const configFilePath = ref('');
const settingsFormRef = ref();
const appVersion = ref('加载中...');

// 验证函数（必须在 settingsFormRules 之前定义）
// URL 验证函数
const isValidWebSocketUrl = (url: string): boolean => {
  try {
    // 必须是有效的 URL
    const parsed = new URL(url);

    // 只允许 ws:// 或 wss:// 协议
    if (!['ws:', 'wss:'].includes(parsed.protocol)) {
      return false;
    }

    // 必须有主机名
    if (!parsed.hostname || parsed.hostname.trim() === '') {
      return false;
    }

    // 拒绝 localhost 以外的本地地址（防止内网探测）
    const hostname = parsed.hostname.toLowerCase();
    const localPatterns = ['127.0.0.1', 'localhost', '[::1]'];
    if (!localPatterns.includes(hostname) &&
        (hostname === '0.0.0.0' || hostname.startsWith('192.168.') ||
         hostname.startsWith('10.') || hostname.startsWith('172.'))) {
      // 允许私有网络地址（本地部署场景）
      // 但可以记录警告
      console.warn(`连接到私有网络地址: ${hostname}`);
    }

    return true;
  } catch {
    return false;
  }
};

const isValidHttpsUrl = (url: string): boolean => {
  if (!url.trim()) return true; // 允许为空
  try {
    const parsed = new URL(url);
    return ['https:', 'http:'].includes(parsed.protocol) &&
           parsed.hostname.trim() !== '';
  } catch {
    return false;
  }
};

// 设置表单
const settingsForm = reactive<AppConfig>({
  serverUrl: 'ws://localhost:8081/ws',
  updateServerBaseUrl: '',
  defaultCodec: 'pcm',
  defaultVolume: 1.0,
});

// 表单验证规则
const settingsFormRules = {
  serverUrl: [
    { required: true, message: '广播服务器地址不能为空', trigger: 'blur' },
    {
      validator: (_rule: any, value: string, callback: any) => {
        if (!value || isValidWebSocketUrl(value)) {
          callback();
        } else {
          callback(new Error('请输入有效的 WebSocket 地址（ws:// 或 wss://）'));
        }
      },
      trigger: 'blur'
    }
  ],
  updateServerBaseUrl: [
    {
      validator: (_rule: any, value: string, callback: any) => {
        if (!value || isValidHttpsUrl(value)) {
          callback();
        } else {
          callback(new Error('请输入有效的 HTTP/HTTPS 地址'));
        }
      },
      trigger: 'blur'
    }
  ],
  defaultCodec: [
    { required: true, message: '请选择默认编码', trigger: 'change' },
    {
      validator: (_rule: any, value: string, callback: any) => {
        if (['pcm', 'opus'].includes(value)) {
          callback();
        } else {
          callback(new Error('编码格式必须是 PCM 或 Opus'));
        }
      },
      trigger: 'change'
    }
  ],
  defaultVolume: [
    {
      validator: (_rule: any, value: number, callback: any) => {
        if (value >= 0 && value <= 1.5) {
          callback();
        } else {
          callback(new Error('音量必须在 0% 到 150% 之间'));
        }
      },
      trigger: 'change'
    }
  ]
};

// Initialize update functionality
const {
  updateAvailable,
  updateInfo,
  isDownloading,
  isInstalling,
  checkForUpdates,
  downloadAndUpdate,
  showUpdateDialog,
} = useAppUpdate();

const isBroadcasting = ref(false);
const isConnecting = ref(false);
const isOperating = ref(false);
const status = ref('准备就绪');
const busyDuration = ref(0);
const broadcastDuration = ref(0);
const broadcastStartTime = ref<Date | null>(null);

// WebSocket 相关（保留用于状态检查，但不直接使用）
// ws.value 不再直接使用，而是通过 Rust 后端管理
let audioContext: AudioContext | null = null;
let processor: AudioWorkletNode | null = null;
let stream: MediaStream | null = null;
let audioSource: MediaStreamAudioSourceNode | null = null;
let gainNode: GainNode | null = null;
let broadcastTimer: number | null = null;
let busyTimer: number | null = null;
const maxBroadcastTime = 5 * 60;
let targetSampleRate = 0; // 将从 AudioContext 获取实际硬件采样率

// Tauri 事件监听器清理函数
const eventListeners: Array<() => void> = [];

// 预加载状态管理
const preloadState = ref({
  audioContextLoaded: false,
  audioWorkletLoaded: false,
  isPreloading: false
});

// Load preferences from localStorage
const loadPreferences = () => {
  const savedCodec = localStorage.getItem('broadcast-codec');
  if (savedCodec && (savedCodec === 'pcm' || savedCodec === 'opus')) {
    codec.value = savedCodec as CodecType;
  }

  const savedVolume = localStorage.getItem('broadcast-volume');
  if (savedVolume) {
    volume.value = parseFloat(savedVolume);
  }
};

// 监听音量变化
watch(volume, (newVolume) => {
  if (gainNode) {
    gainNode.gain.value = newVolume;
  }
});

// Load config file
const loadConfigFile = async () => {
  try {
    const config = await readConfig();
    serverUrl.value = config.serverUrl;
    settingsForm.serverUrl = config.serverUrl;
    settingsForm.updateServerBaseUrl = config.updateServerBaseUrl;
    settingsForm.defaultCodec = config.defaultCodec;
    settingsForm.defaultVolume = config.defaultVolume;
  } catch (error) {
    configLogger.error(`加载配置失败: ${error}`);
  }
};

// Show settings dialog
const showSettingsDialog = async () => {
  // 重新加载配置以获取最新值
  try {
    const config = await readConfig();
    settingsForm.serverUrl = config.serverUrl;
    settingsForm.updateServerBaseUrl = config.updateServerBaseUrl;
    settingsForm.defaultCodec = config.defaultCodec;
    settingsForm.defaultVolume = config.defaultVolume;
  } catch (error) {
    configLogger.error(`加载配置失败: ${error}`);
  }
  settingsDialogVisible.value = true;
};

// 保存设置
const saveSettings = async () => {
  // 使用表单验证
  try {
    await settingsFormRef.value?.validate();
  } catch (error) {
    // 验证失败，不继续保存
    return;
  }

  try {
    await writeConfig(settingsForm);
    serverUrl.value = settingsForm.serverUrl;
    codec.value = settingsForm.defaultCodec as CodecType;
    volume.value = settingsForm.defaultVolume;
    ElMessage.success('设置已保存');
    settingsDialogVisible.value = false;
  } catch (error) {
    configLogger.error(`保存配置失败: ${error}`);
    ElMessage.error('保存设置失败');
  }
};

// 预加载 AudioContext 和 AudioWorklet
const preloadAudioResources = async () => {
  if (preloadState.value.audioContextLoaded || preloadState.value.isPreloading) {
    return;
  }

  preloadState.value.isPreloading = true;

  const maxRetries = 3;
  let retryCount = 0;

  while (retryCount < maxRetries) {
    try {
      if (!audioContext) {
        const AudioContextConstructor = (window.AudioContext || (window as any).webkitAudioContext) as typeof AudioContext;
        audioContext = new AudioContextConstructor();

        if (audioContext.state === 'suspended') {
          await audioContext.resume();
        }
        preloadState.value.audioContextLoaded = true;
      }
      break;
    } catch (err) {
      retryCount++;
      if (retryCount >= maxRetries) {
        ElNotification.warning({
          title: '音频资源预加载失败',
          message: '首次广播时可能需要额外时间加载音频资源',
          duration: 5000,
          position: 'top-right',
        });
        if (audioContext) {
          try {
            await audioContext.close();
          } catch (e) { /* ignore */ }
          audioContext = null;
        }
        preloadState.value.audioContextLoaded = false;
        preloadState.value.audioWorkletLoaded = false;
      } else {
        await new Promise(resolve => setTimeout(resolve, 1000));
      }
    }
  }

  preloadState.value.isPreloading = false;
};

// 防抖函数：延迟执行 func，如果在 delay 时间内再次调用则重置计时器
function debounce<T extends (...args: any[]) => void>(func: T, delay: number): (...args: Parameters<T>) => void {
  let timeoutId: number | null = null;
  return (...args: Parameters<T>) => {
    if (timeoutId !== null) {
      clearTimeout(timeoutId);
    }
    timeoutId = setTimeout(() => {
      func(...args);
      timeoutId = null;
    }, delay);
  };
}

// Save preferences
const saveCodecPreference = () => {
  localStorage.setItem('broadcast-codec', codec.value);
};

// 使用防抖的音量保存函数（300ms 延迟）
const saveVolumePreferenceDebounced = debounce(() => {
  localStorage.setItem('broadcast-volume', volume.value.toString());
}, 300);

// 保存音量偏好（使用防抖版本）
const saveVolumePreference = () => {
  saveVolumePreferenceDebounced();
};

// Connection status
const connectionStatus = computed(() => {
  if (isConnecting.value) return 'connecting';
  if (isBroadcasting.value) return 'connected';
  if (status.value.includes('占用') || status.value.includes('busy')) return 'busy';
  if (status.value.includes('错误') || status.value.includes('失败')) return 'disconnected';
  return 'idle';
});

const getStatusLabel = computed(() => {
  switch (connectionStatus.value) {
    case 'connected': return '广播中';
    case 'connecting': return '连接中';
    case 'busy': return '广播被占用';
    case 'disconnected': return '连接失败';
    default: return '准备就绪';
  }
});

const setStatus = (msg: string) => {
  status.value = msg;
};

const formatTime = (seconds: number) => {
  const mins = Math.floor(seconds / 60);
  const secs = Math.floor(seconds % 60);
  return `${mins.toString().padStart(2, '0')}:${secs.toString().padStart(2, '0')}`;
};

const formatDate = (dateString: string) => {
  try {
    const date = new Date(dateString);
    return date.toLocaleDateString('zh-CN', {
      year: 'numeric',
      month: 'long',
      day: 'numeric',
    });
  } catch {
    return dateString;
  }
};

const formatReleaseNotes = (notes: string | null) => {
  if (!notes) return '<p>暂无更新说明</p>';

  // 净化 HTML 以防止 XSS 攻击
  const escapeHtml = (text: string) => {
    const div = document.createElement('div');
    div.textContent = text;
    return div.innerHTML;
  };

  // 先转义所有 HTML 特殊字符
  const escaped = escapeHtml(notes);

  // 然后应用安全的 Markdown 格式转换
  return escaped
    .replace(/^### (.*)$/gm, '<h4>$1</h4>')
    .replace(/^## (.*)$/gm, '<h3>$1</h3>')
    .replace(/^- (.*)$/gm, '<li>$1</li>')
    .replace(/\n/g, '<br>');
};

// 按钮鼠标事件处理 - 按住说话
const handleMouseDown = async () => {
  if (!isBroadcasting.value) {
    await startBroadcast();
  }
};

const handleMouseUp = () => {
  if (isBroadcasting.value) {
    stopBroadcast();
  }
};

// 触摸事件处理（移动端）
const handleTouchStart = async () => {
  if (!isBroadcasting.value) {
    await startBroadcast();
  }
};

const handleTouchEnd = () => {
  if (isBroadcasting.value) {
    stopBroadcast();
  }
};

// Keyboard handler - 空格键按住说话
const handleKeyDown = async (e: KeyboardEvent) => {
  if (e.code === 'Space' && !e.repeat && !isBroadcasting.value) {
    e.preventDefault();
    await startBroadcast();
  }
};

const handleKeyUp = (e: KeyboardEvent) => {
  if (e.code === 'Space' && isBroadcasting.value) {
    e.preventDefault();
    stopBroadcast();
  }
};

// Broadcast timer
const startBroadcastTimer = () => {
  broadcastStartTime.value = new Date();
  broadcastDuration.value = 0;

  const updateTimer = () => {
    if (!broadcastStartTime.value) return;

    const now = new Date();
    const elapsed = (now.getTime() - broadcastStartTime.value.getTime()) / 1000;
    broadcastDuration.value = elapsed;

    if (elapsed >= maxBroadcastTime) {
      stopBroadcast();
      ElMessage.warning('广播超时，已自动停止');
    }
  };

  broadcastTimer = window.setInterval(updateTimer, 100);
};

const stopBroadcastTimer = () => {
  if (broadcastTimer) {
    clearInterval(broadcastTimer);
    broadcastTimer = null;
  }
  broadcastStartTime.value = null;
};

const stopBusyTimer = () => {
  if (busyTimer) {
    clearInterval(busyTimer);
    busyTimer = null;
  }
  busyDuration.value = 0;
};

const startBroadcast = async () => {
  if (isOperating.value) return;
  if (isBroadcasting.value || isConnecting.value) return;
  if (connectionStatus.value === 'busy') return;
  if (!codec.value) return;

  isOperating.value = true;

  try {
    setStatus('请求麦克风权限...');
    isConnecting.value = true;

    const constraints: MediaStreamConstraints = {
      audio: true
    };

    stream = await navigator.mediaDevices.getUserMedia(constraints);

    setStatus('正在连接服务器...');

    if (!audioContext || !preloadState.value.audioContextLoaded) {
      const AudioContextConstructor = (window.AudioContext || (window as any).webkitAudioContext) as typeof AudioContext;
      audioContext = new AudioContextConstructor();

      if (audioContext.state === 'suspended') {
        await audioContext.resume();
      }
      preloadState.value.audioContextLoaded = true;
    } else {
      if (audioContext.state === 'suspended') {
        await audioContext.resume();
      }
    }

    // 使用 AudioContext 的实际硬件采样率
    targetSampleRate = audioContext.sampleRate;
    broadcastLogger.info(`采样率: ${targetSampleRate}Hz`);

    setStatus('正在启动 WebSocket 连接（Rust 后端）...');

    // 验证服务器 URL
    const baseUrl = serverUrl.value.replace(/\/ws$/, '');
    if (!isValidWebSocketUrl(baseUrl)) {
      const errorMsg = '无效的服务器地址，请检查配置';
      broadcastLogger.error(`URL 验证失败: ${baseUrl}`);
      setStatus(errorMsg);
      ElMessage.error('服务器地址格式错误，请使用 ws:// 或 wss:// 开头的有效地址');
      isConnecting.value = false;
      isOperating.value = false;
      return;
    }

    await invoke('ws_start_broadcast', {
      serverUrl: baseUrl,
      codec: codec.value,
      sampleRate: targetSampleRate,
      channels: 1
    });

    setStatus('等待服务器确认...');

  } catch (err: any) {
    broadcastLogger.error(`广播失败: ${err}`);
    const errorMsg = '广播失败: ' + err;
    setStatus(errorMsg);
    ElMessage.error('启动广播失败: ' + err);

    if (stream) {
      stream.getTracks().forEach(track => track.stop());
      stream = null;
    }

    isConnecting.value = false;
    isOperating.value = false;

    setTimeout(() => {
      setStatus('准备就绪');
    }, 3000);
  }
};

// 开始音频处理（当收到 ws-broadcasting 事件时调用）
const startAudioProcessing = async (stream: MediaStream) => {
  try {
    if (!audioContext) {
      throw new Error('AudioContext not initialized');
    }

    if (processor) {
      processor.disconnect();
      try { processor.port.close(); } catch (e) { /* ignore */ }
      processor = null;
    }

    if (gainNode) {
      gainNode.disconnect();
      gainNode = null;
    }
    if (audioSource) {
      audioSource.disconnect();
      audioSource = null;
    }

    audioSource = audioContext.createMediaStreamSource(stream);
    gainNode = audioContext.createGain();
    audioSource.connect(gainNode);

    const moduleUrl = `/src/audio-processor.ts?t=${Date.now()}`;
    await audioContext.audioWorklet.addModule(moduleUrl);
    preloadState.value.audioWorkletLoaded = true;

    processor = new AudioWorkletNode(audioContext, 'audio-processor');

    const messageHandler = (event: MessageEvent) => {
      const chunk = event.data.chunk;
      invoke('ws_send_audio', { data: Array.from(new Uint8Array(chunk)) })
        .catch(err => logger.error(`发送音频数据失败: ${err}`));
    };
    processor.port.onmessage = messageHandler;
    (processor as any)._messageHandler = messageHandler;

    processor.port.postMessage({
      codec: codec.value,
      sampleRate: targetSampleRate,
      volume: volume.value
    });

    gainNode.connect(processor);

    isBroadcasting.value = true;
    isConnecting.value = false;
    startBroadcastTimer();
    setStatus('广播中...');
  } catch (err) {
    audioLogger.error(`启动音频处理失败: ${err}`);
    setStatus('音频处理失败');
    ElMessage.error('音频处理失败');
    isConnecting.value = false;
  }
};

const stopBroadcast = async () => {
  if (!isBroadcasting.value && !isConnecting.value) return;

  isBroadcasting.value = false;
  isConnecting.value = false;
  isOperating.value = false;
  setStatus('已停止');
  stopBroadcastTimer();
  stopBusyTimer();

  try {
    await invoke('ws_stop_broadcast');
  } catch (err) {
    broadcastLogger.warn(`ws_stop_broadcast 失败: ${err}`);
  }

  // 确保资源总是被清理，无论 ws_stop_broadcast 是否成功
  try {
    if (stream) {
      stream.getTracks().forEach(track => {
        try { track.stop(); } catch (e) {
          broadcastLogger.warn(`停止音频轨道失败: ${e}`);
        }
      });
      stream = null;
    }

    if (gainNode) {
      try { gainNode.disconnect(); } catch (e) {
        broadcastLogger.warn(`断开 gainNode 失败: ${e}`);
      }
      gainNode = null;
    }

    if (audioSource) {
      try { audioSource.disconnect(); } catch (e) {
        broadcastLogger.warn(`断开 audioSource 失败: ${e}`);
      }
      audioSource = null;
    }

    if (processor) {
      try {
        const messageHandler = (processor as any)._messageHandler;
        if (messageHandler) {
          processor.port.onmessage = null;
          delete (processor as any)._messageHandler;
        }
        processor.port.close();
      } catch (e) {
        broadcastLogger.warn(`关闭 processor port 失败: ${e}`);
      }
      try { processor.disconnect(); } catch (e) {
        broadcastLogger.warn(`断开 processor 失败: ${e}`);
      }
      processor = null;
    }
  } catch (cleanupError) {
    broadcastLogger.error(`资源清理过程中发生错误: ${cleanupError}`);
  }
};

// 事件监听器选项常量（确保添加和移除时参数一致）
const EVENT_LISTENER_OPTIONS = { capture: true, passive: false } as const;

// 组件卸载时的清理函数
const cleanup = async () => {
  if (isBroadcasting.value) {
    await stopBroadcast();
  }

  if (gainNode) {
    gainNode.disconnect();
    gainNode = null;
  }
  if (audioSource) {
    audioSource.disconnect();
    audioSource = null;
  }
  if (processor) {
    const messageHandler = (processor as any)._messageHandler;
    if (messageHandler) {
      processor.port.onmessage = null;
      delete (processor as any)._messageHandler;
    }
    try { processor.port.close(); } catch (e) { /* ignore */ }
    processor.disconnect();
    processor = null;
  }
  if (audioContext) {
    await audioContext.close();
    audioContext = null;
  }
};

onMounted(async () => {
  // 获取应用版本号
  try {
    appVersion.value = await getVersion();
  } catch (error) {
    logger.warn(`获取应用版本失败: ${error}`);
    appVersion.value = '未知版本';
  }

  await loadConfigFile();

  try {
    configFilePath.value = await getConfigPath();
  } catch (error) {
    configLogger.error(`获取配置路径失败: ${error}`);
  }

  loadPreferences();
  await preloadAudioResources();

  // 监听 Tauri 事件
  const unlistenBroadcasting = await listen('ws-broadcasting', () => {
    if (stream) {
      startAudioProcessing(stream);
    }
  });
  eventListeners.push(unlistenBroadcasting);

  const unlistenConnected = await listen('ws-connected', () => {
    setStatus('服务器已连接');
  });
  eventListeners.push(unlistenConnected);

  const unlistenDisconnected = await listen('ws-disconnected', () => {
    setStatus('连接已断开');
    isBroadcasting.value = false;
    isConnecting.value = false;
  });
  eventListeners.push(unlistenDisconnected);

  const unlistenIdle = await listen('ws-idle', () => {
    setStatus('广播已结束');
    isBroadcasting.value = false;
  });
  eventListeners.push(unlistenIdle);

  const unlistenError = await listen('ws-error', (event: { payload: string }) => {
    eventLogger.error(`ws-error: ${event.payload}`);
    setStatus('错误: ' + event.payload);
    ElMessage.error('服务器错误: ' + event.payload);
    isConnecting.value = false;
    isBroadcasting.value = false;
    isOperating.value = false;
  });
  eventListeners.push(unlistenError);

  // 监听广播任务错误
  const unlistenBroadcastError = await listen('broadcast-error', (event: { payload: string }) => {
    eventLogger.error(`broadcast-error: ${event.payload}`);
    setStatus('广播错误: ' + event.payload);
    ElMessage.error('广播失败: ' + event.payload);
    isConnecting.value = false;
    isBroadcasting.value = false;
    isOperating.value = false;  // 重置操作标志
  });
  eventListeners.push(unlistenBroadcastError);

  window.addEventListener('keydown', handleKeyDown, EVENT_LISTENER_OPTIONS);
  window.addEventListener('keyup', handleKeyUp, EVENT_LISTENER_OPTIONS);
});

onBeforeUnmount(() => {
  // 注意：cleanup 是异步的，但 onBeforeUnmount 不保证等待
  // 我们需要确保清理操作完成
  cleanup().catch(err => {
    logger.error(`清理失败: ${err}`);
  });
  stopBroadcastTimer();
  stopBusyTimer();

  // 清理 Tauri 事件监听器
  eventListeners.forEach(unlisten => unlisten());

  // 使用同一常量移除事件监听器，确保参数完全匹配
  window.removeEventListener('keydown', handleKeyDown, EVENT_LISTENER_OPTIONS);
  window.removeEventListener('keyup', handleKeyUp, EVENT_LISTENER_OPTIONS);
});
</script>

<style>
/* ============================================
   全局样式重置 - 移除浏览器感
   ============================================ */
* {
  margin: 0;
  padding: 0;
  box-sizing: border-box;
}

html, body, #app {
  height: 100%;
  width: 100%;
  overflow: hidden;
}

body {
  font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, 'Helvetica Neue', Arial, sans-serif;
}

/* Element Plus 按钮 */
.el-button--small {
  height: 28px;
  padding: 0 12px;
  font-size: 13px;
  border-radius: 6px;
}
</style>

<style scoped>
/* ============================================
   App Container - 固定布局
   ============================================ */
.app-container {
  height: 100vh;
  width: 100vw;
  display: flex;
  flex-direction: column;
  background: #fafafa;
}

/* ============================================
   Header (48px)
   ============================================ */
.app-header {
  height: 48px;
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 0 24px;
  background: #ffffff;
  border-bottom: 1px solid #e5e7eb;
  flex-shrink: 0;
}

.header-left {
  display: flex;
  align-items: center;
  gap: 8px;
}

.app-logo {
  font-size: 18px;
  line-height: 1;
}

.app-name {
  margin: 0;
  font-size: 15px;
  font-weight: 500;
  color: #111827;
}

.header-right {
  display: flex;
  align-items: center;
  gap: 8px;
}

/* ============================================
   Main Content
   ============================================ */
.app-main {
  flex: 1;
  display: flex;
  align-items: center;
  justify-content: center;
  padding: 24px;
  overflow: hidden;
}

/* ============================================
   Control Card
   ============================================ */
.control-card {
  background: #ffffff;
  border: 1px solid #e5e7eb;
  border-radius: 16px;
  padding: 24px;
  width: 100%;
  max-width: 420px;
  box-shadow: 0 1px 2px rgba(0, 0, 0, 0.05);
  display: flex;
  flex-direction: column;
  gap: 20px;
}

/* ============================================
   Status Bar
   ============================================ */
.status-bar {
  display: flex;
  align-items: center;
  gap: 12px;
  padding: 12px 16px;
  background: #f9fafb;
  border: 1px solid #e5e7eb;
  border-radius: 12px;
  transition: all 150ms ease;
}

.status-bar.status-idle {
  border-color: #e5e7eb;
  background: #f9fafb;
}

.status-bar.status-connected {
  border-color: rgba(16, 185, 129, 0.3);
  background: rgba(16, 185, 129, 0.05);
}

.status-bar.status-connecting,
.status-bar.status-reconnecting {
  border-color: rgba(245, 158, 11, 0.3);
  background: rgba(245, 158, 11, 0.05);
}

.status-bar.status-busy,
.status-bar.status-disconnected {
  border-color: rgba(239, 68, 68, 0.3);
  background: rgba(239, 68, 68, 0.05);
}

.status-icon {
  font-size: 18px;
  min-width: 24px;
  text-align: center;
}

.status-text {
  flex: 1;
}

.status-label {
  font-size: 14px;
  font-weight: 500;
  color: #111827;
}

.status-message {
  font-size: 12px;
  color: #6b7280;
}

.busy-info {
  display: flex;
  align-items: center;
  gap: 4px;
  padding: 4px 8px;
  background: rgba(239, 68, 68, 0.1);
  border-radius: 8px;
  font-size: 11px;
  font-weight: 500;
  color: #ef4444;
}

.busy-icon {
  font-size: 12px;
}

/* ============================================
   Config Grid
   ============================================ */
.config-grid {
  display: flex;
  flex-direction: column;
  gap: 16px;
}

.config-item {
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.config-label {
  font-size: 13px;
  font-weight: 500;
  color: #6b7280;
}

:deep(.el-select .el-input__wrapper) {
  border-radius: 8px;
  box-shadow: none;
  border-color: #e5e7eb;
}

:deep(.el-select .el-input__wrapper:hover) {
  border-color: #3b82f6;
}

:deep(.el-select .el-input__wrapper.is-focus) {
  border-color: #3b82f6;
  box-shadow: 0 0 0 2px rgba(59, 130, 246, 0.1);
}

:deep(.el-slider__runway) {
  height: 4px;
  background: #f3f4f6;
}

:deep(.el-slider__bar) {
  background: #3b82f6;
}

:deep(.el-slider__button) {
  width: 14px;
  height: 14px;
  border: 2px solid #3b82f6;
}

/* ============================================
   Broadcast Button
   ============================================ */
.broadcast-btn {
  width: 100%;
  height: 56px;
  background: #3b82f6;
  color: white;
  border: none;
  border-radius: 12px;
  font-size: 15px;
  font-weight: 500;
  cursor: pointer;
  transition: all 150ms ease;
  display: flex;
  align-items: center;
  justify-content: center;
  gap: 8px;
}

.broadcast-btn:hover:not(:disabled) {
  background: #2563eb;
  transform: translateY(-1px);
  box-shadow: 0 4px 6px -1px rgba(0, 0, 0, 0.1);
}

.broadcast-btn:active:not(:disabled) {
  transform: translateY(0);
}

.broadcast-btn:disabled {
  opacity: 0.5;
  cursor: not-allowed;
}

.broadcast-btn.broadcasting {
  background: #ef4444;
  animation: pulse 2s ease-in-out infinite;
}

@keyframes pulse {
  0%, 100% {
    box-shadow: 0 0 0 0 rgba(239, 68, 68, 0.4);
  }
  50% {
    box-shadow: 0 0 0 8px rgba(239, 68, 68, 0);
  }
}

.btn-icon {
  font-size: 16px;
}

/* ============================================
   Timer & Hints
   ============================================ */
.timer-section {
  display: flex;
  align-items: center;
  justify-content: center;
  gap: 6px;
  height: 36px;
  font-size: 15px;
  font-weight: 500;
  color: #10b981;
  background: rgba(16, 185, 129, 0.1);
  border-radius: 8px;
}

.timer-icon {
  font-size: 14px;
}

.timer-text {
  font-variant-numeric: tabular-nums;
}

.hints-section {
  height: 36px;
  display: flex;
  align-items: center;
  justify-content: center;
  font-size: 13px;
  color: #9ca3af;
  text-align: center;
}

/* ============================================
   Footer (32px)
   ============================================ */
.app-footer {
  height: 32px;
  display: flex;
  align-items: center;
  justify-content: center;
  background: #ffffff;
  border-top: 1px solid #e5e7eb;
  flex-shrink: 0;
}

.version {
  font-size: 11px;
  color: #9ca3af;
}

/* ============================================
   Update Dialog Styles
   ============================================ */
.update-header {
  text-align: center;
  margin-bottom: 24px;
  padding: 20px;
  background: #3b82f6;
  border-radius: 12px;
  color: white;
}

.update-icon {
  font-size: 48px;
  margin-bottom: 12px;
}

.update-header h3 {
  margin: 0;
  font-size: 20px;
  font-weight: 600;
}

.update-content {
  padding: 0 8px;
}

.update-info {
  background: #f9fafb;
  border-radius: 8px;
  padding: 16px;
  margin-bottom: 20px;
}

.info-item {
  display: flex;
  justify-content: space-between;
  padding: 8px 0;
  border-bottom: 1px solid #f3f4f6;
}

.info-item:last-child {
  border-bottom: none;
}

.info-label {
  color: #6b7280;
  font-weight: 500;
}

.info-value {
  color: #111827;
  font-weight: 600;
}

.update-notes {
  background: #ffffff;
  border: 1px solid #e5e7eb;
  border-radius: 8px;
  padding: 16px;
}

.update-notes h4 {
  margin: 0 0 12px 0;
  font-size: 14px;
  font-weight: 600;
  color: #111827;
}

.notes-content {
  color: #6b7280;
  line-height: 1.6;
  font-size: 13px;
}

.notes-content h3 {
  margin: 16px 0 8px 0;
  font-size: 15px;
  font-weight: 600;
  color: #111827;
}

.notes-content h4 {
  margin: 12px 0 6px 0;
  font-size: 14px;
  font-weight: 600;
  color: #111827;
}

.notes-content li {
  margin: 4px 0;
  padding-left: 20px;
  position: relative;
}

.notes-content li:before {
  content: '•';
  position: absolute;
  left: 8px;
  color: #3b82f6;
  font-weight: bold;
}

.no-update {
  text-align: center;
  padding: 40px 20px;
}

.no-update-icon {
  font-size: 64px;
  margin-bottom: 16px;
}

.no-update h3 {
  margin: 0 0 12px 0;
  font-size: 20px;
  font-weight: 600;
  color: #111827;
}

.no-update p {
  margin: 0;
  color: #9ca3af;
  font-size: 14px;
}

/* ============================================
   Settings Dialog Styles
   ============================================ */
.config-path-info {
  margin-top: 16px;
  padding: 12px;
  background: #f9fafb;
  border-radius: 8px;
  display: flex;
  flex-direction: column;
  gap: 4px;
}

.config-path-info .info-label {
  font-size: 12px;
  color: #6b7280;
}

.config-path-info .info-value {
  font-size: 12px;
  color: #111827;
  font-family: monospace;
  word-break: break-all;
}

:deep(.el-form-item__label) {
  color: #6b7280;
  font-weight: 500;
}

.form-item-hint {
  margin-top: 4px;
  font-size: 12px;
  color: #9ca3af;
}
</style>
