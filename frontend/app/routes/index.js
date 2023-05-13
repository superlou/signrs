import Route from '@ember/routing/route';
import { service } from '@ember/service';

export default class IndexRoute extends Route {
  @service signServer;

  model(params) {
    return {
      edit: params.edit,
    };
  }

  async setupController(controller, model) {
    controller.source = await this.signServer.getSource(model.edit);
  }
}
