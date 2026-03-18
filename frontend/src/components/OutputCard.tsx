import { Component, Show, createSignal } from 'solid-js';
import type { OutputConfig, CropRegion } from '../api/types';
import { updateOutput } from '../api/client';

interface Props {
  output: OutputConfig;
  color: string;
  selected: boolean;
  onUpdated: () => Promise<void> | void;
  onSelectOutput: (id: string | null) => void;
  onCropChange: (id: string, crop: CropRegion) => void;
  pipelineRunning: boolean;
  onRemove?: (id: string) => void;
}

const OutputCard: Component<Props> = (props) => {
  const [flipH, setFlipH] = createSignal(props.output.transform.flip.horizontal);
  const [flipV, setFlipV] = createSignal(props.output.transform.flip.vertical);
  const [error, setError] = createSignal('');
  const [editing, setEditing] = createSignal(false);

  const crop = () => props.output.transform.crop;
  const hasCrop = () => crop() != null;

  const handleApply = async () => {
    try {
      setError('');
      await updateOutput(props.output.id, {
        crop: crop(),
        flip: { horizontal: flipH(), vertical: flipV() },
      });
      setEditing(false);
      props.onSelectOutput(null);
      await props.onUpdated();
    } catch (e) {
      setError(e instanceof Error ? e.message : 'Failed');
    }
  };

  const handleEditCrop = () => {
    if (!hasCrop()) {
      const defaultCrop: CropRegion = { x: 0, y: 0, width: 1920, height: 1080 };
      props.onCropChange(props.output.id, defaultCrop);
    }
    setEditing(true);
    props.onSelectOutput(props.output.id);
  };

  const handleClearCrop = () => {
    setError('');
    updateOutput(props.output.id, {
      crop: null,
      flip: { horizontal: flipH(), vertical: flipV() },
    }).then(async () => {
      setEditing(false);
      props.onSelectOutput(null);
      await props.onUpdated();
    }).catch((e) => {
      setError(e instanceof Error ? e.message : 'Failed');
    });
  };

  const handleCropFieldChange = (field: 'x' | 'y' | 'width' | 'height', value: number) => {
    const c = crop();
    if (!c) return;
    const updated = { ...c };
    if (field === 'x') updated.x = Math.round(value / 2) * 2;
    else if (field === 'y') updated.y = value;
    else if (field === 'width') updated.width = Math.round(value / 2) * 2;
    else if (field === 'height') updated.height = value;
    props.onCropChange(props.output.id, updated);
  };

  return (
    <div
      class={`output-card${props.selected ? ' selected' : ''}`}
      style={{ "--card-color": props.color } as any}
    >
      <div style={{
        position: "absolute", top: 0, left: 0,
        width: "4px", height: "100%",
        background: props.color, "border-radius": "2px"
      }} />
      <div style={{ display: "flex", "align-items": "center", "justify-content": "space-between" }}>
        <h3 style={{ margin: 0 }}>{props.output.name} <span style={{ color: "var(--text-muted)", "font-weight": "400" }}>({props.output.id})</span></h3>
        <Show when={!props.pipelineRunning && props.onRemove}>
          <button
            class="btn-action accent-red"
            style={{ padding: "2px 8px", "font-size": "0.75rem", "min-width": "auto" }}
            onClick={() => props.onRemove?.(props.output.id)}
          >Delete</button>
        </Show>
      </div>
      <div class="fields">
        <label>Mode</label>
        <span>{props.output.display_mode}</span>
        <label>Fmt</label>
        <span>{props.output.pixel_format}</span>
        <label>Dev</label>
        <span>#{props.output.device_index}</span>
      </div>

      <Show when={props.pipelineRunning}>
        <div class="output-preview">
          <img src={`/api/preview/output/${props.output.id}`} alt={`${props.output.name} preview`} />
        </div>
      </Show>

      <Show when={hasCrop()}>
        <div class="crop-fields">
          <label>Crop</label>
          <div class="crop-inputs">
            <label>X</label>
            <input type="number" value={crop()!.x} step={2} disabled={!editing()}
              onInput={(e) => handleCropFieldChange('x', parseInt(e.target.value) || 0)} />
            <label>Y</label>
            <input type="number" value={crop()!.y} disabled={!editing()}
              onInput={(e) => handleCropFieldChange('y', parseInt(e.target.value) || 0)} />
            <label>W</label>
            <input type="number" value={crop()!.width} step={2} disabled={!editing()}
              onInput={(e) => handleCropFieldChange('width', parseInt(e.target.value) || 0)} />
            <label>H</label>
            <input type="number" value={crop()!.height} disabled={!editing()}
              onInput={(e) => handleCropFieldChange('height', parseInt(e.target.value) || 0)} />
          </div>
        </div>
      </Show>

      <div style={{ "margin-top": "8px" }}>
        <div class="checkbox-row">
          <input type="checkbox" checked={flipH()} disabled={!editing()} onChange={(e) => setFlipH(e.target.checked)} />
          <label>Flip H</label>
          <input type="checkbox" checked={flipV()} disabled={!editing()} onChange={(e) => setFlipV(e.target.checked)} />
          <label>Flip V</label>
        </div>
        <div class="output-card-actions">
          <Show when={editing()} fallback={
            <button class="btn-action accent-blue" onClick={handleEditCrop}>Edit Crop</button>
          }>
            <button class="btn-action accent-blue" onClick={handleApply}>Apply</button>
            <button class="btn-action" onClick={() => { setEditing(false); props.onSelectOutput(null); }}>Cancel</button>
            <Show when={hasCrop()}>
              <button class="btn-action accent-red" onClick={handleClearCrop}>Clear</button>
            </Show>
          </Show>
        </div>
      </div>
      {error() && <div class="error-msg">{error()}</div>}
    </div>
  );
};

export default OutputCard;
