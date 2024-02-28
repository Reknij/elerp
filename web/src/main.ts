import { createApp } from "vue";
import "./style.css";
import App from "./App.vue";
import router from "./router";
import { createPinia } from "pinia";
import { i18n } from "./i18n";

const app = createApp(App);
const pinia = createPinia();

app.use(router);
app.use(pinia);
app.use(i18n);

const meta = document.createElement("meta");
meta.name = "naive-ui-style";
document.head.appendChild(meta);
app.mount("#app");
