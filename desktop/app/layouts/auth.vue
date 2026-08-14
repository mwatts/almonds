<template>
  <div
    class="min-h-screen w-full bg-gray-50 dark:bg-surface-950 flex items-center justify-center p-6 relative overflow-hidden"
  >
    <div
      class="pointer-events-none absolute -top-24 -left-24 size-96 rounded-full bg-accent-300/20 dark:bg-accent-500/10 blur-3xl"
    />
    <div
      class="pointer-events-none absolute -bottom-24 -right-24 size-96 rounded-full bg-violet-300/20 dark:bg-violet-500/10 blur-3xl"
    />

    <div
      class="relative w-full max-w-md bg-white dark:bg-gray-800 rounded-2xl border border-gray-100 dark:border-gray-700 shadow-xl p-8"
    >
      <div class="flex flex-col items-center gap-2 mb-6">
        <div
          class="size-11 rounded-xl bg-accent-500 flex items-center justify-center shadow-lg shadow-accent-500/25"
        >
          <UIcon name="heroicons:bolt-solid" class="size-6 text-white" />
        </div>
        <h1 class="text-xl font-semibold text-gray-800 dark:text-gray-100">
          Almonds
        </h1>
      </div>

      <slot />
    </div>
  </div>
</template>

<script lang="ts" setup></script>
