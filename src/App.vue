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
            <span v-else-if="connectionStatus === 'reconnecting'">🔄</span>
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
          :disabled="connectionStatus === 'busy' || connectionStatus === 'disconnected'"
          @mousedown="startBroadcast"
          @mouseup="stopBroadcast"
          @mouseleave="stopBroadcast"
          @touchstart.prevent="startBroadcast"
          @touchend.prevent="stopBroadcast"
        >
          <template v-if="connectionStatus === 'busy'">
            <span class="btn-icon">🚫</span>
            <span>广播被占用</span>
          </template>
          <template v-else-if="!isBroadcasting">
            <span class="btn-icon">🎙️</span>
            <span>按住说话</span>
          </template>
          <template v-else>
            <span class="btn-icon">🔴</span>
            <span>广播中...</span>
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
      <span class="version">v1.0.0</span>
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
              <span class="info-value">1.0.0</span>
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
      <el-form :model="settingsForm" label-width="100px">
        <el-form-item label="服务器地址">
          <el-input
            v-model="settingsForm.serverUrl"
            placeholder="ws://localhost:8080/broadcast"
          />
        </el-form-item>
        <el-form-item label="默认编码">
          <el-select v-model="settingsForm.defaultCodec">
            <el-option label="PCM（最低延迟）" value="pcm" />
            <el-option label="Opus（省流量）" value="opus" />
          </el-select>
        </el-form-item>
        <el-form-item label="默认音量">
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
import { useWebSocket } from '@/utils/useWebSocket';
import { useAppUpdate } from '@/utils/useAppUpdate';
import { APP_CONFIG } from '@/config';
import { readConfig, writeConfig, getConfigPath, type AppConfig } from '@/composables';
import type { CodecType } from '@/types/index';

const codec = ref<CodecType>(APP_CONFIG.DEFAULT_CODEC);
const volume = ref(APP_CONFIG.DEFAULT_VOLUME);
const serverUrl = ref(APP_CONFIG.WS_URL);
const updateDialogVisible = ref(false);
const settingsDialogVisible = ref(false);
const configFilePath = ref('');

// 设置表单
const settingsForm = reactive<AppConfig>({
  serverUrl: 'ws://localhost:8080/broadcast',
  defaultCodec: 'pcm',
  defaultVolume: 1.0,
});

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

const ws = ref<ReturnType<typeof useWebSocket> | null>(null);
let audioContext: AudioContext | null = null;
let processor: AudioWorkletNode | null = null;
let stream: MediaStream | null = null;
let audioSource: MediaStreamAudioSourceNode | null = null;
let gainNode: GainNode | null = null;
let broadcastTimer: number | null = null;
let busyTimer: number | null = null;
const maxBroadcastTime = 5 * 60;

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

// 监听音量变化 - 移到 setup 顶层作用域
watch(volume, (newVolume) => {
  if (gainNode) {
    console.log(`[Volume] 实时更新音量: ${newVolume}`);
    gainNode.gain.value = newVolume;
  }
});

// Load config file
const loadConfigFile = async () => {
  try {
    const config = await readConfig();
    serverUrl.value = config.serverUrl;
    settingsForm.serverUrl = config.serverUrl;
    settingsForm.defaultCodec = config.defaultCodec;
    settingsForm.defaultVolume = config.defaultVolume;
    console.log('[Config] 配置文件已加载:', config);
  } catch (error) {
    console.error('[Config] 加载配置文件失败:', error);
  }
};

// Show settings dialog
const showSettingsDialog = async () => {
  // 重新加载配置以获取最新值
  try {
    const config = await readConfig();
    settingsForm.serverUrl = config.serverUrl;
    settingsForm.defaultCodec = config.defaultCodec;
    settingsForm.defaultVolume = config.defaultVolume;
  } catch (error) {
    console.error('[Config] 加载配置失败:', error);
  }
  settingsDialogVisible.value = true;
};

// Save settings
const saveSettings = async () => {
  try {
    await writeConfig(settingsForm);
    serverUrl.value = settingsForm.serverUrl;
    codec.value = settingsForm.defaultCodec as CodecType;
    volume.value = settingsForm.defaultVolume;
    ElMessage.success('设置已保存');
    settingsDialogVisible.value = false;
  } catch (error) {
    console.error('[Config] 保存配置失败:', error);
    ElMessage.error('保存设置失败');
  }
};

