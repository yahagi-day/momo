import { Component, Index } from 'solid-js';
import type { OutputConfig, CropRegion } from '../api/types';
import { OUTPUT_COLORS } from '../utils/coordinates';
import OutputCard from './OutputCard';

interface Props {
  outputs: OutputConfig[];
  onUpdated: () => Promise<void> | void;
  selectedOutputId: string | null;
  onSelectOutput: (id: string | null) => void;
  onCropChange: (id: string, crop: CropRegion) => void;
}

const OutputList: Component<Props> = (props) => {
  return (
    <div class="panel">
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
            />
          )}
        </Index>
      </div>
    </div>
  );
};

export default OutputList;
