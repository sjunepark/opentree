import { mount } from 'svelte';
import './app.css';
import TestPage from './TestPage.svelte';

mount(TestPage, { target: document.getElementById('app')! });
