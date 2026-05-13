<script setup lang="ts">
import Icon from "../components/Icon.vue";

function goBack() {
  window.dispatchEvent(new CustomEvent('navigateHome'));
}
</script>

<template>
  <div class="user-agreement">
    <var-app-bar class="floating-app-bar" title="用户协议">
      <template #left>
        <button class="btn-icon" @click="goBack">
          <Icon name="arrow-left" :size="24" />
        </button>
      </template>
    </var-app-bar>
    
    <div class="content">
      <div class="agreement-content">
        <h1>用户服务协议</h1>
        
        <div class="section">
          <h2>1. 协议的接受</h2>
          <p>欢迎使用咪萌（以下简称“本应用”）。本应用由个人开发者开发和维护。</p>
          <p>通过安装、使用、访问本应用，您即表示同意接受本协议的条款和条件。如果您不同意本协议的条款，请不要使用本应用。</p>
        </div>
        
        <div class="section">
          <h2>2. 知识产权声明</h2>
          <p>本应用的所有内容，包括但不限于软件、代码、文字、图像、音频、视频等，均受中华人民共和国著作权法、商标法和国际公约的保护。</p>
          <p>本应用的知识产权归开发者所有。您可以免费安装和使用本应用，但不得进行反编译、修改、分发或用于商业用途。</p>
        </div>
        
        <div class="section">
          <h2>3. 用户使用规范</h2>
          <p>您承诺仅将本应用用于合法目的，遵守所有适用的法律法规。</p>
          <p>您不得使用本应用从事任何违法活动，包括但不限于传播有害信息、侵犯他人权益等。</p>
        </div>
        
        <div class="section">
          <h2>4. 免责声明</h2>
          <p>本应用按“原样”提供，开发者不提供任何明示或暗示的保证，包括但不限于适销性、特定用途适用性和非侵权性的保证。</p>
          <p>开发者对因使用本应用而产生的任何直接或间接损失不承担责任。</p>
        </div>
        
        <div class="section">
          <h2>5. 隐私政策</h2>
          <p>本应用尊重并保护您的隐私。本应用仅在本地存储您的表情包数据，不会收集、传输或共享您的个人信息。</p>
          <p>详细信息请查看<a @click="$emit('open-privacy-policy')" class="link">隐私政策</a>。</p>
        </div>
        
        <div class="section">
          <h2>6. 协议的修改</h2>
          <p>开发者有权随时修改本协议的条款。修改后的协议将在本应用内发布，您继续使用本应用即表示接受修改后的协议。</p>
        </div>
        
        <div class="section">
          <h2>7. 法律适用和管辖</h2>
          <p>本协议的解释和适用均适用中华人民共和国法律。如发生争议，双方应协商解决；协商不成的，提交开发者所在地人民法院诉讼解决。</p>
        </div>
        
        <div class="section">
          <h2>8. 联系方式</h2>
          <p>如果您对本协议有任何疑问，请通过以下方式联系开发者：</p>
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
.user-agreement {
  height: 100%;
  display: flex;
  flex-direction: column;
}

.content {
  flex: 1;
  overflow-y: auto;
  padding: 16px;
}

.agreement-content {
  max-width: 800px;
  margin: 0 auto;
  line-height: 1.8;
}

.agreement-content h1 {
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