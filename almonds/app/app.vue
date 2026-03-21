<script setup lang="ts">
import {
  isPermissionGranted,
  requestPermission,
} from "@tauri-apps/plugin-notification";
import { useAlarmScheduler } from "~/composables/useAlarmScheduler";
import { useWorkspaceSetup } from "./composables/useWorkspaceSetup";
import type { DropdownMenuItem } from "@nuxt/ui";
import { reactive, ref, computed } from "vue";
import { useOnline } from "@vueuse/core";

const online = useOnline();
const workspaceStore = useWorkspacesStore();
const router = useRouter();
const colorMode = useColorMode();
const { searchConfig, searchQuery } = useAppSearch();

const isDark = computed({
  get: () => colorMode.value === "dark",
  set: (v) => (colorMode.preference = v ? "dark" : "light"),
});
const themeIcon = computed(() =>
  isDark.value ? "heroicons:sun" : "heroicons:moon",
);
const themeLabel = computed(() => (isDark.value ? "Light mode" : "Dark mode"));
const internetStatusColor = computed(() =>
  online.value ? "success" : "error",
);

const internetLabel = computed(() => (online.value ? "Online" : "Offline"));
const sidebarCollapsed = ref(false);
const asideOpen = ref(false);
const mobileNavOpen = ref(false);

function onSearchInput(val: string) {
  searchQuery.value = val;
  searchConfig.value?.searchFn?.(val);
}

const showCreateModal = ref(false);

const activeId = computed(() => workspaceStore.currentWorkspace?.identifier);

const workspaces = computed<DropdownMenuItem[]>(() => [
  ...workspaceStore.workspaces
    .filter((w): w is Workspace => !!w)
    .map((w) => {
      const isActive = w.identifier === activeId.value;
      return {
        label: w.name,
        value: w.identifier,
        icon: isActive
          ? "heroicons:check-circle-solid"
          : "heroicons:check-circle",
        class: isActive ? "font-semibold text-accent-500" : "",
        onSelect: () => workspaceStore.setActiveWorkspace(w.identifier),
      };
    }),
  {
    label: "Manage Workspaces",
    color: "neutral",
    icon: "ri:paint-brush-line",
    onSelect: () => navigateTo("/settings?section=workspaces"),
  },
  {
    label: "Add Workspace",
    color: "success",
    icon: "heroicons:plus",
    onSelect: () => (showCreateModal.value = true),
  },
]);

const form = reactive({ name: "", description: "" });
const loading = ref(false);
const errors = reactive({ name: "", description: "" });

function validate(): boolean {
  errors.name = form.name.trim() ? "" : "Name is required";
  errors.description = form.description.trim() ? "" : "Description is required";
  return !errors.name && !errors.description;
}

async function handleSubmit() {
  if (!validate()) return;
  loading.value = true;
  try {
    await workspaceStore.createWorkspace({
      name: form.name.trim(),
      description: form.description.trim(),
    });
    showCreateModal.value = false;
    form.name = "";
    form.description = "";
    errors.name = "";
    errors.description = "";
  } catch (e) {
    console.error(e);
  } finally {
    loading.value = false;
  }
}

const { init } = useAccentColor();
const { init: initFontSize } = useFontSize();
const { init: initDarkTheme } = useDarkTheme();
const { setupRequired, checkSetup, initializing } = useUserSetup();
const {
  setupRequired: workspaceSetupRequired,
  checkSetup: checkWorkspaceSetup,
  initializing: workspaceInitializing,
} = useWorkspaceSetup();

useAlarmScheduler();

onMounted(async () => {
  init();
  initFontSize();
  initDarkTheme();
  await checkSetup();
  await checkWorkspaceSetup();

  const workspaceStore = useWorkspacesStore();
  await workspaceStore.fetchWorkspaces();

  let permissionGranted = await isPermissionGranted();

  if (!permissionGranted) {
    const permission = await requestPermission();
    permissionGranted = permission === "granted";
  }
});

const isMacOS = ref(true);
</script>

