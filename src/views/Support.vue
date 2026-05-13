<script setup lang="ts">
import { ref, onMounted, onUnmounted } from 'vue';
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
  window.dispatchEvent(new CustomEvent('navigateHome'));
}

function openLink(url: string) {
  if (typeof window !== 'undefined') {
    window.open(url, '_blank');
  }
}

const isAndroidTauri = () => {
  return /Android/i.test(navigator.userAgent);
};

function handleAndroidBack(event: any) {
  goBack();
  event.preventDefault?.();
  return true;
}

onMounted(() => {
  if (isAndroidTauri()) {
    window.addEventListener('tauri-android-back', handleAndroidBack);
  }
});

onUnmounted(() => {
  if (isAndroidTauri()) {
    window.removeEventListener('tauri-android-back', handleAndroidBack);
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
        <div class="heart-icon">❤️</div>
        <h2>感谢你的支持</h2>
        <p>你的每一份支持都是我持续开发的动力</p>
      </div>

      <div class="payment-section">
        <var-tabs v-model:active="activeTab" class="payment-tabs">
          <var-tab
            v-for="(method, index) in paymentMethods"
            :key="index"
            :name="method.name"
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
                <div v-if="method.qrcode" class="qrcode-wrapper">
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
                <p class="payment-desc">{{ method.desc }}</p>
              </div>

              <div v-else-if="method.type === 'link'" class="link-section">
                <div class="link-card" @click="openLink(method.url)">
                  <Icon name="external-link" :size="48" />
                  <h3>前往爱发电</h3>
                  <p>{{ method.desc }}</p>
                  <var-button type="primary" size="large" round>
                    立即支持
                  </var-button>
                </div>
              </div>
            </div>
          </var-tab-item>
        </var-tabs-items>
      </div>

      <div class="thanks-section">
        <p>✨ 感谢每一位支持者 ✨</p>
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