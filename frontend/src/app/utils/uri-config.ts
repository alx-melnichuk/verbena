import { environment } from 'src/environments/environment';
import { Uri } from './uri';

export class UriConfig {
  public static initial(): void {
    Uri.replace('appRoot://', environment.appRoot);
    Uri.replace('appApi://', environment.appApi);
  }
}
