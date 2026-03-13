import { Component, createSignal, onMount, onCleanup } from 'solid-js';
import type { Config, PipelineState } from './api/types';
import { getStatus, getConfig, startPipeline, stopPipeline } from './api/client';
import { connectWebSocket } from './api/websocket';
import StatusBar from './components/StatusBar';
import InputPanel from './components/InputPanel';
import OutputList from './components/OutputList';
import ConfigActions from './components/ConfigActions';

const App: Component = () => {
  const [state, setState] = createSignal<PipelineState>('Stopped');
  const [fps, setFps] = createSignal(0);
  const [config, setConfig] = createSignal<Config | null>(null);
  const [error, setError] = createSignal('');

  const fetchState = async () => {
    try {
      const status = await getStatus();
      setState(status.state);
    } catch {
      // ignore
    }
  };

  const fetchConfig = async () => {
    try {
      const cfg = await getConfig();
      setConfig(cfg);
    } catch {
      // ignore - no config set
    }
  };

  onMount(() => {
    fetchState();
    fetchConfig();
  });

  const cleanup = connectWebSocket((event) => {
    switch (event.type) {
      case 'StateChanged':
        if (event.state) setState(event.state);
        break;
      case 'FpsUpdate':
        if (event.fps !== undefined) setFps(event.fps);
        break;
      case 'ConfigChanged':
        fetchConfig();
        break;
      case 'Error':
        setError(event.message || 'Unknown error');
        break;
    }
  });

  onCleanup(cleanup);

  const handleStart = async () => {
    try {
      setError('');
      await startPipeline();
    } catch (e) {
      setError(e instanceof Error ? e.message : 'Failed to start');
    }
  };

  const handleStop = async () => {
    try {
      setError('');
      setFps(0);
      await stopPipeline();
    } catch (e) {
      setError(e instanceof Error ? e.message : 'Failed to stop');
    }
  };

  return (
    <div class="app">
      <StatusBar state={state()} fps={fps()} onStart={handleStart} onStop={handleStop} />
      {error() && <div class="error-msg" style={{ "margin-bottom": "12px" }}>{error()}</div>}
      <div class="main-content">
        <InputPanel input={config()?.input ?? null} pipelineState={state()} />
        <OutputList outputs={config()?.outputs ?? []} onUpdated={fetchConfig} />
      </div>
      <ConfigActions onConfigLoaded={fetchConfig} />
    </div>
  );
};

export default App;
