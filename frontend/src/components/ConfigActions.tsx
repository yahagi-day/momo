import { Component, createSignal } from 'solid-js';
import { saveConfig, loadConfig } from '../api/client';

interface Props {
  onConfigLoaded: () => void;
}

const ConfigActions: Component<Props> = (props) => {
  const [error, setError] = createSignal('');
  const [message, setMessage] = createSignal('');

  const handleSave = async () => {
    try {
      setError('');
      setMessage('');
      await saveConfig();
      setMessage('Saved');
      setTimeout(() => setMessage(''), 2000);
    } catch (e) {
      setError(e instanceof Error ? e.message : 'Failed to save');
    }
  };

  const handleLoad = async () => {
    const path = prompt('Config file path:', 'config.json');
    if (!path) return;
    try {
      setError('');
      setMessage('');
      await loadConfig(path);
      setMessage('Loaded');
      props.onConfigLoaded();
      setTimeout(() => setMessage(''), 2000);
    } catch (e) {
      setError(e instanceof Error ? e.message : 'Failed to load');
    }
  };

  return (
    <div class="config-actions">
      <button class="btn-action" onClick={handleSave}>Save Config</button>
      <button class="btn-action" onClick={handleLoad}>Load Config</button>
      {message() && <span style={{ color: '#6c6', "font-size": "0.85rem" }}>{message()}</span>}
      {error() && <span class="error-msg">{error()}</span>}
    </div>
  );
};

export default ConfigActions;
