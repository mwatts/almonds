<script setup lang="ts">
definePageMeta({ layout: "auth" });

const route = useRoute();
const authApi = useAuthApi();
const authStore = useAuthStore();
const { notify } = useAppNotification();

const flow = computed(() =>
  route.query.flow === "reset" ? "reset" : "verify",
);

const otp = ref("");
const errors = reactive({ otp: "" });
const loading = ref(false);
const submitError = ref("");

const title = computed(() =>
  flow.value === "reset" ? "Verify reset code" : "Confirm your email",
);

const description = computed(() =>
  flow.value === "reset"
    ? "Enter the 6-digit code we sent to your email to reset your password."
    : "Enter the 6-digit verification code we sent to your email to activate your account.",
);

function validate(): boolean {
  errors.otp = /^\d{6}$/.test(otp.value.trim()) ? "" : "";
  return !errors.otp;
}

async function handleSubmit() {
  if (!validate()) return;
  if (!authStore.hasPendingToken) {
    submitError.value =
      "Your verification session has expired. Please start over.";
    return;
  }

  loading.value = true;
  submitError.value = "";
  try {
    const response =
      flow.value === "reset"
        ? await authApi.verifyResetOtp(
            { otp: otp.value.trim() },
            authStore.pendingToken,
          )
        : await authApi.verifyAccount(
            { otp: otp.value.trim() },
            authStore.pendingToken,
          );

    if (flow.value === "reset") {
      authStore.setPendingToken(response.token);
      notify({ message: "Code verified", type: "success" });
      await navigateTo("/auth/reset-password?step=set");
    } else {
      authStore.clearPendingToken();
      authStore.setSession(response.token, "", 0);
      notify({ message: "Account verified successfully", type: "success" });
      await navigateTo("/");
    }
  } catch (error) {
    submitError.value = (error as Error).message;
  } finally {
    loading.value = false;
  }
}
</script>

<template>
  <div class="flex flex-col gap-5">
    <div class="flex flex-col gap-1">
      <h2 class="text-lg font-semibold text-gray-900 dark:text-white">
        {{ title }}
      </h2>
      <p class="text-sm text-gray-500 dark:text-gray-400">{{ description }}</p>
    </div>

    <form class="flex flex-col gap-4" @submit.prevent="handleSubmit">
      <AppNumberInput
        v-model="otp"
        label="Verification code"
        name="otp"
        placeholder="••••••"
        :disabled="loading"
        inputmode="numeric"
        autocomplete="one-time-code"
        :maxlength="6"
      />
      <p v-if="errors.otp" class="text-xs text-red-500 -mt-3">
        {{ errors.otp }}
      </p>

      <p v-if="submitError" class="text-sm text-red-500">{{ submitError }}</p>

      <AppButton
        type="submit"
        color="primary"
        class="w-full py-3 bg-accent-500 hover:bg-accent-600 rounded-lg text-white font-medium disabled:opacity-50 text-center"
        :loading="loading"
        :disabled="loading"
      >
        Verify code
      </AppButton>
    </form>

    <p class="text-sm text-center text-gray-500 dark:text-gray-400">
      {{ flow === "reset" ? "Remembered your password?" : "Already verified?" }}
      <NuxtLink
        :to="flow === 'reset' ? '/auth/login' : '/auth/login'"
        class="text-accent-500 hover:text-accent-600 font-medium"
      >
        Sign in
      </NuxtLink>
    </p>
  </div>
</template>
