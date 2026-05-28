<template>
  <div class="settings-section">
    <h2>
      <Icon name="information" :size="24" />
      关于
    </h2>

    <div class="setting-row" @click="openUserAgreement">
      <Icon name="file-text" :size="22" class="setting-icon" />
      <div class="setting-label">
        <label>用户协议</label>
        <p class="setting-desc">查看用户使用协议</p>
      </div>
      <Icon name="chevron-right" :size="20" class="arrow-icon" />
    </div>

    <div class="setting-row" @click="openPrivacyPolicy">
      <Icon name="shield" :size="22" class="setting-icon" />
      <div class="setting-label">
        <label>隐私政策</label>
        <p class="setting-desc">查看隐私保护政策</p>
      </div>
      <Icon name="chevron-right" :size="20" class="arrow-icon" />
    </div>

    <div class="setting-row" @click="openGitHubRepo">
      <Icon name="github" :size="22" class="setting-icon" />
      <div class="setting-label">
        <label>开源地址</label>
        <p class="setting-desc">Web 版已开源</p>
      </div>
      <Icon name="chevron-right" :size="20" class="arrow-icon" />
    </div>

    <div class="setting-row" @click="goSupport">
      <Icon name="heart" :size="22" class="setting-icon" />
      <div class="setting-label">
        <label>支持一下</label>
        <p class="setting-desc">喜欢本应用吗? 豪我一下吧</p>
      </div>
      <Icon name="chevron-right" :size="20" class="arrow-icon" />
    </div>

    <div class="card about-card">
      <div class="about-content">
        <div class="app-logo">≧▽≦</div>
        <p class="version">v{{ appVersion }}</p>
        <p class="description">本地表情包管理和分享工具</p>
        <div class="platform-info">
          <p>支持平台：Linux (x86_64) / Android (aarch64)</p>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted } from 'vue';
import Icon from '../components/Icon.vue';
import { invoke } from '../utils/tauri';

const appVersion = ref('0.0.0');

onMounted(async () => {
  try {
    const version = await invoke('get_version');
    appVersion.value = version as string;
  } catch { /* ignore */ }
});

function openUserAgreement() {
  window.dispatchEvent(new CustomEvent('navigateToMenu', { detail: 'user-agreement' }));
}

function openPrivacyPolicy() {
  window.dispatchEvent(new CustomEvent('navigateToMenu', { detail: 'privacy-policy' }));
}

function openGitHubRepo() {
  window.open('https://github.com/your-repo/meme', '_blank');
}

function goSupport() {
  window.dispatchEvent(new CustomEvent('navigateToMenu', { detail: 'support' }));
}
</script>

<style scoped>
.settings-section h2 {
  display: flex;
  align-items: center;
  gap: 10px;
  margin: 0 0 16px 0;
  font-size: 18px;
  color: var(--color-text);
}

.setting-row {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 16px;
  margin-bottom: 8px;
  border-radius: 12px;
  background: var(--color-surface);
  box-shadow: var(--shadow-sm);
  cursor: pointer;
  transition: background-color 0.2s ease, box-shadow 0.2s ease;
}

.setting-row:hover {
  background: var(--color-surface-variant);
  box-shadow: var(--shadow-md);
}

.setting-icon {
  flex-shrink: 0;
  margin-right: 12px;
  color: var(--color-text-secondary);
}

.setting-label {
  flex: 1;
  min-width: 200px;
}

.setting-label label {
  display: block;
  font-weight: 600;
  font-size: 14px;
  color: var(--color-text);
  margin-bottom: 4px;
}

.setting-desc {
  font-size: 12px;
  color: var(--color-text-3);
  margin: 0;
}

.arrow-icon {
  color: var(--color-text-secondary);
}

.card {
  background: var(--color-surface);
  border-radius: var(--radius-lg);
  box-shadow: var(--shadow-sm);
  overflow: hidden;
}

.about-card {
  margin: 16px;
  padding: 24px;
  border-radius: 16px;
  background: var(--color-surface-variant);
}

.about-content {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 8px;
}

.app-logo {
  font-size: 48px;
  line-height: 1;
  color: var(--color-text);
}

.version {
  font-size: 16px;
  font-weight: 600;
  color: var(--color-text);
}

.description {
  font-size: 14px;
  color: var(--color-text-2);
  margin: 0;
}

.platform-info {
  text-align: center;
  font-size: 12px;
  color: var(--color-text-3);
}

.platform-info p {
  margin: 2px 0;
}
</style>
