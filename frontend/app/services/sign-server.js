import Service from '@ember/service';
import { task, timeout } from 'ember-concurrency';
import { tracked } from '@glimmer/tracking';

export default class SignServerService extends Service {
  init() {
    super.init(...arguments);
    this.getStatus.perform();
  }

  @tracked appPath = null;
  @tracked fullScreen = null;
  @tracked fileList = [];

  getStatus = task(async () => {
    try {
      let response = await fetch('http://localhost:3000/api/status');
      let data = await response.json();
      this.appPath = data.root_path;
      this.fullScreen = data.is_fullscreen;
    } catch (error) {
      this.appPath = null;
      this.fullScreen = null;
    }

    try {
      let response = await fetch('http://localhost:3000/api/fs/');
      let data = await response.json();
      let paths = data.contents.map((path) => {
        return path.replace(this.appPath + '/', '');
      });
      this.fileList = paths;
    } catch (error) {
      this.fileList = [];
    }

    await timeout(1000);
    this.getStatus.perform();
  });

  async getSource(path) {
    let response = await fetch('http://localhost:3000/api/fs/' + path);
    let data = await response.json();

    if (data.kind === 'file') {
      return data.content;
    }
  }
}
