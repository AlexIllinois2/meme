/**
 * useVisibility — 窗口可见性 composable
 *
 * 跟踪桌面窗口焦点 + 页面可见性，非活动时暂停 CSS 动画以释放 CPU。
 * 移动端（Android）始终返回 isActive=true，不生效。
 */
import { ref, onMounted, onUnmounted } from "vue";

export function useVisibility() {

  const isActive = ref(true);
  // 移动端始终视为活跃
  const isMobile = /android/i.test(navigator.userAgent);

  function update() {
    const now = isMobile || (!document.hidden && document.hasFocus());
    if (now !== isActive.value) {
      isActive.value = now;
      if (!isMobile) {
        document.documentElement.classList.toggle("window-inactive", !now);
      }
    }
  }

  onMounted(() => {
    // 先同步一次初始状态
    update();
    window.addEventListener("focus", update);
    window.addEventListener("blur", update);
    document.addEventListener("visibilitychange", update);
  });

  onUnmounted(() => {
    window.removeEventListener("focus", update);
    window.removeEventListener("blur", update);
    document.removeEventListener("visibilitychange", update);
    if (!isMobile) {
      document.documentElement.classList.remove("window-inactive");
    }
  });

  return { isActive };
}
