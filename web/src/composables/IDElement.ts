import { NTag } from "naive-ui";
import { h } from "vue";

export const getIDElement = (id: number) => {
  return h(
    NTag,
    {
      color: {
        color: "#f0f0f0",
        textColor: "#303030",
        borderColor: "gray",
      },
      style: {
        margin: '3px'
      }
    },
    () => `ID-${id}`
  );
};
