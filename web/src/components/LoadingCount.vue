<script setup lang="ts">
import { NSpace } from 'naive-ui';
import { ref } from 'vue';
import { useI18n } from 'vue-i18n';

const seconds = ref(0);
const {t} = useI18n();

function timer() {
    seconds.value += 1;
}
setInterval(() => {
    timer()
}, 1000);
</script>

<template>
    <NSpace align="center" justify="center" class="p-6">
        <h1 class="text-6xl">{{ t("common.loading") }} {{ seconds > 0? `(${seconds}s)`: '' }}</h1>
    </NSpace>
</template>