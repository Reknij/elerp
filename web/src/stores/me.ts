import { defineStore } from "pinia";
import {
  AuthenticatedUser,
  GetTokenQuery,
  UserConfigure,
} from "../api/user_system/models";
import { ref, computed } from "vue";
import {
  get_me,
  get_user_configure,
  get_user_token,
  remove_user_token,
  update_user_configure,
} from "../api/user_system";
import Cookies from "js-cookie";
import { web, erp } from "../api";
import { notication_subscribe } from "../api/ws";
import { WebSocketFlag, WebSocketFlagJson } from "../api/ws/models";
import { error_to_string } from "../AppError";
import { i18n } from "../i18n";

const AUTH_NAME = "authorization";
export const useMySelfUser = defineStore("myself", () => {
  const authenticated = ref<AuthenticatedUser>();
  const readyAccess = ref(false);
  const config = ref<UserConfigure>();
  const ws = ref<WebSocket>();
  const subscribeCallbacks = new Set<(flag: WebSocketFlagJson) => void>();
  let reconnectHandler: number | null = null;

  async function set_token(query: GetTokenQuery) {
    authenticated.value = await get_user_token(query);
    Cookies.set(AUTH_NAME, authenticated.value.token, {
      expires: 30,
    });
    web.defaults.headers.common["X-authorization"] =
      erp.defaults.headers.common["X-authorization"] =
        authenticated.value.token;
    await reconnectWebSocket();
    if (authenticated.value != undefined) {
      config.value = await get_user_configure(authenticated.value.user.id);
      invokeConfig(config.value);
    }
  }

  async function reconnectWebSocket() {
    readyAccess.value = false;
    const callback = (flag: WebSocketFlagJson) => {
      if (flag.isFlag(WebSocketFlag.ReadyAccess)) {
        readyAccess.value = true;
      }
    };
    subscribe(callback);
    if (authenticated.value) {
      if (reconnectHandler) {
        clearTimeout(reconnectHandler);
        reconnectHandler = null;
      }
      ws.value = notication_subscribe(
        authenticated.value.token,
        subscribeCallbacks
      );

      ws.value.onclose = () => {
        ws.value = undefined;
        reconnectHandler = setTimeout(() => {
          console.log("(6s) trying reconnect to server socket...");
          reconnectWebSocket();
        }, 6000);
      };
    }
  }

  function subscribe(callback: (flag: WebSocketFlagJson) => void) {
    unsubscribe(callback);
    subscribeCallbacks.add(callback);
  }

  function unsubscribe(callback: (flag: WebSocketFlagJson) => void) {
    subscribeCallbacks.delete(callback);
  }

  function clearAuthorization() {
    if (authenticated.value?.user.id) {
      remove_user_token(authenticated.value.user.id);
    }
    Cookies.remove(AUTH_NAME);
    config.value = authenticated.value = undefined;
    ws.value?.close(1000);
    ws.value = undefined;
    readyAccess.value = false;
    web.defaults.headers.common["X-authorization"] =
      erp.defaults.headers.common["X-authorization"] = undefined;
  }

  async function refresh() {
    config.value = authenticated.value = undefined;
    const auth = Cookies.get(AUTH_NAME);
    if (auth) {
      try {
        web.defaults.headers.common["X-authorization"] =
          erp.defaults.headers.common["X-authorization"] = auth;
        authenticated.value = await get_me();
        reconnectWebSocket();
        if (authenticated.value != undefined) {
          config.value = await get_user_configure(authenticated.value.user.id);
          invokeConfig(config.value);
        }
      } catch (error) {
        clearAuthorization();
        console.log(`failed to authorize: ${error_to_string(error)}`);
      }
    }
  }

  async function changeConfig(config: UserConfigure) {
    if (authenticated.value != undefined) {
      await update_user_configure(authenticated.value.user.id, config);
    }
    await invokeConfig(config);
  }
  async function invokeConfig(config: UserConfigure) {
    if (i18n.global.locale.value !== config.language) {
      if (!i18n.global.availableLocales.includes(config.language as any)) {
        let lang = {};
        if (config.language === "cn") {
          lang = (await import("../i18n/lang/cn")).default;
        } else if (config.language === "malay") {
          lang = (await import("../i18n/lang/malay")).default;
        }
        i18n.global.setLocaleMessage(config.language, lang as any);
      }

      i18n.global.locale.value = config.language as any;
    }
  }
  const logined = computed(() => authenticated.value != undefined);

  return {
    authenticated,
    logined,
    readyAccess,
    config,
    changeConfig,
    invokeConfig,
    refresh,
    clearAuthorization,
    set_token,
    subscribe,
    unsubscribe,
  };
});
