import { Component, createSignal, onMount, onCleanup } from 'solid-js';
import type { Config, PipelineState, CropRegion } from './api/types';
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
  const [selectedOutputId, setSelectedOutputId] = createSignal<string | null>(null);

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

  const handleCropChange = (id: string, crop: CropRegion) => {
    const cfg = config();
    if (!cfg) return;
    const newOutputs = cfg.outputs.map((o) =>
      o.id === id ? { ...o, transform: { ...o.transform, crop } } : o
    );
    setConfig({ ...cfg, outputs: newOutputs });
  };

  return (
    <div class="app">
      <StatusBar state={state()} fps={fps()} onStart={handleStart} onStop={handleStop} />
      {error() && <div class="error-msg" style={{ "margin-bottom": "12px" }}>{error()}</div>}
      <div class="main-content">
        <InputPanel
          input={config()?.input ?? null}
          config={config()}
          pipelineState={state()}
          onUpdated={fetchConfig}
          outputs={config()?.outputs ?? []}
          selectedOutputId={selectedOutputId()}
          onSelectOutput={setSelectedOutputId}
          onCropChange={handleCropChange}
        />
        <OutputList
          outputs={config()?.outputs ?? []}
          onUpdated={fetchConfig}
          selectedOutputId={selectedOutputId()}
          onSelectOutput={setSelectedOutputId}
          onCropChange={handleCropChange}
        />
      </div>
      <ConfigActions onConfigLoaded={fetchConfig} />
    </div>
  );
};

export default App;
