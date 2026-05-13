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
          <h2>一、运营主体信息</h2>
          <p>运营主体：个人开发者</p>
          <p>联系人：AlexIllinois</p>
          <p>联系方式：AlexIllinois@163.com</p>
        </div>
        
        <div class="section">
          <h2>二、个人信息的收集</h2>
          <p>2.1 本服务为本地应用，**不收集、不传输任何用户个人信息**。</p>
          <p>2.2 用户所有操作均在本地设备完成，包括表情包的上传、存储、管理等。</p>
          <p>2.3 本服务不会请求或获取以下权限：</p>
          <p>（一）手机号、通讯录权限；</p>
          <p>（二）设备定位权限；</p>
          <p>（三）麦克风、摄像头权限；</p>
          <p>（四）短信、通话记录权限；</p>
          <p>（五）其他涉及个人信息的系统权限。</p>
        </div>
        
        <div class="section">
          <h2>三、个人信息的使用</h2>
          <p>3.1 本服务仅在本地设备上处理用户数据，用于实现表情包管理功能。</p>
          <p>3.2 所有数据处理均在用户设备内完成，不会向任何第三方传输。</p>
        </div>
        
        <div class="section">
          <h2>四、个人信息的存储</h2>
          <p>4.1 用户所有数据均仅存储在本地设备中，开发者无法访问。</p>
          <p>4.2 数据存储期限由用户自行决定，用户可随时删除或清除数据。</p>
          <p>4.3 卸载应用后，所有本地数据将被永久删除。</p>
        </div>
        
        <div class="section">
          <h2>五、第三方SDK共享</h2>
          <p>5.1 本服务**不集成任何第三方SDK**，不存在数据共享情况。</p>
          <p>5.2 本服务不包含广告、统计、支付等第三方服务。</p>
        </div>
        
        <div class="section">
          <h2>六、用户权利</h2>
          <p>6.1 查阅权：用户可随时在本地设备上查看所有数据。</p>
          <p>6.2 更正权：用户可随时修改、编辑本地数据。</p>
          <p>6.3 删除权：用户可随时删除不需要的表情包或数据。</p>
          <p>6.4 注销权：用户可通过卸载应用的方式永久清除所有数据。</p>
        </div>
        
        <div class="section">
          <h2>七、未成年人保护</h2>
          <p>7.1 本服务不主动收集未成年人的任何个人信息。</p>
          <p>7.2 未成年人使用本服务应在监护人指导下进行。</p>
          <p>7.3 如发现未成年人使用服务，开发者将采取措施保护其权益。</p>
        </div>
        
        <div class="section">
          <h2>八、信息安全</h2>
          <p>8.1 开发者采取合理技术措施保护本地数据的安全性。</p>
          <p>8.2 用户数据的安全主要取决于设备的安全防护措施。</p>
          <p>8.3 用户应注意设备安全，避免数据丢失或泄露。</p>
        </div>
        
        <div class="section">
          <h2>九、跨境传输</h2>
          <p>9.1 本服务**不进行任何数据的跨境传输**，所有数据均仅存储在本地设备。</p>
        </div>
        
        <div class="section">
          <h2>十、隐私政策的变更</h2>
          <p>10.1 开发者有权根据法律法规变化或业务需要修改本政策。</p>
          <p>10.2 修改后的政策将在应用内发布，用户继续使用服务即视为接受修改。</p>
        </div>
        
        <div class="section">
          <h2>十一、联系方式</h2>
          <p>如对本隐私政策有任何疑问，请通过以下方式联系：</p>
          <p>邮箱：AlexIllinois@163.com</p>
        </div>
        
        <div class="section">
          <h2>十二、版本说明</h2>
          <p>本应用分为两个版本：</p>
          <p>1. Web版本：已开源，基于MIT许可证，源代码地址：<a href="https://github.com/AlexIllinois2/meme_web" target="_blank" class="link">https://github.com/AlexIllinois2/meme_web</a></p>
          <p>2. Tauri桌面版本：暂时闭源，免费供个人非商业用途使用</p>
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

.link {
  color: var(--color-primary);
  cursor: pointer;
}

.link:hover {
  text-decoration: underline;
}

.last-updated {
  margin-top: 32px;
  text-align: center;
  font-size: 12px;
  color: var(--text-tertiary);
}
</style>