<script setup lang="ts">
import { NCheckbox } from "naive-ui";

defineProps<{
  valueSet: Set<string>;
  valueKey: string;
}>();
defineEmits<{
  (e: "update:valueSet", v: Set<string>): void;
}>();
</script>

<template>
  <NCheckbox
    :checked="!valueSet.has(valueKey)"
    @update-checked="
      (checked) => {
        if (checked) {
          valueSet.delete(valueKey);
          $emit('update:valueSet', valueSet);
        } else {
          $emit('update:valueSet', valueSet.add(valueKey));
        }
      }
    "
    ><slot></slot
  ></NCheckbox>
</template>
