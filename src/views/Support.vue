<script setup lang="ts">
import { ref, onMounted, onUnmounted } from 'vue';
import { Snackbar } from '@varlet/ui';
import { invoke } from '@tauri-apps/api/core';
import { listen } from '@tauri-apps/api/event';
import alipayImg from '@/assets/images/support/alipay.jpg'
import wechatImg from '@/assets/images/support/wechat.png'
import qqImg from '@/assets/images/support/qq.png'
import Icon from "../components/Icon.vue";

const activeTab = ref(0);

const paymentMethods = [
  {
    name: '微信',
    icon: 'wechat',
    type: 'qrcode' as const,
    qrcode: wechatImg,
    desc: '扫描二维码向我转账'
  },
  {
    name: '支付宝',
    icon: 'alipay',
    type: 'qrcode' as const,
    qrcode: alipayImg,
    desc: '扫描二维码向我转账'
  },
  {
    name: 'QQ',
    icon: 'qq',
    type: 'qrcode' as const,
    qrcode: qqImg,
    desc: '扫描二维码向我转账'
  },
  {
    name: '爱发电',
    icon: 'flashlight',
    type: 'link' as const,
    url: 'https://afdian.com',
    desc: '点击前往爱发电支持我'
  }
];

function goBack() {
  window.dispatchEvent(new CustomEvent('navigateToMenu', { detail: 'settings' }));
}

// function openLink(url: string) {
//   if (typeof window !== 'undefined') {
//     window.open(url, '_blank');
//   }
// }

async function saveQrcode(method: any) {
  try {
    const imgSrc = method.qrcode;
    const response = await fetch(imgSrc);
    const blob = await response.blob();

    const base64 = await new Promise<string>((resolve, reject) => {
      const reader = new FileReader();
      reader.onload = () => {
        const result = reader.result as string;
        const base64 = result.split(',')[1];
        resolve(base64);
      };
      reader.onerror = reject;
      reader.readAsDataURL(blob);
    });

    const ext = method.name.includes('微信') ? 'png' : 'jpg';
    const filename = `${method.name}收款码.${ext}`;

    const result = await invoke<string>('save_image_to_gallery', {
      imageData: base64,
      filename: filename,
    });

    if (isAndroidTauri()) {
      Snackbar.success('图片已保存到相册');
    } else {
      Snackbar.success(`已保存到 ${result}`);
    }
  } catch (error) {
    console.error('保存失败:', error);
    Snackbar.error('保存失败');
  }
}

const isAndroidTauri = () => {
  return /Android/i.test(navigator.userAgent);
};

let unlistenSaveImage: (() => void) | null = null;

onMounted(async () => {
  if (isAndroidTauri()) {
    unlistenSaveImage = await listen<{ path: string; displayName: string }>('saveImageToGallery', (event) => {
      const win = window as any;
      if (win.AndroidNative && win.AndroidNative.saveImageToGallery) {
        win.AndroidNative.saveImageToGallery(JSON.stringify(event.payload));
      }
    });
  }
});

onUnmounted(() => {
  if (isAndroidTauri()) {
    if (unlistenSaveImage) {
      unlistenSaveImage();
    }
  }
});
</script>

<template>
  <div class="support">
    <var-app-bar class="floating-app-bar" title="支持一下">
      <template #left>
        <button class="btn-icon" @click="goBack">
          <Icon name="arrow-left" :size="24" />
        </button>
      </template>
    </var-app-bar>

    <div class="content">
      <div class="support-header">
        <!-- <div class="heart-icon">❤️</div> -->
        <h2>我叫红云</h2>
        <p>
          道祖鸿钧, 大会讲道<br>
          西方秃驴, 接引准提<br>
          道德绑架, 抢我蒲团<br>
          妖师鲲鹏, 同遭绑架<br>
          技不如人, 迁怒于我<br>
          设计偷袭, 大意没闪<br>
          爱子未至, 含恨而终<br>
          <br>
          天不亡我, 又活一世<br>
          格物致知, 悟道赛博<br>
          证道在即, 将赴洪荒<br>
          v我5块, 带你飞！<br>
        </p>
      </div>

      <div class="payment-section">
        <var-tabs v-model:active="activeTab" class="payment-tabs">
          <var-tab
            v-for="(method, index) in paymentMethods"
            :key="index"
            :name="index"
          >
            <Icon :name="method.icon" :size="18" />
            <span>{{ method.name }}</span>
          </var-tab>
        </var-tabs>

        <var-tabs-items v-model:active="activeTab" class="payment-content">
          <var-tab-item
            v-for="(method, index) in paymentMethods"
            :key="index"
          >
            <div class="payment-card">
              <div v-if="method.type === 'qrcode'" class="qrcode-section">
                <div v-if="method.qrcode" class="qrcode-wrapper" @click="saveQrcode(method)">
                  <var-image
                    :src="method.qrcode"
                    fit="contain"
                    class="qrcode-image"
                  />
                </div>
                <div v-else class="qrcode-placeholder">
                  <Icon name="qr-code" :size="64" />
                  <p>请添加{{ method.name }}收款码图片</p>
                  <p class="hint">将二维码图片路径填入 Support.vue 的 paymentMethods 中</p>
                </div>
                <!-- <p class="payment-desc">{{ method.desc }}</p> -->
                <!-- <p class="save-hint" v-if="method.qrcode">点击图片保存到下载目录</p> -->
              </div>

              <div v-else-if="method.type === 'link'" class="link-section">
                <!-- <div class="link-card" @click="openLink(method.url)"> -->
                <div class="link-card">
                  <Icon name="external-link" :size="48" />
                  <h3>爱发电</h3>
                  <!-- <p>{{ method.desc }}</p>
                  <var-button type="primary" size="large" round>
                    立即支持
                  </var-button> -->
                </div>
              </div>
            </div>
          </var-tab-item>
        </var-tabs-items>
      </div>

      <div class="thanks-section">
        <!-- <p>✨ 感谢每一位支持者 ✨</p> -->
      </div>
    </div>
  </div>
