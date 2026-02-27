import { createApp } from 'vue';
import App from './App.vue';
import ElementPlus from 'element-plus';
import 'element-plus/dist/index.css';

document.addEventListener('DOMContentLoaded', () => {
  createApp(App).use(ElementPlus).mount('#app');
});