<template>
  <UButton
    :type="type"
    :color="color"
    :loading="loading"
    :size="size"
    :disabled="disabled || loading"
    class="dark:text-gray-200 dark:bg-primary-300 bg-primary-400 disabled:bg-primary-200"
  >
    <slot />
  </UButton>
</template>

<script setup lang="ts">
defineProps({
  type: {
    type: String as PropType<"button" | "submit" | "reset" | "link">,
    default: "button",
  },
  color: {
    type: String as PropType<
      | "primary"
      | "error"
      | "secondary"
      | "success"
      | "info"
      | "warning"
      | "neutral"
    >,
    default: "primary",
  },
  loading: {
    type: Boolean,
    default: false,
  },
  disabled: {
    type: Boolean,
    default: false,
  },
  error: {
    type: Boolean,
    default: false,
  },
  size: {
    type: String as PropType<"xs" | "sm" | "md" | "lg" | "xl">,
    default: "sm",
  },
});
</script>
