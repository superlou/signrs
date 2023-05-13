import Modifier from 'ember-modifier';
import { registerDestructor } from '@ember/destroyable';
import { basicSetup } from 'codemirror';
import { EditorState } from '@codemirror/state';
import { EditorView } from '@codemirror/view';
import { javascript } from '@codemirror/lang-javascript';
import { json } from '@codemirror/lang-json';

export default class LoadCodemirrorModifier extends Modifier {
  editorView = null;
  
  modify(element, positional /*, named*/) {
    let source = positional[0];
    let filename = positional[1];

    let extensions = [basicSetup];
    
    if (filename.endsWith(".js")) {
      extensions.push(javascript());
    } else if (filename.endsWith(".json")) {
      extensions.push(json());
    }
    
    let state = EditorState.create({
      doc: source,
      extensions: extensions,
    });
    
    if (this.editorView === null) {
      console.log("setup");
      this.editorView = new EditorView({
        parent: element,
        state: state,
      });
      registerDestructor(this, cleanup);
    } else {
      this.editorView.setState(state);
    }
  }
}

function cleanup(instance) {
  let { element, event, handler } = instance;
  this.editorView.destroy();
}