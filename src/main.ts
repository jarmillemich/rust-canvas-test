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
    },
    {
      path: '/host',
      name: 'host',
      component: () => import('./components/pages/HostSession.vue')
    },
    {
      path: '/join',
      name: 'join',
      component: () => import('./components/pages/JoinSession.vue')
    }
  ]
})

createApp(App)
  .use(router)
  .mount('#app')
