import type {Service} from './bridge';

export function needsNodeScript(service:Service):boolean {
 return !service.npmScript&&service.args.length===0&&/(?:^|[\\/])node\.exe$/i.test(service.executable);
}
