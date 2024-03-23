import React from 'react'
import ReactDOM from 'react-dom/client'
import App from './components/App.tsx'
import './index.css'
import init, { greet } from 'rs-pixel-maps';

init().then(() => {
  console.log('init wasm-pack');
  greet('from vite!');
});

ReactDOM.createRoot(document.getElementById('root')!).render(
  <React.StrictMode>
    <App />
  </React.StrictMode>,
)
