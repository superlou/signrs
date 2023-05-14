import Component from '@glimmer/component';

export default class FileTreeComponent extends Component {
  get fileTree() {
    let items = [...this.args.items];   
    
    items.sort((a, b) => {
      if (a.is_dir && !b.is_dir) {
        return -1;
      } else if (!a.is_dir && b.is_dir) {
        return 1;
      } else {
        return a.name.localeCompare(b.name);
      }
    });
    
    let tree = {
      name: "/",
      isDir: true,
      children: [],
    };
    
    items.forEach((item) => {
      // console.log(`--- ${item.name}`);
      let parts = item.name.split('/');
      
      let currentParent = tree;
      
      parts.forEach((part) => {
        // console.log(`part: ${part}`);
        let parent = currentParent.children.find((item) => item.name == part);
        if (parent === undefined) {
          let newParent = {
            name: part,
            fullName: item.name,
            isDir: item.is_dir,
            children: [],
          };
          
          currentParent.children.push(newParent);
          currentParent = newParent;
        } else {
          currentParent = parent;
        }
      });
      
      return item;
    });
    
    // console.log(tree);

    return tree;
  }
}
