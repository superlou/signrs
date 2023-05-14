import Component from '@glimmer/component';
import { action } from '@ember/object';
import { tracked } from '@glimmer/tracking';
import { Folder, LanguageJavascript, CodeJson, TextBox, File } from 'ember-mdi'
import { FormatFont, FileImage, ChevronRight, ChevronDown } from 'ember-mdi'

export default class FileTreeItemComponent extends Component {
  @tracked expanded = false;
  
  get icon() {
    if (this.args.item.isDir) {
      // return Folder;
      
      if (this.expanded) {
        return ChevronDown;
      } else {
        return ChevronRight;
      }
      
      
    } else {
      let name = this.args.item.name;
      if (name.endsWith('.js')) {
        return LanguageJavascript;
      } else if (name.endsWith('.json')) {
        return CodeJson;
      } else if (name.endsWith('.txt')) {
        return TextBox; 
      } else if (name.endsWith('.ttf')) {
        return FormatFont;
      } else if (name.endsWith('.otf')) {
        return FormatFont;
      } else if (name.endsWith('.png')) {
        return FileImage;  
      } else if (name.endsWith('.jpg')) {
        return FileImage;                  
      } else {
        return File;
      }
    }
  }
  
  @action
  toggleExpanded() {
    this.expanded = !this.expanded;
  }
}
