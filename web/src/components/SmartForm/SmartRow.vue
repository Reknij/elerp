<script setup lang="ts">
import { NSpace } from "naive-ui";
import { FormRowType } from "./interfaces";
import { ref } from "vue";
import { useCached } from "../../stores/cached";
import { VNode } from "vue";
import { getTagElement } from "../../composables/TagElement";
import { getGuestOrderStatusElement } from "../../composables/GuestOrderStatusElement";

const props = defineProps<{
  value?: any;
  row: FormRowType;
}>();

const cached = useCached();

const toRenderArr = ref<VNode[]>([]);
if (!props.value) {
  toRenderArr.value.push(getTagElement("Empty", undefined, "gray"));
} else {
  switch (props.row) {
    case FormRowType.SKU:
      const sku = await cached.getSKU(props.value);
      const category = await cached.getSKUCategory(sku.sku_category_id);
      toRenderArr.value.push(
        getTagElement(category.name, category.color, category.text_color),
        getTagElement(sku.name, sku.color, sku.text_color)
      );
      break;
    case FormRowType.GuestOrderStatus:
      toRenderArr.value.push(getGuestOrderStatusElement(props.value));
      break;
    default:
      break;
  }
}
</script>

<template>
  <NSpace align="center" v-bind="$attrs">
    <component v-for="item in toRenderArr" :is="item"></component>
  </NSpace>
</template>
