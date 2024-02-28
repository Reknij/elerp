<script setup lang="ts">
import { NButton, NModal, NCard, useMessage } from "naive-ui";
import { ref } from "vue";
import { useI18n } from "vue-i18n";
import CloseButton from "../../CloseButton.vue";

defineProps<{
  title: string;
  show: boolean;
}>();
defineEmits<{
  (e: "update:show", v: boolean): void;
}>();

const { t } = useI18n();
const message = useMessage();
const child = ref<HTMLElement>();

const copyText = () => {
  if (!child.value) {
    console.log("Result's child is null.");
    return;
  }
  navigator.clipboard.writeText(child.value.innerText);
  message.success(t("message.copySuccess"));
};
</script>

<template>
  <n-modal :show="show">
    <n-card
      style="width: 600px"
      :title="title"
      :bordered="false"
      size="huge"
      role="dialog"
      aria-modal="true"
    >
      <template #header-extra>
        <CloseButton @click="$emit('update:show', false)" />
      </template>
      <div ref="child">
        <slot></slot>
      </div>
      <br />
      <NButton @click="copyText">{{ t("common.copy") }}</NButton>
    </n-card>
  </n-modal>
</template>

<style scoped></style>
