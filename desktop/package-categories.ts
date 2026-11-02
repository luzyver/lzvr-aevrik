import definitions from '../crates/aevrik-engine/src/packages/catalog.categories.json' with {type:'json'};
import type {RuntimePackage} from './package-model';

export function packageCategory(package_:RuntimePackage):string {
 return Object.entries(definitions).find(([,families])=>families.includes(package_.family??''))?.[0]??package_.category;
}
export function packageCategories(items:RuntimePackage[]):string[] {
 const present=new Set(items.map(packageCategory));
 return ['All',...Object.keys(definitions).filter(category=>present.has(category)),
  ...[...present].filter(category=>!(category in definitions)).sort()];
}
