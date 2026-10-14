import { shallowRef } from "vue";

export type TimerHandle = number;

const announcement = shallowRef("");
let activeTimer: TimerHandle | undefined;

export function useAnnouncer() {
  function announce(message: string) {
    if (!message) return;
    clearTimeout(activeTimer);
    announcement.value = "";
    activeTimer = window.setTimeout(() => {
      announcement.value = message;
    }, 50);
  }
  return {
    announcement,
    announce,
  };
}
