import { mount } from 'svelte'
import './app.css'
import App from './App.svelte'

// Only the e2e build mode installs the mock; the condition is a build-time constant,
// so the production bundle drops the import.
if (import.meta.env.MODE === 'e2e') {
  const { installMockIpc } = await import('./lib/mock-ipc')
  installMockIpc()
}

const app = mount(App, {
  target: document.getElementById('app')!,
})

export default app
