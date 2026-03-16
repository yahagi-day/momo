import { Component, Show, createSignal, createEffect } from 'solid-js';
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
  const [cropX, setCropX] = createSignal(0);
  const [cropY, setCropY] = createSignal(0);
  const [cropW, setCropW] = createSignal(0);
  const [cropH, setCropH] = createSignal(0);
  const [error, setError] = createSignal('');

  // Sync local state from props (including overlay drag changes)
  createEffect(() => {
    setFlipH(props.output.transform.flip.horizontal);
    setFlipV(props.output.transform.flip.vertical);
    const crop = props.output.transform.crop;
    if (crop) {
      setCropX(crop.x);
      setCropY(crop.y);
      setCropW(crop.width);
      setCropH(crop.height);
    }
  });

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
    // Set crop to null by updating the config locally
    // We update via onCropChange with a special flow — clear locally
    const cfg = props.output;
    // We need a way to clear crop. Use updateOutput directly is for Apply.
    // For local clearing, we set the transform.crop to null via parent callback.
    // Since onCropChange always sets a CropRegion, we handle clear differently:
    // We'll modify the output in parent by calling onUpdated after clearing.
    // Simplest: just apply null crop immediately to backend.
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
    const crop: CropRegion = {
      x: field === 'x' ? value : cropX(),
      y: field === 'y' ? value : cropY(),
      width: field === 'width' ? value : cropW(),
      height: field === 'height' ? value : cropH(),
    };
    // Align x and width to 2px
    if (field === 'x') crop.x = Math.round(value / 2) * 2;
    if (field === 'width') crop.width = Math.round(value / 2) * 2;
    props.onCropChange(props.output.id, crop);
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
