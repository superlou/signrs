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
  
  getStatus = task(async () => {
    try {
      let response = await fetch("http://localhost:3000/api/status");
      let data = await response.json();
      this.appPath = data.root_path;
      this.fullScreen = data.is_fullscreen;
    } catch (error) {
      this.appPath = null;
      this.fullScreen = null;
    }
    
    await timeout(1000);
    this.getStatus.perform();
  });
}
