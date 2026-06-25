// Svelte 5 entry point. Mounts the root App component.
import App from './App.svelte';
import { mount } from 'svelte';

const app = mount(App, { target: document.getElementById('app') });
export default app;
