import { web } from "..";
import { WebSocketFlagJson } from "./models";

export function notication_subscribe(
  auth: string,
  subscribeCallbacks: Set<(flag: WebSocketFlagJson) => void>
): WebSocket {
  const protocols = ["elerp-ws", auth];
  const host = web.defaults.baseURL?.startsWith("http")
    ? new URL(web.defaults.baseURL).host
    : window.location.host;
  const protocol = location.protocol.startsWith("https") ? "wss" : "ws";
  var ws = new WebSocket(`${protocol}://${host}/socket`, protocols);
  ws.addEventListener("open", () => {
    console.log("Server socket connected.");
  });
  ws.addEventListener("error", (e) => {
    console.log(`Server socket occurs error:`);
    console.error(e);
  });

  ws.addEventListener("message", (e) => {
    const data = JSON.parse(e.data);
    const f = new WebSocketFlagJson(data);
    const callbacks = Array.from(subscribeCallbacks.values());
    for (let i = 0; i < callbacks.length; i++) {
      const callback = callbacks[i];
      callback(f);
    }
  });

  return ws;
}
