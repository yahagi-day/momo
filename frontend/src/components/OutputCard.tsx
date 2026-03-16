import { Component, Show, createSignal } from 'solid-js';
import type { OutputConfig, CropRegion } from '../api/types';
import { updateOutput } from '../api/client';

interface Props {
  output: OutputConfig;
  color: string;
  selected: boolean;
  onUpdated: () => void;
  onSelectOutput: (id: string | null) => void;
  onCropChange: (id: string, crop: CropRegion) => void;
}

const OutputCard: Component<Props> = (props) => {
  const [flipH, setFlipH] = createSignal(props.output.transform.flip.horizontal);
  const [flipV, setFlipV] = createSignal(props.output.transform.flip.vertical);
  const initCrop = props.output.transform.crop;
  const [cropX, setCropX] = createSignal(initCrop?.x ?? 0);
  const [cropY, setCropY] = createSignal(initCrop?.y ?? 0);
  const [cropW, setCropW] = createSignal(initCrop?.width ?? 0);
  const [cropH, setCropH] = createSignal(initCrop?.height ?? 0);
  const [error, setError] = createSignal('');

  const hasCrop = () => props.output.transform.crop != null;

  const handleApply = async () => {
    try {
      setError('');
      const crop: CropRegion | null = hasCrop()
        ? { x: cropX(), y: cropY(), width: cropW(), height: cropH() }
        : null;
      await updateOutput(props.output.id, {
        crop,
        flip: { horizontal: flipH(), vertical: flipV() },
      });
      props.onUpdated();
    } catch (e) {
      setError(e instanceof Error ? e.message : 'Failed');
    }
  };

  const handleEditCrop = () => {
    if (!hasCrop()) {
      // Initialize crop to full frame — use a sensible default
      const defaultCrop: CropRegion = { x: 0, y: 0, width: 1920, height: 1080 };
      props.onCropChange(props.output.id, defaultCrop);
    }
    props.onSelectOutput(props.output.id);
  };

  const handleClearCrop = () => {
    setError('');
    updateOutput(props.output.id, {
      crop: null,
      flip: { horizontal: flipH(), vertical: flipV() },
    }).then(() => {
      props.onUpdated();
      props.onSelectOutput(null);
    }).catch((e) => {
      setError(e instanceof Error ? e.message : 'Failed');
    });
  };

  const handleCropFieldChange = (field: 'x' | 'y' | 'width' | 'height', value: number) => {
    // Align x and width to 2px boundary (UYVY constraint)
    if (field === 'x') setCropX(Math.round(value / 2) * 2);
    else if (field === 'y') setCropY(value);
    else if (field === 'width') setCropW(Math.round(value / 2) * 2);
    else if (field === 'height') setCropH(value);
  };

  return (
    <div
      class="output-card"
      style={{ "border-left": `3px solid ${props.color}` }}
    >
      <h3>{props.output.name} ({props.output.id})</h3>
      <div class="fields">
        <label>Mode:</label>
        <span>{props.output.display_mode}</span>
        <label>Format:</label>
        <span>{props.output.pixel_format}</span>
        <label>Device:</label>
        <span>#{props.output.device_index}</span>
      </div>

      <Show when={hasCrop()}>
        <div class="crop-fields">
          <label>Crop:</label>
          <div class="crop-inputs">
            <label>X</label>
            <input type="number" value={cropX()} step={2}
              onInput={(e) => handleCropFieldChange('x', parseInt(e.target.value) || 0)} />
            <label>Y</label>
            <input type="number" value={cropY()}
              onInput={(e) => handleCropFieldChange('y', parseInt(e.target.value) || 0)} />
            <label>W</label>
            <input type="number" value={cropW()} step={2}
              onInput={(e) => handleCropFieldChange('width', parseInt(e.target.value) || 0)} />
            <label>H</label>
            <input type="number" value={cropH()}
              onInput={(e) => handleCropFieldChange('height', parseInt(e.target.value) || 0)} />
          </div>
        </div>
      </Show>

      <div style={{ "margin-top": "8px" }}>
        <div class="checkbox-row">
          <input type="checkbox" checked={flipH()} onChange={(e) => setFlipH(e.target.checked)} />
          <label>Flip H</label>
          <input type="checkbox" checked={flipV()} onChange={(e) => setFlipV(e.target.checked)} />
          <label>Flip V</label>
        </div>
        <div class="output-card-actions">
          <button class="btn-action" onClick={handleApply}>Apply</button>
          <button class="btn-action" onClick={handleEditCrop}>Edit Crop</button>
          <Show when={hasCrop()}>
            <button class="btn-action" onClick={handleClearCrop}>Clear Crop</button>
          </Show>
        </div>
      </div>
      {error() && <div class="error-msg">{error()}</div>}
    </div>
  );
};

export default OutputCard;
