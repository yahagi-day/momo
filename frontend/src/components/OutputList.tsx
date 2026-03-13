import { Component, For } from 'solid-js';
import type { OutputConfig } from '../api/types';
import OutputCard from './OutputCard';

interface Props {
  outputs: OutputConfig[];
  onUpdated: () => void;
}

const OutputList: Component<Props> = (props) => {
  return (
    <div class="panel">
      <h2>Outputs</h2>
      <div class="output-list">
        <For each={props.outputs}>
          {(output) => <OutputCard output={output} onUpdated={props.onUpdated} />}
        </For>
      </div>
    </div>
  );
};

export default OutputList;
