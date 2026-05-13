<script setup lang="ts">
import Icon from "../components/Icon.vue";

function goBack() {
  window.dispatchEvent(new CustomEvent('navigateHome'));
}
</script>

<template>
  <div class="privacy-policy">
    <var-app-bar class="floating-app-bar" title="隐私政策">
      <template #left>
        <button class="btn-icon" @click="goBack">
          <Icon name="arrow-left" :size="24" />
        </button>
      </template>
    </var-app-bar>
    
    <div class="content">
      <div class="policy-content">
        <h1>隐私政策</h1>
        
        <div class="section">
          <h2>1. 引言</h2>
          <p>咪萌（以下简称“本应用”）尊重并保护您的隐私。本政策旨在说明本应用如何收集、使用、存储和保护您的信息。</p>
          <p>本应用仅在本地设备上处理您的数据，不会将您的个人信息传输到任何第三方服务器。</p>
        </div>
        
        <div class="section">
          <h2>2. 信息收集</h2>
          <p>本应用不会收集您的任何个人身份信息，如姓名、邮箱地址、电话号码等。</p>
          <p>本应用仅在本地存储您选择的表情包目录路径和应用设置，这些信息仅用于提供更好的用户体验。</p>
        </div>
        
        <div class="section">
          <h2>3. 信息使用</h2>
          <p>本应用仅使用您提供的信息来：</p>
          <ul>
            <li>管理和显示您的表情包</li>
            <li>保存您的应用设置偏好</li>
            <li>提供更好的用户体验</li>
          </ul>
        </div>
        
        <div class="section">
          <h2>4. 信息存储</h2>
          <p>所有数据均存储在您的本地设备上，本应用不会将您的数据上传到任何云端服务器。</p>
          <p>您可以随时卸载本应用，卸载后所有相关数据将被删除。</p>
        </div>
        
        <div class="section">
          <h2>5. 信息共享</h2>
          <p>本应用不会与任何第三方共享您的信息，包括但不限于广告商、合作伙伴或其他服务提供商。</p>
          <p>本应用不会收集或传输您的表情包内容到任何外部服务器。</p>
        </div>
        
        <div class="section">
          <h2>6. 安全措施</h2>
          <p>本应用采取合理的安全措施保护您的数据免受未经授权的访问、使用或披露。</p>
          <p>由于本应用仅在本地存储数据，数据安全完全取决于您的设备安全措施。</p>
        </div>
        
        <div class="section">
          <h2>7. 儿童隐私</h2>
          <p>本应用不面向13岁以下的儿童。如果您是13岁以下的儿童，请不要使用本应用。</p>
        </div>
        
        <div class="section">
          <h2>8. 政策修改</h2>
          <p>开发者有权随时修改本隐私政策。修改后的政策将在本应用内发布，您继续使用本应用即表示接受修改后的政策。</p>
        </div>
        
        <div class="section">
          <h2>9. 联系方式</h2>
          <p>如果您对本隐私政策有任何疑问，请通过以下方式联系开发者：</p>
          <p>邮箱：AlexIllinois@163.com</p>
        </div>
        
        <div class="last-updated">
          <p>最后更新日期：2026年5月13日</p>
        </div>
      </div>
    </div>
  </div>
</template>

<style scoped>
.privacy-policy {
  height: 100%;
  display: flex;
  flex-direction: column;
}

.content {
  flex: 1;
  overflow-y: auto;
  padding: 16px;
}

.policy-content {
  max-width: 800px;
  margin: 0 auto;
  line-height: 1.8;
}

.policy-content h1 {
  text-align: center;
  font-size: 24px;
  margin-bottom: 32px;
  color: var(--text-color);
}

.section {
  margin-bottom: 24px;
  padding-bottom: 16px;
  border-bottom: 1px solid var(--border-color);
}

.section:last-child {
  border-bottom: none;
}

.section h2 {
  font-size: 18px;
  margin-bottom: 12px;
  color: var(--text-color);
}

.section p {
  font-size: 14px;
  margin-bottom: 8px;
  color: var(--text-secondary);
}

.section ul {
  font-size: 14px;
  margin-left: 24px;
  margin-bottom: 8px;
  color: var(--text-secondary);
}

.section li {
  margin-bottom: 4px;
}

.last-updated {
  margin-top: 32px;
  text-align: center;
  font-size: 12px;
  color: var(--text-tertiary);
}
</style>