import { ref, onMounted, onBeforeUnmount } from 'vue';
import { invoke } from '@tauri-apps/api/core';
import { ElMessage, ElNotification } from 'element-plus';

export interface UpdateInfo {
  available: boolean;
  version: string | null;
  body: string | null;
  date: string | null;
}

// 单例模式：确保只有一个实例
let singletonInstance: ReturnType<typeof createAppUpdate> | null = null;

function createAppUpdate() {
  const updateAvailable = ref(false);
  const updateInfo = ref<UpdateInfo>({
    available: false,
    version: null,
    body: null,
    date: null,
  });
  const isDownloading = ref(false);
  const isInstalling = ref(false);
  let checkTimer: number | null = null;
  let isInitialized = false;

  // Check for updates
  const checkForUpdates = async (silent = false) => {
    try {
      const info = await invoke<UpdateInfo>('get_update_info');
      updateInfo.value = info;

      if (info.available) {
        updateAvailable.value = true;

        if (!silent) {
          ElNotification({
            title: '发现新版本',
            message: `版本 ${info.version} 可用，点击查看更新内容`,
            type: 'info',
            duration: 0,
            position: 'top-right',
            onClick: () => {
              showUpdateDialog();
            },
          });
        }
      }
    } catch (error) {
      console.error('Failed to check for updates:', error);
      if (!silent) {
        ElMessage.error('检查更新失败');
      }
    }
  };

  // Download and install update
  const downloadAndUpdate = async () => {
    try {
      isDownloading.value = true;
      ElMessage.info('正在下载更新...');

      await invoke('install_update');

      isDownloading.value = false;
      isInstalling.value = true;

      ElNotification({
        title: '更新下载完成',
        message: '更新已下载，将在应用关闭后自动安装。请手动关闭应用以完成更新。',
        type: 'success',
        duration: 0,
        position: 'top-right',
      });
    } catch (error: any) {
      isDownloading.value = false;
      console.error('Failed to download update:', error);
      ElMessage.error(error || '下载更新失败');
    }
  };

  // Show update dialog
  const showUpdateDialog = () => {
    const message = updateInfo.value.body || '更新内容加载中...';

    ElNotification({
      title: `新版本 ${updateInfo.value.version} 可用`,
      message: message,
      type: 'info',
      duration: 0,
      position: 'top-right',
      dangerouslyUseHTMLString: true,
      customClass: 'update-notification',
    });
  };

  // Load update configuration
  const loadUpdateConfig = async () => {
    try {
      const response = await fetch('/update-config.json');
      if (!response.ok) {
        throw new Error(`HTTP ${response.status}: ${response.statusText}`);
      }
      const config = await response.json();
      return config.update;
    } catch (error) {
      console.error('Failed to load update config:', error);
      // 仅在开发环境中警告
      if (import.meta.env?.DEV) {
        console.warn('[AppUpdate] 使用默认更新配置，请检查 update-config.json');
      }
      return {
        enabled: true,
        checkInterval: 86400000,
        autoInstallOnExit: true,
        showNotification: true,
      };
    }
  };

  // Initialize update checker
  const initUpdateChecker = async () => {
    if (isInitialized) {
      console.log('[AppUpdate] 已经初始化，跳过重复初始化');
      return;
    }

    const config = await loadUpdateConfig();

    if (config.enabled) {
      // Check for updates on startup
      await checkForUpdates(true);

      // Set up periodic checks
      if (config.checkInterval > 0) {
        checkTimer = window.setInterval(() => {
          checkForUpdates(true);
        }, config.checkInterval);
      }
    }

    isInitialized = true;
  };

  // Cleanup on unmount
  const cleanup = () => {
    if (checkTimer) {
      clearInterval(checkTimer);
      checkTimer = null;
    }
    isInitialized = false;
  };

  // Auto-install on exit（同步函数，确保在 beforeunload 中执行）
  const handleExit = () => {
    // 使用同步方式调用，因为 beforeunload 不等待异步操作
    loadUpdateConfig().then(config => {
      if (config.autoInstallOnExit && updateAvailable.value && !isInstalling.value) {
        invoke('install_update')
          .then(() => {
            ElNotification({
              title: '正在安装更新',
              message: '更新将在应用关闭后自动应用',
              type: 'success',
              duration: 3000,
            });
          })
          .catch(error => {
            console.error('Failed to install update on exit:', error);
            // 向用户显示错误通知
            ElNotification({
              title: '更新安装失败',
              message: '自动安装更新失败，请手动检查更新',
              type: 'error',
              duration: 5000,
            });
          });
      }
    }).catch(error => {
      console.error('Failed to load update config on exit:', error);
    });
  };

  onMounted(() => {
    initUpdateChecker();

    // Set up exit handler
    window.addEventListener('beforeunload', handleExit);
  });

  onBeforeUnmount(() => {
    cleanup();
    window.removeEventListener('beforeunload', handleExit);
  });

  return {
    updateAvailable,
    updateInfo,
    isDownloading,
    isInstalling,
    checkForUpdates,
    downloadAndUpdate,
    showUpdateDialog,
  };
}

export function useAppUpdate() {
  if (!singletonInstance) {
    singletonInstance = createAppUpdate();
  }
  return singletonInstance;
}