<template>
  <UApp>
    <NuxtLayout>
      <NuxtPage />
    </NuxtLayout>
    <AppNotification />
    <UserSetupModal v-if="setupRequired" />
    <WorkspaceSetupModal v-if="workspaceSetupRequired" />

    <Transition
      enter-active-class="transition-opacity duration-200"
      leave-active-class="transition-opacity duration-300"
      enter-from-class="opacity-0"
      leave-to-class="opacity-0"
    >
      <AppSplashScreen v-if="initializing || workspaceInitializing" />
    </Transition>
  </UApp>

  <Script>
    import { getCurrentWindow } from '@tauri-apps/api/window'; const appWindow =
    getCurrentWindow(); document .getElementById('titlebar-minimize')
    ?.addEventListener('click', () => appWindow.minimize()); document
    .getElementById('titlebar-maximize') ?.addEventListener('click', () =>
    appWindow.toggleMaximize()); document .getElementById('titlebar-close')
    ?.addEventListener('click', () => appWindow.close());
  </Script>
  <Body>
  <UApp>
      <div class="titlebar grid grid-cls-12">
        <!-- mac os controls-->
        <div v-if="isMacOS" class="traffic-lights col-span-1">
          <span class="btn close"></span>
          <span class="btn minimize"></span>
          <span class="btn maximize"></span>
        </div>
  
        <!-- Back & forward button -->
        <div
          class="col-col-end-3 flex items-center justify-center gap-x-1.25 ml-10"
        >
          <UButton
            size="sm"
            color="neutral"
            variant="ghost"
            icon="heroicons:chevron-left"
            @click="router.back()"
          />
          <UButton
            size="sm"
            color="neutral"
            variant="ghost"
            icon="heroicons:chevron-right"
            @click="router.forward()"
          />
        </div>
  
        <!-- Search -->
        <div class="col-span-4 mx-auto">
          <input
            :value="searchQuery"
            :placeholder="searchConfig?.placeholder ?? 'Search...'"
            :disabled="!searchConfig"
            class="w-full bg-transparent outline-none text-sm text-gray-700 dark:text-gray-300 placeholder-gray-400"
            @input="onSearchInput(($event.target as HTMLInputElement).value)"
          />
  
          <!-- Right actions -->
        </div>
  
        <div class="flex items-center gap-1 ml-auto">
          <UTooltip :text="internetLabel">

            <UButton size="sm" :color="internetStatusColor" variant="ghost">
              <UBadge size="sm" class="size-2" :color="internetStatusColor" />
            </UButton>
          </UTooltip>
  
          <UButton size="sm" color="neutral" variant="ghost" />
  
          <UButton
            size="sm"
            color="neutral"
            variant="ghost"
            :icon="themeIcon"
            :aria-label="themeLabel"
            @click="isDark = !isDark"
          />

            <!-- <UTooltip text="Change workspaces"> -->
            <UButton
              size="sm"
              color="neutral"
              variant="ghost"
              icon="heroicons:briefcase"
              aria-label="Switch workspace"
            />
    
  
          <!-- Notifications -->
          <UButton
            size="sm"
            color="neutral"
            variant="ghost"
            icon="heroicons:bell"
            aria-label="Notifications"
            @click="navigateTo('/notifications')"
          />
  
          <!-- Right panel toggle (mobile only) -->
          <UButton
            class="flex md:hidden"
            size="sm"
            color="neutral"
            variant="ghost"
            icon="heroicons:bars-3-bottom-right"
            aria-label="Open panel"
          />
        </div>
  
        <div v-if="!isMacOS" class="controls">
          <button id="titlebar-minimize" title="minimize">
            <!-- https://api.iconify.design/mdi:window-minimize.svg -->
            <svg
              xmlns="http://www.w3.org/2000/svg"
              width="24"
              height="24"
              viewBox="0 0 24 24"
            >
              <path fill="currentColor" d="M19 13H5v-2h14z" />
            </svg>
          </button>
          <button id="titlebar-maximize" title="maximize">
            <!-- https://api.iconify.design/mdi:window-maximize.svg -->
            <svg
              xmlns="http://www.w3.org/2000/svg"
              width="24"
              height="24"
              viewBox="0 0 24 24"
            >
              <path fill="currentColor" d="M4 4h16v16H4zm2 4v10h12V8z" />
            </svg>
          </button>
          <button id="titlebar-close" title="close">
            <!-- https://api.iconify.design/mdi:close.svg -->
            <svg
              xmlns="http://www.w3.org/2000/svg"
              width="24"
              height="24"
              viewBox="0 0 24 24"
            >
              <path
                fill="currentColor"
                d="M13.46 12L19 17.54V19h-1.46L12 13.46L6.46 19H5v-1.46L10.54 12L5 6.46V5h1.46L12 10.54L17.54 5H19v1.46z"
              />
            </svg>
          </button>
        </div>
      </div>
  </UApp>

    <UModal v-model:open="showCreateModal">
      <template #content>
        <div class="px-6 pt-6 pb-2 flex flex-col gap-1">
          <h2 class="text-lg font-semibold text-gray-900 dark:text-white">
            Create a New Workspace
          </h2>
          <p class="text-sm text-gray-500 dark:text-gray-400">
            Set up a new workspace to organize your projects and files. You can
            have multiple workspaces and switch between them easily.
          </p>
        </div>

        <form
          class="px-6 pb-6 mt-4 flex flex-col gap-4"
          @submit.prevent="handleSubmit"
        >
          <div class="grid grid-cols-2 gap-3">
            <AppInput
              v-model="form.name"
              label="Name"
              hint="required"
              type="text"
              name="workspace-name"
              placeholder="Almonds"
              :error="errors.name"
              :disabled="loading"
            />
            <AppInput
              v-model="form.description"
              label="Description"
              hint="required"
              type="text"
              name="workspace-description"
              placeholder="Organize files and tasks"
              :error="errors.description"
              :disabled="loading"
            />
          </div>

          <div class="flex justify-end gap-2 pt-1">
            <UButton
              color="neutral"
              variant="ghost"
              :disabled="loading"
              @click="showCreateModal = false"
            >
              Cancel
            </UButton>
            <UButton
              type="submit"
              color="primary"
              :loading="loading"
              :disabled="loading"
            >
              Save and continue
            </UButton>
          </div>
        </form>
      </template>
    </UModal>
  </Body>
</template>

<style>
.traffic-lights {
  display: flex;
  gap: 8px;
  padding: 10px;
}

.btn {
  width: 11px;
  height: 12px;
  border-radius: 50%;
  display: inline-block;
  cursor: pointer;
}

/* Colors */
.close {
  background: #ff5f57;
}

.minimize {
  background: #ffbd2e;
}

.maximize {
  background: #28c840;
}

/* Optional hover icons */
.traffic-lights:hover .btn::after {
  content: "";
  display: block;
  width: 100%;
  height: 100%;
  background-size: 8px;
  background-repeat: no-repeat;
  background-position: center;
}

.close:hover::after {
  content: "✕";
  font-size: 8px;
  color: #4d0000;
  text-align: center;
}

.minimize:hover::after {
  content: "–";
  font-size: 10px;
  color: #664d00;
  text-align: center;
}

.maximize:hover::after {
  content: "+";
  font-size: 10px;
  color: #003300;
  text-align: center;
}
</style>
