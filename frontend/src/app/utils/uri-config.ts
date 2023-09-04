import { Uri } from './uri';

export class UriConfig {
  public static initial(appRoot: string, appApi: string): void {
    Uri.replace('appRoot://', appRoot);
    Uri.replace('appApi://', appApi);
  }
}
