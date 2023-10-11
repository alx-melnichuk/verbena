import { AUTHORIZATION_DENIED, AUTHORIZATION_REQUIRED } from './routes';

export class Authorization {
  public static isAuthorizationRequired(): boolean {
    const currentRoute = window.location.pathname;
    return AUTHORIZATION_REQUIRED.findIndex((item) => currentRoute.startsWith(item)) > -1;
  }

  public static isAuthorizationDenied(): boolean {
    const currentRoute = window.location.pathname;
    return AUTHORIZATION_DENIED.findIndex((item) => currentRoute.startsWith(item)) > -1;
  }

  public static isAuthorizationNotRequiredNotDenied(): boolean {
    return !Authorization.isAuthorizationDenied() && !Authorization.isAuthorizationDenied();
  }
}
