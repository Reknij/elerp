import axios from "axios";
import qs from "qs";

export const web = axios.create({
  baseURL: "/api",
  paramsSerializer: {
    indexes: true,
    serialize: (params) => {
      const paramsKeys = Object.keys(params);
      const paramsArr = Object.values(params);
      for (let i = 0; i < paramsArr.length; i++) {
        const v = paramsArr[i];
        if (v instanceof Set) {
          const key = paramsKeys[i];
          params[key] = Array.from(v);
        }
      }
      return qs.stringify(params, { arrayFormat: "indices", encode: false });
    },
  },
});

export const erp = axios.create({
  baseURL: "/api/erp",
  paramsSerializer: {
    indexes: true,
    serialize: (params) => {
      const paramsKeys = Object.keys(params);
      const paramsArr = Object.values(params);
      for (let i = 0; i < paramsArr.length; i++) {
        const v = paramsArr[i];
        if (v instanceof Set) {
          const key = paramsKeys[i];
          params[key] = Array.from(v);
        }
      }
      return qs.stringify(params, { arrayFormat: "indices", encode: false });
    },
  },
});
