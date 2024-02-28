import { NTag } from "naive-ui";
import { h, ref } from "vue";
import { useCached } from "../stores/cached";

export const getUserElement = (
  id: number,
) => {
  const cached = useCached();
  const alias = ref('...');
  cached.getUser(id).then(v=> alias.value = v.alias).catch(() => alias.value = 'Unknown!');
  return h(
    NTag,
    () => alias.value
  );
};