import { Component, createSignal, onMount, onCleanup, Show } from 'solid-js';
import type { Config, PipelineState, CropRegion, OutputConfig } from './api/types';
import { getStatus, getConfig, putConfig, startPipeline } from './api/client';
import { connectWebSocket } from './api/websocket';
import { PreviewStream } from './api/webrtc';
import StatusBar from './components/StatusBar';
import InputPanel from './components/InputPanel';
import OutputList from './components/OutputList';
import ConfigActions from './components/ConfigActions';
import FpsChart from './components/FpsChart';
import CropOverlay from './components/CropOverlay';
import { getInputResolution } from './utils/coordinates';

const App: Component = () => {
  const [state, setState] = createSignal<PipelineState>('Stopped');
  const [fps, setFps] = createSignal(0);
  const [config, setConfig] = createSignal<Config | null>(null);
  const [error, setError] = createSignal('');
  const [selectedOutputId, setSelectedOutputId] = createSignal<string | null>(null);
  const [webrtcStreams, setWebrtcStreams] = createSignal<Record<string, MediaStream>>({});

  let previewStream: PreviewStream | null = null;

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

  const setupWebRTC = () => {
    if (!PreviewStream.isSupported()) return;

    previewStream = new PreviewStream((streamId, stream) => {
      setWebrtcStreams((prev) => {
        const next = { ...prev };
        if (stream) {
          next[streamId] = stream;
        } else {
          delete next[streamId];
        }
        return next;
      });
    });
    previewStream.connect();
  };

  const subscribeWebRTCPreviews = () => {
    if (!previewStream) return;
    if (state() !== 'Running') return;
    const cfg = config();

    previewStream.subscribe('input');
    if (cfg) {
      for (const output of cfg.outputs) {
        previewStream.subscribe(output.id);
      }
    }
  };

  onMount(() => {
    fetchState();
    fetchConfig();
    setupWebRTC();
  });

  const ws = connectWebSocket((event) => {
    switch (event.type) {
      case 'StateChanged':
        if (event.state) {
          setState(event.state);
          if (event.state === 'Running') {
            subscribeWebRTCPreviews();
          }
        }
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

  onCleanup(() => {
    ws.cleanup();
    previewStream?.destroy();
  });

  const handleStart = async () => {
    try {
      setError('');
      await startPipeline();
    } catch (e) {
      setError(e instanceof Error ? e.message : 'Failed to start');
    }
  };

  const handleStop = () => {
    setError('');
    setFps(0);
    ws.send('stop');
  };

  const handleCropChange = (id: string, crop: CropRegion) => {
    const cfg = config();
    if (!cfg) return;
    const newOutputs = cfg.outputs.map((o) =>
      o.id === id ? { ...o, transform: { ...o.transform, crop } } : o
    );
    setConfig({ ...cfg, outputs: newOutputs });
  };

  const handleAddOutput = async (output: OutputConfig) => {
    const cfg = config();
    if (!cfg) return;
    const newConfig: Config = { ...cfg, outputs: [...cfg.outputs, output] };
    await putConfig(newConfig);
    await fetchConfig();
  };

  const handleRemoveOutput = async (id: string) => {
    const cfg = config();
    if (!cfg) return;
    const newConfig: Config = { ...cfg, outputs: cfg.outputs.filter((o) => o.id !== id) };
    await putConfig(newConfig);
    await fetchConfig();
  };

  const isRunning = () => state() === 'Running';

  const getWebRTCStream = (streamId: string): MediaStream | null => {
    return webrtcStreams()[streamId] ?? null;
  };

  return (
    <div class="app">
      <StatusBar state={state()} fps={fps()} onStart={handleStart} onStop={handleStop} />

      {error() && <div class="error-msg" style={{ padding: "0 8px" }}>{error()}</div>}

      <div class="main-content">
        {/* Left Joy-Con: Input Controls */}
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

        {/* Center: Main Screen */}
        <div class="panel screen-panel">
          <div class="screen-content">
            <span class="screen-label">Preview</span>
            <Show when={isRunning() && config()?.input} fallback={
              <span class="screen-placeholder">Standby</span>
            }>
              <CropOverlay
                src="/api/preview/input"
                webrtcStream={getWebRTCStream('input')}
                outputs={config()?.outputs ?? []}
                inputRes={getInputResolution(config()!.input)}
                selectedOutputId={selectedOutputId()}
                onSelectOutput={setSelectedOutputId}
                onCropChange={handleCropChange}
              />
            </Show>
          </div>
        </div>

        {/* Right Joy-Con: Outputs */}
        <OutputList
          outputs={config()?.outputs ?? []}
          onUpdated={fetchConfig}
          selectedOutputId={selectedOutputId()}
          onSelectOutput={setSelectedOutputId}
          onCropChange={handleCropChange}
          pipelineRunning={isRunning()}
          onAddOutput={handleAddOutput}
          onRemoveOutput={handleRemoveOutput}
          getWebRTCStream={getWebRTCStream}
        />
      </div>

      <div class="bottom-bar">
        <div class="waveform-footer">
          <FpsChart height={64} fps={fps()} />
        </div>
        <ConfigActions onConfigLoaded={fetchConfig} />
      </div>
    </div>
  );
};

export default App;
