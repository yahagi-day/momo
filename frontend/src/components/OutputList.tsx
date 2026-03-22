import { Component, Index, Show, For, createSignal } from 'solid-js';
import type { OutputConfig, CropRegion, DeviceInfo } from '../api/types';
import { OUTPUT_COLORS } from '../utils/coordinates';
import { getDevices } from '../api/client';
import OutputCard from './OutputCard';

const DISPLAY_MODES = [
  'Hd720p50', 'Hd720p5994', 'Hd720p60',
  'Hd1080i50', 'Hd1080i5994',
  'Hd1080p24', 'Hd1080p25', 'Hd1080p2997', 'Hd1080p30',
  'Hd1080p50', 'Hd1080p5994', 'Hd1080p60',
  'Uhd2160p24', 'Uhd2160p25', 'Uhd2160p2997', 'Uhd2160p30',
  'Uhd2160p50', 'Uhd2160p5994', 'Uhd2160p60',
];

const PIXEL_FORMATS = ['Uyvy', 'Bgra', 'V210'];

interface Props {
  outputs: OutputConfig[];
  onUpdated: () => Promise<void> | void;
  selectedOutputId: string | null;
  onSelectOutput: (id: string | null) => void;
  onCropChange: (id: string, crop: CropRegion) => void;
  pipelineRunning: boolean;
  onAddOutput: (output: OutputConfig) => Promise<void>;
  onRemoveOutput: (id: string) => Promise<void>;
  getWebRTCStream?: (streamId: string) => MediaStream | null;
}

const OutputList: Component<Props> = (props) => {
  const [adding, setAdding] = createSignal(false);
  const [devices, setDevices] = createSignal<DeviceInfo[]>([]);
  const [devIndex, setDevIndex] = createSignal(0);
  const [mode, setMode] = createSignal('Hd1080p5994');
  const [format, setFormat] = createSignal('Uyvy');
  const [name, setName] = createSignal('');
  const [error, setError] = createSignal('');

  const outputDevices = () => devices().filter(d => d.device_type === 'DeckLink' && d.has_output);

  const fetchDevices = async () => {
    try {
      const result = await getDevices();
      setDevices(result);
    } catch {
      setDevices([]);
    }
  };

  const openForm = () => {
    setAdding(true);
    setName(`output-${Date.now()}`);
    setError('');
    fetchDevices();
  };

  const handleAdd = async () => {
    const id = `out-${Date.now()}`;
    const output: OutputConfig = {
      id,
      name: name() || id,
      device_index: devIndex(),
      display_mode: mode(),
      pixel_format: format(),
      transform: { flip: { horizontal: false, vertical: false } },
      enabled: true,
    };
    try {
      setError('');
      await props.onAddOutput(output);
      setAdding(false);
    } catch (e) {
      setError(e instanceof Error ? e.message : 'Failed to add output');
    }
  };

  return (
    <div class="panel joycon-right">
      <h2>Outputs</h2>
      <div class="output-list">
        <Index each={props.outputs}>
          {(output, index) => (
            <OutputCard
              output={output()}
              color={OUTPUT_COLORS[index % OUTPUT_COLORS.length]}
              selected={props.selectedOutputId === output().id}
              onUpdated={props.onUpdated}
              onSelectOutput={props.onSelectOutput}
              onCropChange={props.onCropChange}
              pipelineRunning={props.pipelineRunning}
              onRemove={(id) => props.onRemoveOutput(id)}
              webrtcStream={props.getWebRTCStream?.(output().id) ?? null}
            />
          )}
        </Index>
      </div>

      <Show when={!props.pipelineRunning}>
        <Show when={!adding()} fallback={
          <div class="input-form" style={{ "margin-top": "8px" }}>
            <div class="input-form-row">
              <label>Device</label>
              <div style="display: flex; gap: 4px; flex: 1">
                <select
                  value={devIndex()}
                  onChange={(e) => setDevIndex(parseInt(e.target.value))}
                  style="flex: 1"
                >
                  <Show when={outputDevices().length === 0}>
                    <option value={0}>No devices (mock)</option>
                  </Show>
                  <For each={outputDevices()}>
                    {(d) => <option value={d.index}>{d.name} ({d.model_name})</option>}
                  </For>
                </select>
                <button class="btn-action" onClick={fetchDevices} title="Refresh devices" style="padding: 0 6px; min-width: auto">↻</button>
              </div>
            </div>
            <div class="input-form-row">
              <label>Mode</label>
              <select value={mode()} onChange={(e) => setMode(e.target.value)}>
                {DISPLAY_MODES.map((m) => <option value={m}>{m}</option>)}
              </select>
            </div>
            <div class="input-form-row">
              <label>Format</label>
              <select value={format()} onChange={(e) => setFormat(e.target.value)}>
                {PIXEL_FORMATS.map((f) => <option value={f}>{f}</option>)}
              </select>
            </div>
            <div class="input-form-row">
              <label>Name</label>
              <input type="text" value={name()} onInput={(e) => setName(e.target.value)} />
            </div>
            <div class="input-form-buttons">
              <button class="btn-action accent-blue" onClick={handleAdd}>Add</button>
              <button class="btn-action" onClick={() => { setAdding(false); setError(''); }}>Cancel</button>
            </div>
            {error() && <div class="error-msg">{error()}</div>}
          </div>
        }>
          <button class="btn-action accent-blue" style={{ "margin-top": "8px", width: "100%" }} onClick={openForm}>
            + Add Output
          </button>
        </Show>
      </Show>
    </div>
  );
};

export default OutputList;
