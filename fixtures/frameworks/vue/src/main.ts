import { createApp, defineComponent } from "vue";

const App = defineComponent({
  name: "App",
  template: "<h1>Hello Vue</h1>",
});

createApp(App).mount("#app");
