import { Component, Show } from 'solid-js';
import type { InputSource, PipelineState } from '../api/types';
import PreviewImage from './PreviewImage';

interface Props {
  input: InputSource | null;
  pipelineState: PipelineState;
}

const InputPanel: Component<Props> = (props) => {
  const inputLabel = () => {
    const src = props.input;
    if (!src) return 'Not configured';
    switch (src.type) {
      case 'Mock': return `Mock (${src.width}x${src.height} @ ${src.fps}fps)`;
      case 'DeckLink': return `DeckLink #${src.device_index}`;
      case 'Uvc': return `UVC (${src.device_path})`;
    }
  };

  return (
    <div class="panel">
      <h2>Input</h2>
      <p style={{ "margin-bottom": "12px", "font-size": "0.9rem", color: "#aaa" }}>
        {inputLabel()}
      </p>
      <Show when={props.pipelineState === 'Running'}>
        <PreviewImage src="/api/preview/input" alt="Input preview" />
      </Show>
    </div>
  );
};

export default InputPanel;
