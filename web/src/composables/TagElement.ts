import { NTag } from "naive-ui";
import { h } from "vue";

export const getTagElement = (
  content: string,
  color: string | undefined = undefined,
  textColor: string | undefined = undefined,
) => {
  return h(
    NTag,
    {
      bordered: false,
      color: {
        color,
        textColor
      },
      style: {
        margin: '3px'
      }
    },
    () => content
  );
};