<script setup lang="ts">
import { NSpace, NInput, NImage, NButton, useMessage } from "naive-ui";
import { ref } from "vue";
import { useRouter } from "vue-router";
import { GetTokenQuery } from "../api/user_system/models";
import { useMySelfUser } from "../stores/me";
import {
  CustomErrorType,
  error_parse,
  error_to_string,
  fmt_err,
} from "../AppError";
import { useI18n } from "vue-i18n";

const { t } = useI18n();
const query = ref<GetTokenQuery>({
  username: "",
  password: "",
});
const router = useRouter();
const message = useMessage();
const myself = useMySelfUser();
if (myself.logined) {
  router.replace("/");
}
async function login() {
  if (query.value.username.length < 5 || query.value.password.length < 8) {
    message.warning(t("message.loginIllegalWarning"));
    return;
  }
  try {
    await myself.set_token(query.value);
    await router.replace("/");
    message.success(t("message.loginSuccess", { obj: query.value.username }));
  } catch (error) {
    const err = error_parse(error);
    if (err?.error_type === CustomErrorType.SameObject) {
      message.warning("You have login already! Please logout current account to continue login to other account.");
      await router.replace("/");
    } else {
      const msg = fmt_err(error, {
        obj: query.value.username,
      });
      message.error(msg ?? error_to_string(error));
    }
  }
}
</script>

<template>
  <NSpace class="h-screen" vertical justify="center" align="center">
    <n-image width="64" src="/elerp_logo.svg" />
    <h1 class="text-4xl m-3">{{ t("message.pleaseLogin") }}</h1>
    <NInput
      :placeholder="t('common.username')"
      v-model:value="query.username"
      @keyup="
        async (e) =>
          e.key.toLocaleLowerCase() == 'enter' ? await login() : undefined
      "
    ></NInput>
    <NInput
      :placeholder="t('common.password')"
      v-model:value="query.password"
      @keyup="
        async (e) =>
          e.key.toLocaleLowerCase() == 'enter' ? await login() : undefined
      "
    ></NInput>
    <NButton @click="login">{{ t("action.login") }}</NButton>
  </NSpace>
</template>

<style scoped></style>
