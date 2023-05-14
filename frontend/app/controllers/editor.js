import Controller from '@ember/controller';
import { service } from '@ember/service';
import { tracked } from '@glimmer/tracking';
import { action } from '@ember/object';

export default class EditorController extends Controller {
  queryParams = ['edit'];
  @tracked edit = null;

  @service signServer;

  @tracked source = null;

  @action
  async editFile(path) {
    this.edit = path;
    this.source = await this.signServer.getSource(path);
  }
  
  @action
  async save(content, path) {
    console.log(`Saving ${path}`);
    await this.signServer.putSource(path, content);
  }
}
