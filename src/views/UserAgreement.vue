<script setup lang="ts">
import Icon from "../components/Icon.vue";

function goBack() {
  window.dispatchEvent(new CustomEvent('navigateHome'));
}
</script>

<template>
  <div class="user-agreement">
    <var-app-bar class="floating-app-bar" title="用户服务协议">
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
          <h2>一、定义</h2>
          <p>1.1 "本服务"：指由开发者提供的表情包管理工具，包括Web版本（开源）和Tauri版本（闭源）。</p>
          <p>1.2 "用户"：指安装、使用本服务的自然人、法人或其他组织。</p>
          <p>1.3 "内容"：指用户通过本服务上传、存储、管理的表情包图片及相关数据。</p>
        </div>
        
        <div class="section">
          <h2>二、账号注册与使用</h2>
          <p>2.1 本服务无需注册账号即可使用，用户通过本地设备直接访问和管理个人数据。</p>
          <p>2.2 用户应确保使用本服务的设备符合系统要求，并对设备的安全性负责。</p>
          <p>2.3 如用户选择使用云端同步功能（若提供），需提供有效联系方式进行注册，并对账号信息的真实性、安全性负责。</p>
          <p>2.4 账号注销：用户可通过清除本地数据或卸载应用的方式终止使用本服务，所有本地数据将被永久删除。</p>
        </div>
        
        <div class="section">
          <h2>三、用户行为规范</h2>
          <p>3.1 用户承诺遵守中华人民共和国法律法规，不得利用本服务从事任何违法活动。</p>
          <p>3.2 用户不得上传、存储、传播含有以下内容的表情包：</p>
          <p>（一）违反宪法确定的基本原则的；</p>
          <p>（二）危害国家安全、泄露国家秘密、颠覆国家政权、破坏国家统一的；</p>
          <p>（三）损害国家荣誉和利益的；</p>
          <p>（四）歪曲、丑化、亵渎、否定英雄烈士事迹和精神，以侮辱、诽谤或者其他方式侵害英雄烈士的姓名、肖像、名誉、荣誉的；</p>
          <p>（五）宣扬恐怖主义、极端主义或者煽动实施恐怖活动、极端主义活动的；</p>
          <p>（六）煽动民族仇恨、民族歧视，破坏民族团结的；</p>
          <p>（七）破坏国家宗教政策，宣扬邪教和封建迷信的；</p>
          <p>（八）散布谣言，扰乱经济秩序和社会秩序的；</p>
          <p>（九）散布淫秽、色情、赌博、暴力、凶杀、恐怖或者教唆犯罪的；</p>
          <p>（十）侮辱或者诽谤他人，侵害他人合法权益的；</p>
          <p>（十一）含有法律、行政法规禁止的其他内容的。</p>
          <p>3.3 用户不得对本服务进行反向工程、反编译、破解、修改或制作衍生作品。</p>
          <p>3.4 用户不得利用本服务侵犯他人知识产权、隐私权或其他合法权益。</p>
        </div>
        
        <div class="section">
          <h2>四、平台权利义务</h2>
          <p>4.1 平台权利：</p>
          <p>（一）对于违反本协议的用户，平台有权采取警告、限制功能使用、封禁设备等措施；</p>
          <p>（二）对于用户上传的违法内容，平台有权采取删除、屏蔽等措施，并保留向有关部门报告的权利；</p>
          <p>（三）有权根据业务需要对服务功能进行调整、更新或终止。</p>
          <p>4.2 平台义务：</p>
          <p>（一）按照本协议约定向用户提供稳定的服务；</p>
          <p>（二）采取合理技术措施保护用户本地数据的安全；</p>
          <p>（三）尊重用户隐私，不收集、不传输用户的个人信息和内容数据。</p>
        </div>
        
        <div class="section">
          <h2>五、知识产权</h2>
          <p>5.1 本服务的知识产权归开发者所有，包括但不限于软件著作权、商标权、专利权等。</p>
          <p>5.2 Web版本基于MIT许可证开源，源代码地址：<a href="https://github.com/AlexIllinois2/meme_web" target="_blank" class="link">https://github.com/AlexIllinois2/meme_web</a></p>
          <p>5.3 Tauri版本暂时闭源，用户可免费用于个人非商业用途，禁止用于商业目的。</p>
          <p>5.4 用户上传的内容知识产权归用户所有，用户应确保拥有合法的使用权或所有权。</p>
        </div>
        
        <div class="section">
          <h2>六、免责声明</h2>
          <p>6.1 本服务按"原样"提供，开发者不提供任何明示或暗示的保证，包括但不限于适销性、特定用途适用性的保证。</p>
          <p>6.2 开发者对因使用本服务而产生的任何直接或间接损失不承担责任，包括但不限于数据丢失、设备损坏、业务中断等。</p>
          <p>6.3 对于因不可抗力、网络故障、第三方服务中断等非开发者原因导致的服务异常或终止，开发者不承担责任。</p>
        </div>
        
        <div class="section">
          <h2>七、协议变更与终止</h2>
          <p>7.1 开发者有权根据法律法规变化或业务需要对本协议进行修改，修改后的协议将在应用内发布。</p>
          <p>7.2 用户继续使用本服务即视为接受修改后的协议，如有异议应立即停止使用。</p>
          <p>7.3 用户可随时通过卸载应用的方式终止使用本服务。</p>
          <p>7.4 对于违反本协议的用户，开发者有权终止其使用本服务的权利。</p>
        </div>
        
        <div class="section">
          <h2>八、未成年人使用条款</h2>
          <p>8.1 未成年人应在监护人的指导下使用本服务。</p>
          <p>8.2 监护人应履行监护职责，保护未成年人的合法权益，避免接触不适宜的内容。</p>
          <p>8.3 开发者不主动收集未成年人的个人信息，如发现未成年人使用服务，将采取措施保护其权益。</p>
        </div>
        
        <div class="section">
          <h2>九、争议解决</h2>
          <p>9.1 本协议的解释和适用均适用中华人民共和国法律。</p>
          <p>9.2 如发生争议，双方应协商解决；协商不成的，提交开发者所在地人民法院诉讼解决。</p>
        </div>
        
        <div class="section">
          <h2>十、联系方式</h2>
          <p>如对本协议有任何疑问，请通过以下方式联系开发者：</p>
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