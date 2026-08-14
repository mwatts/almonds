<script setup lang="ts">
import { useAppStore } from "~/stores/app";

const props = withDefaults(
  defineProps<{
    collapsed?: boolean;
    variant?: "default" | "light";
    size?: "sm" | "md";
  }>(),
  { collapsed: false, variant: "default", size: "sm" },
);

const appStore = useAppStore();

const appName = computed(() => appStore.config?.appName || "Lunar");
const logoUrl = computed(() => appStore.config?.logoUrl || null);

onMounted(() => {
  if (!appStore.config) appStore.fetchConfig();
});

const boxClass = computed(() => {
  const sizeClass = props.size === "md" ? "size-12" : "size-8";
  const bgClass =
    props.variant === "light" ? "bg-white/20" : "bg-gray-100 dark:bg-white/5";
  return `${sizeClass} ${bgClass}`;
});

const iconClass = computed(() => {
  const sizeClass = props.size === "md" ? "size-6" : "size-4.5";
  const colorClass =
    props.variant === "light"
      ? "text-white"
      : "text-gray-400 dark:text-white/25";
  return `${sizeClass} ${colorClass}`;
});

const textClass = computed(() => {
  const sizeClass = props.size === "md" ? "text-lg" : "text-sm";
  const colorClass =
    props.variant === "light" ? "text-white" : "text-gray-900 dark:text-white";
  return `${sizeClass} ${colorClass}`;
});
</script>

<template>
  <div class="flex items-center gap-2.5 overflow-hidden">
    <div
      v-if="logoUrl"
      class="rounded-lg overflow-hidden shrink-0"
      :class="boxClass"
    >
      <img :src="logoUrl" :alt="appName" class="w-full h-full object-cover" />
    </div>
    <div
      v-else
      class="rounded-lg flex items-center justify-center shrink-0"
      :class="boxClass"
    >
      <UIcon
        name="heroicons:building-office-2"
        class="shrink-0"
        :class="iconClass"
      />
    </div>
    <span
      v-if="!collapsed"
      class="font-semibold whitespace-nowrap truncate"
      :class="textClass"
    >
      {{ appName }}
    </span>
  </div>
</template>
