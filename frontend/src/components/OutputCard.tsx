import { Component, createSignal } from 'solid-js';
import type { OutputConfig } from '../api/types';
import { updateOutput } from '../api/client';

interface Props {
  output: OutputConfig;
  onUpdated: () => void;
}

const OutputCard: Component<Props> = (props) => {
  const [flipH, setFlipH] = createSignal(props.output.transform.flip.horizontal);
  const [flipV, setFlipV] = createSignal(props.output.transform.flip.vertical);
  const [error, setError] = createSignal('');

  const applyFlip = async () => {
    try {
      setError('');
      await updateOutput(props.output.id, {
        crop: props.output.transform.crop,
        flip: { horizontal: flipH(), vertical: flipV() },
      });
      props.onUpdated();
    } catch (e) {
      setError(e instanceof Error ? e.message : 'Failed');
    }
  };

  return (
    <div class="output-card">
      <h3>{props.output.name} ({props.output.id})</h3>
      <div class="fields">
        <label>Mode:</label>
        <span>{props.output.display_mode}</span>
        <label>Format:</label>
        <span>{props.output.pixel_format}</span>
        <label>Device:</label>
        <span>#{props.output.device_index}</span>
      </div>
      <div style={{ "margin-top": "8px" }}>
        <div class="checkbox-row">
          <input type="checkbox" checked={flipH()} onChange={(e) => setFlipH(e.target.checked)} />
          <label>Flip H</label>
          <input type="checkbox" checked={flipV()} onChange={(e) => setFlipV(e.target.checked)} />
          <label>Flip V</label>
          <button class="btn-action" onClick={applyFlip}>Apply</button>
        </div>
      </div>
      {error() && <div class="error-msg">{error()}</div>}
    </div>
  );
};

export default OutputCard;
