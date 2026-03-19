import { Component, Show, For, createSignal, createEffect } from 'solid-js';
import type { InputSource, PipelineState, Config, OutputConfig, CropRegion, DeviceInfo } from '../api/types';
import { putConfig, getDevices } from '../api/client';

interface Props {
  input: InputSource | null;
  config: Config | null;
  pipelineState: PipelineState;
  onUpdated: () => void;
  outputs: OutputConfig[];
  selectedOutputId: string | null;
  onSelectOutput: (id: string | null) => void;
  onCropChange: (id: string, crop: CropRegion) => void;
}

const InputPanel: Component<Props> = (props) => {
  const [inputType, setInputType] = createSignal<'Mock' | 'DeckLink' | 'Uvc'>('Mock');
  const [mockWidth, setMockWidth] = createSignal(1920);
  const [mockHeight, setMockHeight] = createSignal(1080);
  const [mockFps, setMockFps] = createSignal(30);
  const [dlIndex, setDlIndex] = createSignal(0);
  const [dlMode, setDlMode] = createSignal('Hd1080p5994');
  const [dlFormat, setDlFormat] = createSignal('Uyvy');
  const [uvcPath, setUvcPath] = createSignal('/dev/video0');
  const [error, setError] = createSignal('');
  const [editing, setEditing] = createSignal(false);
  const [devices, setDevices] = createSignal<DeviceInfo[]>([]);

  createEffect(() => {
    const src = props.input;
    if (!src) return;
    setInputType(src.type);
    switch (src.type) {
      case 'Mock':
        setMockWidth(src.width);
        setMockHeight(src.height);
        setMockFps(src.fps);
        break;
      case 'DeckLink':
        setDlIndex(src.device_index);
        setDlMode(src.display_mode);
        setDlFormat(src.pixel_format);
        break;
      case 'Uvc':
        setUvcPath(src.device_path);
        break;
    }
  });

  const fetchDevices = async () => {
    try {
      const result = await getDevices();
      setDevices(result);
    } catch {
      setDevices([]);
    }
  };

  createEffect(() => {
    if (editing()) {
      fetchDevices();
    }
  });

  const deckLinkInputDevices = () => devices().filter(d => d.device_type === 'DeckLink' && d.has_input);
  const uvcDevices = () => devices().filter(d => d.device_type === 'Uvc');

  const inputLabel = () => {
    const src = props.input;
    if (!src) return 'Not configured';
    switch (src.type) {
      case 'Mock': return `Mock ${src.width}x${src.height} @ ${src.fps}fps`;
      case 'DeckLink': return `DeckLink #${src.device_index}`;
      case 'Uvc': return `UVC ${src.device_path}`;
    }
  };

  const buildInputSource = (): InputSource => {
    switch (inputType()) {
      case 'Mock':
        return { type: 'Mock', width: mockWidth(), height: mockHeight(), fps: mockFps() };
      case 'DeckLink':
        return { type: 'DeckLink', device_index: dlIndex(), display_mode: dlMode(), pixel_format: dlFormat() };
      case 'Uvc':
        return { type: 'Uvc', device_path: uvcPath() };
    }
  };

  const handleApply = async () => {
    const cfg = props.config;
    if (!cfg) return;
    try {
      setError('');
      const newConfig: Config = { ...cfg, input: buildInputSource() };
      await putConfig(newConfig);
      setEditing(false);
      props.onUpdated();
    } catch (e) {
      setError(e instanceof Error ? e.message : 'Failed to apply');
    }
  };

  const isRunning = () => props.pipelineState === 'Running';

  const DISPLAY_MODES = [
    'Hd720p50', 'Hd720p5994', 'Hd720p60',
    'Hd1080i50', 'Hd1080i5994',
    'Hd1080p24', 'Hd1080p25', 'Hd1080p2997', 'Hd1080p30',
    'Hd1080p50', 'Hd1080p5994', 'Hd1080p60',
    'Uhd2160p24', 'Uhd2160p25', 'Uhd2160p2997', 'Uhd2160p30',
    'Uhd2160p50', 'Uhd2160p5994', 'Uhd2160p60',
  ];

  return (
    <div class="panel joycon-left">
      <h2>Input</h2>

      <Show when={!editing()}>
        <div class="input-info">{inputLabel()}</div>
        <Show when={!isRunning() && props.config}>
          <button class="btn-action accent-red" onClick={() => setEditing(true)}>
            Change Input
          </button>
        </Show>
      </Show>

      <Show when={editing()}>
        <div class="input-form">
          <div class="input-form-row">
            <label>Type</label>
            <select
              value={inputType()}
              onChange={(e) => setInputType(e.target.value as 'Mock' | 'DeckLink' | 'Uvc')}
            >
              <option value="Mock">Mock</option>
              <option value="DeckLink">DeckLink</option>
              <option value="Uvc">UVC</option>
            </select>
          </div>

          <Show when={inputType() === 'Mock'}>
            <div class="input-form-row">
              <label>Width</label>
              <input type="number" value={mockWidth()} onInput={(e) => setMockWidth(parseInt(e.target.value) || 0)} />
            </div>
            <div class="input-form-row">
              <label>Height</label>
              <input type="number" value={mockHeight()} onInput={(e) => setMockHeight(parseInt(e.target.value) || 0)} />
            </div>
            <div class="input-form-row">
              <label>FPS</label>
              <input type="number" value={mockFps()} onInput={(e) => setMockFps(parseInt(e.target.value) || 0)} />
            </div>
          </Show>

          <Show when={inputType() === 'DeckLink'}>
            <div class="input-form-row">
              <label>Device</label>
              <div style="display: flex; gap: 4px; flex: 1">
                <select
                  value={dlIndex()}
                  onChange={(e) => setDlIndex(parseInt(e.target.value))}
                  style="flex: 1"
                  disabled={deckLinkInputDevices().length === 0}
                >
                  <Show when={deckLinkInputDevices().length === 0}>
                    <option>No devices found</option>
                  </Show>
                  <For each={deckLinkInputDevices()}>
                    {(d) => <option value={d.index}>{d.name} ({d.model_name})</option>}
                  </For>
                </select>
                <button class="btn-action" onClick={fetchDevices} title="Refresh devices" style="padding: 0 6px; min-width: auto">↻</button>
              </div>
            </div>
            <div class="input-form-row">
              <label>Mode</label>
              <select value={dlMode()} onChange={(e) => setDlMode(e.target.value)}>
                {DISPLAY_MODES.map((m) => <option value={m}>{m}</option>)}
              </select>
            </div>
            <div class="input-form-row">
              <label>Format</label>
              <select value={dlFormat()} onChange={(e) => setDlFormat(e.target.value)}>
                <option value="Uyvy">UYVY</option>
                <option value="Bgra">BGRA</option>
                <option value="V210">V210</option>
              </select>
            </div>
          </Show>

          <Show when={inputType() === 'Uvc'}>
            <div class="input-form-row">
              <label>Device</label>
              <div style="display: flex; gap: 4px; flex: 1">
                <select
                  value={uvcPath()}
                  onChange={(e) => setUvcPath(e.target.value)}
                  style="flex: 1"
                  disabled={uvcDevices().length === 0}
                >
                  <Show when={uvcDevices().length === 0}>
                    <option>No devices found</option>
                  </Show>
                  <For each={uvcDevices()}>
                    {(d) => <option value={d.index.toString()}>{d.name}</option>}
                  </For>
                </select>
                <button class="btn-action" onClick={fetchDevices} title="Refresh devices" style="padding: 0 6px; min-width: auto">↻</button>
              </div>
            </div>
          </Show>

          <div class="input-form-buttons">
            <button
              class="btn-action accent-red"
              onClick={handleApply}
              disabled={
                (inputType() === 'DeckLink' && deckLinkInputDevices().length === 0) ||
                (inputType() === 'Uvc' && uvcDevices().length === 0)
              }
            >Apply</button>
            <button class="btn-action" onClick={() => { setEditing(false); setError(''); }}>Cancel</button>
          </div>
          {error() && <div class="error-msg">{error()}</div>}
        </div>
      </Show>
    </div>
  );
};

export default InputPanel;
