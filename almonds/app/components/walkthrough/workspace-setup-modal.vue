<script setup lang="ts">
import { useWorkspacesStore } from "~/stores/workspaces";

const store = useWorkspacesStore();

const form = reactive({
  name: "",
  description: "",
});

const loading = ref(false);
const errors = reactive({
  name: "",
  description: "",
});

function validate(): boolean {
  errors.name = form.name.trim() ? "" : "Name is required";
  errors.description = form.description.trim() ? "" : "Description is required";
  return !errors.name && !errors.description;
}

async function handleSubmit() {
  if (!validate()) return;
  loading.value = true;
  try {
    await store.createWorkspace({
      name: form.name.trim() || "default",
      description: form.description.trim() || "default",
    });
  } finally {
    loading.value = false;
  }
}
</script>

<template>
  <UModal
    :open="true"
    :close="false"
    :dismissible="false"
    @update:open="() => {}"
  >
    <template #header>
      <div class="flex flex-col gap-1">
        <h2 class="text-lg font-semibold text-gray-900 dark:text-white">
          Welcome to Almonds
        </h2>
        <p class="text-sm text-gray-500 dark:text-gray-400">
          Create your first workspace to get started. You can have multiple
          workspaces for different projects or contexts, and switch between them
          seamlessly.
        </p>
      </div>
    </template>

    <template #body>
      <AppCreateWorkspaceForm />
    </template>
  </UModal>
</template>
