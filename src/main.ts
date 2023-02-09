import { createApp } from 'vue'
import App from './App.vue'
import { createRouter, createWebHistory } from 'vue-router'

// Styles
import 'bootstrap/scss/bootstrap.scss'
import '@/style/styles.scss'

const router = createRouter({
  history: createWebHistory(import.meta.env.BASE_URL),
  routes: [
    {
      path: '/',
      name: 'home',
      component: () => import('./components/pages/CanvasTest.vue')
    }
  ]
})

createApp(App)
  .use(router)
  .mount('#app')
