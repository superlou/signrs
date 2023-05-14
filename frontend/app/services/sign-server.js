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
      let items = data.items.map((item) => {
        item.name = item.name.replace(this.appPath + '/', '');
        return item;
      });

      items = items.filter((item) => item.name.length > 0);

      if (!deepEqual(this.fileList, items)) {
        this.fileList = items;
      }
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

function arraysEqual(a, b) {
  if (a.length !== b.length) return false;

  a.sort();
  b.sort();

  for (let i = 0; i < a.length; i++) {
    if (a[i] !== b[i]) return false;
  }

  return true;
}

function deepEqual(object1, object2) {
  const keys1 = Object.keys(object1);
  const keys2 = Object.keys(object2);

  if (keys1.length !== keys2.length) {
    return false;
  }

  for (const key of keys1) {
    const val1 = object1[key];
    const val2 = object2[key];
    const areObjects = isObject(val1) && isObject(val2);
    if (
      areObjects && !deepEqual(val1, val2) ||
      !areObjects && val1 !== val2
    ) {
      return false;
    }
  }

  return true;
}

function isObject(object) {
  return object != null && typeof object === 'object';
}