</template>

<style scoped>
.support {
  height: 100vh;
  display: flex;
  flex-direction: column;
  background-color: var(--color-body);
}

.floating-app-bar {
  flex-shrink: 0;
}

.content {
  flex: 1;
  overflow-y: auto;
  padding: 16px;
  -webkit-overflow-scrolling: touch;
}

.support-header {
  text-align: center;
  padding: 24px 16px 16px;
}

.heart-icon {
  font-size: 48px;
  margin-bottom: 12px;
  animation: heartbeat 1.5s ease-in-out infinite;
}

@keyframes heartbeat {
  0%, 100% { transform: scale(1); }
  25% { transform: scale(1.1); }
  50% { transform: scale(1); }
  75% { transform: scale(1.1); }
}

.support-header h2 {
  margin: 0 0 8px;
  font-size: 22px;
  font-weight: 700;
  color: var(--color-text);
}

.support-header p {
  margin: 0;
  font-size: 14px;
  color: var(--color-text-2);
}

.payment-section {
  margin-top: 16px;
}

.payment-tabs {
  margin-bottom: 16px;
}

.payment-tabs :deep(.var-tab) {
  display: flex;
  align-items: center;
  gap: 6px;
  font-size: 14px;
}

.payment-content {
  min-height: 300px;
}

.payment-card {
  padding: 16px 0;
}

.qrcode-section {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 16px;
}

.qrcode-wrapper {
  width: 240px;
  height: 240px;
  border-radius: 16px;
  overflow: hidden;
  box-shadow: var(--shadow-md);
  background-color: var(--color-surface);
  cursor: pointer;
  transition: transform 0.2s ease, box-shadow 0.2s ease;
}

.qrcode-wrapper:hover {
  transform: scale(1.02);
  box-shadow: var(--shadow-lg);
}

.qrcode-wrapper:active {
  transform: scale(0.98);
}

.save-hint {
  margin: 4px 0 0;
  font-size: 12px;
  color: var(--color-text-3);
}

.qrcode-image {
  width: 100%;
  height: 100%;
}

.qrcode-placeholder {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  width: 240px;
  height: 240px;
  border-radius: 16px;
  border: 2px dashed var(--color-border);
  background-color: var(--color-surface-variant);
  color: var(--color-text-3);
  gap: 8px;
  padding: 16px;
  text-align: center;
}

.qrcode-placeholder p {
  margin: 0;
  font-size: 13px;
}

.qrcode-placeholder .hint {
  font-size: 11px;
  color: var(--color-text-3);
  opacity: 0.7;
}

.payment-desc {
  margin: 0;
  font-size: 14px;
  color: var(--color-text-2);
}

.link-section {
  display: flex;
  justify-content: center;
}

.link-card {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 12px;
  padding: 32px 24px;
  background-color: var(--color-surface);
  border-radius: 16px;
  box-shadow: var(--shadow-sm);
  text-align: center;
  width: 100%;
  max-width: 320px;
}

.link-card h3 {
  margin: 0;
  font-size: 18px;
  font-weight: 600;
  color: var(--color-text);
}

.link-card p {
  margin: 0;
  font-size: 14px;
  color: var(--color-text-2);
}

.thanks-section {
  text-align: center;
  padding: 32px 16px;
}

.thanks-section p {
  margin: 0;
  font-size: 15px;
  color: var(--color-text-2);
  font-weight: 500;
}

.btn-icon {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  width: 40px;
  height: 40px;
  border: none;
  border-radius: 12px;
  background: transparent;
  color: var(--color-text);
  cursor: pointer;
  transition: all 0.2s ease;
}

.btn-icon:hover {
  background-color: var(--color-surface-variant);
}

.btn-icon:active {
  transform: scale(0.95);
}
</style>