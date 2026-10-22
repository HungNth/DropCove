<script setup lang="ts">
import { useMediaQuery } from "@vueuse/core";
import { Check, Monitor, Moon, Sun } from "@lucide/vue";
import type { AppearanceMode } from "../../composables/useSettings";

const props = defineProps<{ mode: AppearanceMode; disabled: boolean }>();
const emit = defineEmits<{ change: [mode: AppearanceMode, event: Event] }>();
const isContrast = useMediaQuery("(forced-colors: active)");
const options = [
  { value: "System", description: "Use the Windows app theme", icon: Monitor },
  { value: "Dark", description: "Always use dark appearance", icon: Moon },
  { value: "Light", description: "Always use light appearance", icon: Sun },
] as const;

function selectMode(mode: AppearanceMode, event: Event) {
  if (!props.disabled && mode !== props.mode) emit("change", mode, event);
}
</script>

<template>
  <section class="flex flex-col gap-3" aria-labelledby="appearance-heading">
    <div class="flex flex-col gap-1">
      <h2 id="appearance-heading" class="m-0 text-[13px] font-semibold text-dc-text-primary">Appearance mode</h2>
      <p class="m-0 text-[12px] text-dc-text-muted leading-relaxed">Choose how DropCove looks.</p>
    </div>
    <div role="radiogroup" aria-labelledby="appearance-heading" :aria-describedby="isContrast ? 'contrast-theme-message' : undefined" class="grid grid-cols-3 gap-2">
      <label v-for="option in options" :key="option.value" class="group relative flex min-w-0 cursor-pointer flex-col gap-2 rounded-lg has-disabled:cursor-default has-disabled:opacity-[var(--dc-disabled-opacity)]">
        <input
          type="radio" name="appearance-mode" :value="option.value" :checked="mode === option.value"
          :disabled="disabled" :aria-label="option.value" :aria-describedby="`appearance-${option.value}-description`"
          class="peer sr-only" @change="selectMode(option.value, $event)"
        />
        <span class="relative flex h-[86px] flex-col rounded-lg border border-dc-border-default peer-checked:border-dc-accent peer-checked:shadow-[0_0_0_1px_var(--dc-accent)] peer-focus-visible:outline-2 peer-focus-visible:outline-offset-2 peer-focus-visible:outline-dc-focus group-hover:border-dc-border-hover forced-colors:border-[ButtonText] forced-colors:peer-checked:border-[Highlight] forced-colors:peer-focus-visible:outline-[Highlight]" aria-hidden="true">
          <span :data-appearance="option.value === 'System' ? undefined : option.value.toLowerCase()" class="flex h-full flex-col justify-center gap-1.5 rounded-[7px] bg-dc-surface-elevated p-3 forced-colors:bg-[Canvas]">
            <component :is="option.icon" class="size-4 text-dc-text-secondary" />
            <span class="flex items-center gap-1.5 rounded border border-dc-border-subtle bg-dc-card px-1.5 py-1">
              <span class="size-2 rounded-xs bg-dc-text-muted/40"></span>
              <span class="h-1 w-8 rounded bg-dc-text-muted/40"></span>
            </span>
            <span class="h-1 w-10 rounded bg-dc-text-muted/25"></span>
          </span>
          <span v-if="mode === option.value" class="absolute right-1.5 top-1.5 inline-flex size-4 items-center justify-center rounded-full bg-dc-action text-dc-accent-text forced-colors:bg-[Highlight] forced-colors:text-[HighlightText]">
            <Check class="size-2.5" :stroke-width="3" />
          </span>
        </span>
        <span class="text-[12px] font-medium text-dc-text-primary">{{ option.value }}</span>
        <span :id="`appearance-${option.value}-description`" class="text-[11px] leading-snug text-dc-text-muted">{{ option.description }}</span>
      </label>
    </div>
    <p v-if="isContrast" id="contrast-theme-message" role="note" class="m-0 mt-1 rounded-md border border-dc-border-default p-2.5 text-[12px] leading-relaxed text-dc-text-secondary forced-colors:border-[ButtonText] forced-colors:text-[CanvasText]">
      Windows Contrast Theme currently overrides this appearance.
    </p>
  </section>
</template>
