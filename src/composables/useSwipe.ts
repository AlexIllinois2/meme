import { ref, nextTick, Ref } from 'vue';
import type { Group } from '../types';

export function useSwipe(
  deps: {
    groups: Ref<Group[]>;
    selectedGroupId: Ref<number | null>;
    switchGroup: (id: number) => Promise<void>;
    isGlobalEditMode: Ref<boolean>;
  },
) {
  const swipeStartX = ref(0);
  const swipeStartY = ref(0);
  const swipeOffset = ref(0);
  const isSwiping = ref(false);

  function handleSwipeStart(event: TouchEvent) {
    if (deps.isGlobalEditMode.value || event.touches.length !== 1) return;
    swipeStartX.value = event.touches[0].clientX;
    swipeStartY.value = event.touches[0].clientY;
    swipeOffset.value = 0;
    isSwiping.value = true;
  }

  function handleSwipeMove(event: TouchEvent) {
    if (!isSwiping.value || deps.isGlobalEditMode.value) return;
    if (event.touches.length !== 1) {
      isSwiping.value = false;
      swipeOffset.value = 0;
      return;
    }
    const deltaX = event.touches[0].clientX - swipeStartX.value;
    const deltaY = event.touches[0].clientY - swipeStartY.value;
    if (Math.abs(deltaY) > Math.abs(deltaX) && Math.abs(deltaY) > 10) {
      isSwiping.value = false;
      swipeOffset.value = 0;
      return;
    }
    if (Math.abs(deltaX) > 10) {
      event.preventDefault();
      swipeOffset.value = deltaX;
    }
  }

  function handleSwipeEnd() {
    if (!isSwiping.value) return;
    const threshold = 80;
    if (Math.abs(swipeOffset.value) < threshold) {
      isSwiping.value = false;
      swipeOffset.value = 0;
      return;
    }
    const currentIndex = deps.groups.value.findIndex(g => g.id === deps.selectedGroupId.value);
    if (currentIndex === -1) {
      isSwiping.value = false;
      swipeOffset.value = 0;
      return;
    }
    if (swipeOffset.value < -threshold && currentIndex < deps.groups.value.length - 1) {
      performSwipeTransition(() => deps.switchGroup(deps.groups.value[currentIndex + 1].id));
    } else if (swipeOffset.value > threshold && currentIndex > 0) {
      performSwipeTransition(() => deps.switchGroup(deps.groups.value[currentIndex - 1].id));
    } else {
      isSwiping.value = false;
      swipeOffset.value = 0;
    }
  }

  async function performSwipeTransition(onComplete: () => Promise<void> | void) {
    await onComplete();
    swipeOffset.value = 0;
    await nextTick();
    await new Promise<void>(resolve => requestAnimationFrame(() => resolve()));
    isSwiping.value = false;
  }

  return {
    swipeStartX,
    swipeStartY,
    swipeOffset,
    isSwiping,
    handleSwipeStart,
    handleSwipeMove,
    handleSwipeEnd,
  };
}