// 预加载 AudioContext 和 AudioWorklet
const preloadAudioResources = async () => {
  if (preloadState.value.audioContextLoaded || preloadState.value.isPreloading) {
    return;
  }

  preloadState.value.isPreloading = true;
  console.log('[Preload] 开始预加载音频资源...');

  const maxRetries = 3;
  let retryCount = 0;

  while (retryCount < maxRetries) {
    try {
      if (!audioContext) {
        // 使用类型安全的 AudioContext 创建方式
        const AudioContextConstructor = (window.AudioContext || (window as any).webkitAudioContext) as typeof AudioContext;
        audioContext = new AudioContextConstructor();
        console.log('[Preload] AudioContext 已创建, sampleRate:', audioContext.sampleRate);

        if (audioContext.state === 'suspended') {
          await audioContext.resume();
          console.log('[Preload] AudioContext 已恢复');
        }
        preloadState.value.audioContextLoaded = true;
      }

      if (!preloadState.value.audioWorkletLoaded) {
        await audioContext.audioWorklet.addModule('/src/audio-processor.ts');
        console.log('[Preload] AudioWorklet 模块已加载');
        preloadState.value.audioWorkletLoaded = true;
      }

      console.log('[Preload] ✅ 音频资源预加载完成');
      break;
    } catch (err) {
      retryCount++;
      console.warn(`[Preload] 预加载失败，重试 ${retryCount}/${maxRetries}:`, err);

      if (retryCount >= maxRetries) {
        console.error('[Preload] ❌ 预加载失败，将在广播时重试');
        // 向用户显示通知
        ElNotification.warning({
          title: '音频资源预加载失败',
          message: '首次广播时可能需要额外时间加载音频资源',
          duration: 5000,
          position: 'top-right',
        });
        // 清理已创建的 AudioContext 以避免状态不一致
        if (audioContext) {
          try {
            await audioContext.close();
            console.log('[Preload] AudioContext 已关闭');
          } catch (err) {
            console.warn('[Preload] 关闭 AudioContext 时出错:', err);
          }
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

// Save preferences
const saveCodecPreference = () => {
  localStorage.setItem('broadcast-codec', codec.value);
};

const saveVolumePreference = () => {
  localStorage.setItem('broadcast-volume', volume.value.toString());
};

// Connection status
const connectionStatus = computed(() => {
  if (ws.value && ws.value.status() === 'reconnecting') return 'reconnecting';
  if (ws.value && ws.value.status() === 'connecting') return 'connecting';
  if (isBroadcasting.value) return 'connected';
  if (status.value.includes('占用') || status.value.includes('busy')) return 'busy';
  if (status.value.includes('错误') || status.value.includes('失败')) return 'disconnected';
  return 'idle';
});

const getStatusLabel = computed(() => {
  switch (connectionStatus.value) {
    case 'connected': return '广播中';
    case 'connecting': return '连接中';
    case 'reconnecting': return '重连中';
    case 'busy': return '广播被占用';
    case 'disconnected': return '已断开';
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

// Keyboard handler
const handleKeyDown = async (e: KeyboardEvent) => {
  if (e.code === 'Space' && !e.repeat && !isBroadcasting.value && connectionStatus.value !== 'busy') {
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

// Busy timer
const startBusyTimer = () => {
  busyDuration.value = 0;

  busyTimer = window.setInterval(() => {
    busyDuration.value += 0.1;
  }, 100);
};

const stopBusyTimer = () => {
  if (busyTimer) {
    clearInterval(busyTimer);
    busyTimer = null;
  }
  busyDuration.value = 0;
};

const startBroadcast = async () => {
  if (isOperating.value) {
    console.log('[Broadcast] 操作进行中，忽略请求');
    return;
  }
  if (isBroadcasting.value || isConnecting.value) return;
  if (connectionStatus.value === 'busy') return;
  if (!codec.value) return;

  isOperating.value = true;
  console.log('[Broadcast] 开始广播流程');

  try {
    setStatus('请求麦克风权限...');
    isConnecting.value = true;

    const constraints: MediaStreamConstraints = {
      audio: true
    };

    stream = await navigator.mediaDevices.getUserMedia(constraints);
    console.log('[Broadcast] MediaStream obtained:', stream.id);

    setStatus('正在连接服务器...');

    if (!audioContext || !preloadState.value.audioContextLoaded) {
      console.log('[Broadcast] AudioContext 未预加载，立即创建...');
      const AudioContextConstructor = (window.AudioContext || (window as any).webkitAudioContext) as typeof AudioContext;
      audioContext = new AudioContextConstructor();
      console.log('[AudioContext] Created new AudioContext');
      console.log('[AudioContext] sampleRate (actual):', audioContext.sampleRate);
      console.log('[AudioContext] state:', audioContext.state);

      if (audioContext.state === 'suspended') {
        console.log('[AudioContext] Resuming suspended AudioContext...');
        await audioContext.resume();
        console.log('[AudioContext] state after resume:', audioContext.state);
      }
      preloadState.value.audioContextLoaded = true;
    } else {
      console.log('[Broadcast] 复用已预加载的 AudioContext');
      console.log('[AudioContext] sampleRate:', audioContext.sampleRate);
      console.log('[AudioContext] state:', audioContext.state);

      if (audioContext.state === 'suspended') {
        await audioContext.resume();
        console.log('[AudioContext] 已恢复到运行状态');
      }
    }

    // broadcast-manager 使用 snake_case 参数格式
    const sampleRateParam = `&sample_rate=${audioContext.sampleRate}`;
    const channelsParam = `&channels=1`;
    const wsUrl = `${serverUrl.value}?codec=${codec.value}${sampleRateParam}${channelsParam}`;
    console.log('[Broadcast] Connecting to:', wsUrl);

    // 关闭现有连接（如果存在）
    if (ws.value) {
      console.log('[Broadcast] Closing existing WebSocket connection before creating new one');
      const oldWs = ws.value;
      ws.value = null;
      oldWs.close();
    }

    ws.value = useWebSocket(wsUrl, {
      autoReconnect: false,
      maxRetries: 5,
      onOpen: () => {
        console.log('[WebSocket] Connected to server');
        setStatus('服务器已连接');
      },
      onMessage: (data) => {
        console.log('[WebSocket] Received message:', data);
        if (typeof data === 'string') {
          if (data === 'ready') {
            console.log('[WebSocket] Ready signal received, sending start_broadcast');
            // 发送开始广播命令（broadcast-manager 协议）
            if (ws.value) {
              ws.value.send('start_broadcast');
            }
          } else if (data === 'broadcasting') {
            // 收到广播确认后才开始音频处理（broadcast-manager 协议）
            console.log('[WebSocket] Broadcasting confirmed');
            if (!stream) {
              throw new Error('MediaStream is null');
            }
            startAudioProcessing(stream);
            isBroadcasting.value = true;
            isConnecting.value = false;
            startBroadcastTimer();
            setStatus('广播中...');
            stopBusyTimer();
          } else if (data === 'idle') {
            // broadcast-manager 广播结束状态
            console.log('[WebSocket] Broadcast idle');
            setStatus('已停止');
          } else if (data.startsWith('busy|')) {
            console.log('[WebSocket] Busy signal received');
            const duration = parseFloat(data.split('|')[1]);
            busyDuration.value = duration;
            setStatus('广播被占用');
            startBusyTimer();
            isConnecting.value = false;
            ElMessage.error('广播被占用，请稍后再试');
          } else if (data.startsWith('error:')) {
            console.error('[WebSocket] Error received:', data);
            const errorMsg = data.split(':', 2)[1];
            setStatus('错误: ' + errorMsg);
            ElMessage.error(errorMsg);
            isConnecting.value = false;
          }
        }
      },
      onClose: () => {
        console.log('[WebSocket] Connection closed');
        if (isBroadcasting.value) {
          setStatus('连接断开');
          stopBroadcast();
        }
      },
      onError: (err) => {
        console.error('[WebSocket] Error:', err);
        setStatus('连接错误: ' + err);
        isConnecting.value = false;
      }
    });
  } catch (err: any) {
    console.error('Microphone access failed', err);
    setStatus('麦克风错误: ' + err.message);
    ElMessage.error('无法访问麦克风');

    if (stream) {
      stream.getTracks().forEach(track => track.stop());
      console.log('[Broadcast] MediaStream released due to error');
      stream = null;
    }

    isConnecting.value = false;
  } finally {
    isOperating.value = false;
  }
};

const startAudioProcessing = async (stream: MediaStream) => {
  try {
    if (!audioContext) {
      throw new Error('AudioContext not initialized');
    }

    console.log('[AudioProcessor] Using audioContext with sampleRate:', audioContext.sampleRate);

    if (processor) {
      console.log('[AudioProcessor] Cleaning up old processor...');
      processor.disconnect();
      try {
        processor.port.close();
        console.log('[AudioProcessor] Old processor port closed');
      } catch (err) {
        console.warn('[AudioProcessor] Error closing old processor port:', err);
      }
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
    // 音量只在 processor 中应用，移除这里的重复设置
    audioSource.connect(gainNode);

    if (!preloadState.value.audioWorkletLoaded) {
      console.log('[AudioProcessor] AudioWorklet 未预加载，立即加载...');
      await audioContext.audioWorklet.addModule('/src/audio-processor.ts');
      preloadState.value.audioWorkletLoaded = true;
    } else {
      console.log('[AudioProcessor] 复用已预加载的 AudioWorklet 模块');
    }

    processor = new AudioWorkletNode(audioContext, 'audio-processor');

    // 保存消息处理器引用，便于后续清理
    const messageHandler = (event: MessageEvent) => {
      const chunk = event.data.chunk;
      const sampleRate = event.data.sampleRate;
      const frameSize = event.data.frameSize;
      console.log('[App] Received chunk from processor:',
                  'size=', chunk.byteLength,
                  'sampleRate=', sampleRate,
                  'frameSize=', frameSize);

      if (ws.value && ws.value.status() === 'open') {
        ws.value.send(chunk);
      } else {
        console.warn('[App] WebSocket not ready, dropping chunk');
      }
    };
    processor.port.onmessage = messageHandler;
    // 保存处理器引用用于清理
    (processor as any)._messageHandler = messageHandler;

    processor.port.postMessage({
      codec: codec.value,
      sampleRate: audioContext.sampleRate,
      volume: volume.value
    });

    gainNode.connect(processor);

    console.log('[App] Audio processing started successfully');
  } catch (err) {
    console.error('Failed to start audio processing', err);
    setStatus('音频处理失败');
    ElMessage.error('音频处理失败');
  }
};

const stopBroadcast = async () => {
  console.log('[Broadcast] 停止广播，关闭所有连接');
  isBroadcasting.value = false;
  setStatus('已停止');
  stopBroadcastTimer();
  stopBusyTimer();

  if (stream) {
    console.log('[Broadcast] Stopping MediaStream tracks...');
    stream.getTracks().forEach(track => {
      track.stop();
      console.log('[Broadcast] Stopped track:', track.kind, track.label, 'enabled:', track.enabled);
    });
    stream = null;
    console.log('[Broadcast] MediaStream released');
  }

  if (gainNode) {
    console.log('[Broadcast] Disconnecting gainNode...');
    gainNode.disconnect();
    gainNode = null;
  }
  if (audioSource) {
    console.log('[Broadcast] Disconnecting audioSource...');
    audioSource.disconnect();
    audioSource = null;
  }

  if (processor) {
    console.log('[Broadcast] Disconnecting processor...');
    processor.disconnect();

    // 清理消息处理器
    const messageHandler = (processor as any)._messageHandler;
    if (messageHandler) {
      processor.port.onmessage = null;
      delete (processor as any)._messageHandler;
      console.log('[Broadcast] Processor message handler cleaned up');
    }

    try {
      processor.port.close();
      console.log('[Broadcast] Processor port closed');
    } catch (err) {
      console.warn('[Broadcast] Error closing processor port:', err);
    }

    processor = null;
    console.log('[Broadcast] Audio processor disconnected');
  }

  if (ws.value) {
    // 发送停止广播命令（broadcast-manager 协议）
    if (ws.value.status() === 'open') {
      console.log('[Broadcast] Sending stop_broadcast command');
      ws.value.send('stop_broadcast');
    }
    ws.value.close();
    ws.value = null;
    console.log('[Broadcast] WebSocket closed');
  }

  console.log('[Broadcast] 保留 AudioContext 和 AudioWorklet 以供下次使用');

  await new Promise(resolve => setTimeout(resolve, 100));
  console.log('[Broadcast] Cleanup complete');
};

// 事件监听器选项常量（确保添加和移除时参数一致）
const EVENT_LISTENER_OPTIONS = { capture: true, passive: false } as const;

// 组件卸载时的清理函数
const cleanup = async () => {
  // 停止正在进行的广播
  if (isBroadcasting.value) {
    await stopBroadcast();
  }

  // 释放音频资源
  if (gainNode) {
    gainNode.disconnect();
    gainNode = null;
  }
  if (audioSource) {
    audioSource.disconnect();
    audioSource = null;
  }
  // 清理 processor 的消息处理器，防止内存泄漏
  if (processor) {
    const messageHandler = (processor as any)._messageHandler;
    if (messageHandler) {
      processor.port.onmessage = null;
      delete (processor as any)._messageHandler;
      console.log('[Cleanup] Processor message handler 已清理');
    }
    try {
      processor.port.close();
    } catch (err) {
      console.warn('[Cleanup] 关闭 processor port 时出错:', err);
    }
    processor.disconnect();
    processor = null;
  }
  if (audioContext) {
    await audioContext.close();
    audioContext = null;
  }

  // 关闭 WebSocket 连接
  if (ws.value) {
    ws.value.close();
    ws.value = null;
  }
};

onMounted(async () => {
  // 加载配置文件
  await loadConfigFile();

  // 获取配置文件路径用于显示
  try {
    configFilePath.value = await getConfigPath();
  } catch (error) {
    console.error('[Config] 获取配置路径失败:', error);
  }

  loadPreferences();
  await preloadAudioResources();

  window.addEventListener('keydown', handleKeyDown, EVENT_LISTENER_OPTIONS);
  window.addEventListener('keyup', handleKeyUp, EVENT_LISTENER_OPTIONS);
});

onBeforeUnmount(() => {
  // 注意：cleanup 是异步的，但 onBeforeUnmount 不保证等待
  // 我们需要确保清理操作完成
  cleanup().catch(err => {
    console.error('[Cleanup] 清理失败:', err);
  });
  stopBroadcastTimer();
  stopBusyTimer();
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
</style>
