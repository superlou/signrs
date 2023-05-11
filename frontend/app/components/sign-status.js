import Component from '@glimmer/component';
import { service } from '@ember/service';

export default class SignStatusComponent extends Component {
  @service signServer;
  
  get appPath() {
    return this.signServer.appPath;
  }
  
  get fullScreen() {
    return this.signServer.fullScreen;
  }
}
