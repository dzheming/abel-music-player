import { createApp } from 'vue'
import { createPinia } from 'pinia'
import router from './router'

import App from './App.vue'
import './assets/styles/global.css'
import { useLibraryStore } from './stores/library'
import { usePlaylistStore } from './stores/playlist'

document.addEventListener('contextmenu', (e) => {
    const target = e.target as HTMLElement
    if (target.tagName === 'INPUT' || target.tagName === 'TEXTAREA') return
    e.preventDefault()
})

const app = createApp(App)
const pinia = createPinia()
app.use(pinia)
app.use(router)

app.config.errorHandler = (err, _instance, info) => {
    console.error('[App Error]', info, err)
}

app.mount('#app')

useLibraryStore().initLibrary()
usePlaylistStore().init()