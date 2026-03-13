import { Component } from 'solid-js';

interface Props {
  src: string;
  alt: string;
}

const PreviewImage: Component<Props> = (props) => {
  return <img class="preview-img" src={props.src} alt={props.alt} />;
};

export default PreviewImage;
